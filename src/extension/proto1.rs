//! This module implements the Bottlerocket settings extension CLI proto1
//!
//! The protocol is provided as a trait so that any new protocols can provide implementations
//! with function name collisions if needed.
use super::{error, SettingsExtensionError};
use crate::cli::proto1::{
    GenerateCommand, MigrateCommand, Proto1Command, SetCommand, ValidateCommand,
};
use crate::migrate::Migrator;
use crate::model::erased::AsTypeErasedModel;
use crate::SettingsExtension;
use snafu::{OptionExt, ResultExt};
use std::fmt::Debug;
use tracing::instrument;

/// Runs a proto1 command against the given settings extension.
///
/// Results are printed to stdout/stderr, adhering to Bottlerocket settings extension CLI proto1.
/// Once the extension has run, the program terminates.
pub fn run_extension<P: Proto1>(extension: P, cmd: Proto1Command) -> ! {
    match try_run_extension(extension, cmd) {
        Err(e) => {
            // TODO use machine-readable output on error.
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(output) => {
            println!("{}", &output);
            std::process::exit(0);
        }
    };
}

/// Runs a proto1 command against the given settings extension.
///
/// The results are returned to the caller.
#[tracing::instrument(err)]
pub fn try_run_extension<P, ME>(
    extension: P,
    cmd: Proto1Command,
) -> Result<String, SettingsExtensionError<ME>>
where
    P: Proto1<MigratorErrorKind = ME>,
    ME: std::error::Error + Send + Sync + 'static,
{
    match cmd {
        Proto1Command::Set(s) => extension.set(s),
        Proto1Command::Generate(g) => extension.generate(g),
        Proto1Command::Migrate(m) => extension.migrate(m),
        Proto1Command::Validate(v) => extension.validate(v),
        Proto1Command::Helper(_h) => {
            todo!("https://github.com/bottlerocket-os/bottlerocket-settings-sdk/issues/3")
        }
    }
    .and_then(|value| serde_json::to_string_pretty(&value).context(error::SerializeResultSnafu))
}

/// A trait representing adherence to Bottlerocket settings extension CLI proto1.
///
/// Implementors of this trait can use `run_extension` to run a proto1 command against a settings extension.
pub trait Proto1: Debug {
    type MigratorErrorKind: std::error::Error + Send + Sync + 'static;

    fn set(
        &self,
        args: SetCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn generate(
        &self,
        args: GenerateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn migrate(
        &self,
        args: MigrateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn validate(
        &self,
        args: ValidateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
}

impl<Mi, Mo> Proto1 for SettingsExtension<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    type MigratorErrorKind = Mi::ErrorKind;

    #[instrument(err)]
    fn set(
        &self,
        args: SetCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>> {
        self.model(&args.setting_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.setting_version,
            })?
            .as_model()
            .set(args.current_value, args.value)
            .context(error::SetSnafu)
    }

    #[instrument(err)]
    fn generate(
        &self,
        args: GenerateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>> {
        self.model(&args.setting_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.setting_version,
            })?
            .as_model()
            .generate(args.existing_partial, args.required_settings)
            .context(error::GenerateSnafu)
            .and_then(|generated_data| {
                serde_json::to_value(generated_data).context(error::SerializeResultSnafu)
            })
    }

    #[instrument(err)]
    fn migrate(
        &self,
        args: MigrateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>> {
        let model = self
            .model(&args.from_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.from_version.clone(),
            })?;

        let starting_value =
            model
                .as_model()
                .parse_erased(args.value)
                .context(error::ModelParseSnafu {
                    setting_version: args.from_version.clone(),
                })?;

        self.migrator
            .perform_migration(
                self,
                starting_value,
                &args.from_version,
                &args.target_version,
            )
            .context(error::MigrateSnafu)
    }

    #[instrument(err)]
    fn validate(
        &self,
        args: ValidateCommand,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>> {
        self.model(&args.setting_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.setting_version,
            })?
            .as_model()
            .validate(args.value, args.required_settings)
            .context(error::ValidateSnafu)
            .and_then(|validation| {
                serde_json::to_value(validation).context(error::SerializeResultSnafu)
            })
    }
}
