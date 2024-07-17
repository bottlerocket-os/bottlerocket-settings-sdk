//! This module implements the Bottlerocket settings extension CLI proto1
//!
//! The protocol is provided as a trait so that any new protocols can provide implementations
//! with function name collisions if needed.
use super::{error, SettingsExtensionError};
use crate::cli::proto1::{
    input::InputFile, FloodMigrateArguments, GenerateArguments, MigrateArguments, Proto1Command,
    SetArguments, TemplateHelperArguments, ValidateArguments,
};
use crate::migrate::Migrator;
use crate::model::erased::AsTypeErasedModel;
use crate::SettingsExtension;
use snafu::{OptionExt, ResultExt};
use std::fmt::Debug;
use std::process::ExitCode;
use tracing::instrument;

/// Runs a proto1 command against the given settings extension.
///
/// Results are printed to stdout/stderr, adhering to Bottlerocket settings extension CLI proto1.
/// Once the extension has run, the program terminates.
pub fn run_extension<P: Proto1>(
    extension: P,
    cmd: Proto1Command,
    input_file: InputFile,
) -> ExitCode {
    match try_run_extension(extension, cmd, input_file) {
        Ok(output) => {
            println!("{}", &output);
            ExitCode::SUCCESS
        }
        Err(e) => {
            println!("{}", e);
            ExitCode::FAILURE
        }
    }
}

/// Runs a proto1 command against the given settings extension.
///
/// The results are returned to the caller.
#[tracing::instrument(err)]
pub fn try_run_extension<P, ME>(
    extension: P,
    cmd: Proto1Command,
    input_file: InputFile,
) -> Result<String, SettingsExtensionError<ME>>
where
    P: Proto1<MigratorErrorKind = ME>,
    ME: std::error::Error + Send + Sync + 'static,
{
    let json_stringify =
        |value| serde_json::to_string_pretty(&value).context(error::SerializeResultSnafu);

    let input = std::fs::read_to_string(&input_file).context(error::ReadInputSnafu {
        filename: input_file.to_string(),
    })?;

    match cmd {
        Proto1Command::Set(_) => {
            let s = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.set(s).map(|_| String::new())
        }
        Proto1Command::Generate(_) => {
            let g = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.generate(g).and_then(json_stringify)
        }
        Proto1Command::Migrate(_) => {
            let m = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.migrate(m).and_then(json_stringify)
        }
        Proto1Command::FloodMigrate(_) => {
            let m = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.flood_migrate(m).and_then(json_stringify)
        }
        Proto1Command::Validate(_) => {
            let v = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.validate(v).map(|_| String::new())
        }
        Proto1Command::Helper(_) => {
            let h = serde_json::from_str(&input).context(error::ParseJSONSnafu)?;
            extension.template_helper(h).and_then(json_stringify)
        }
    }
}

/// A trait representing adherence to Bottlerocket settings extension CLI proto1.
///
/// Implementors of this trait can use `run_extension` to run a proto1 command against a settings extension.
pub trait Proto1: Debug {
    type MigratorErrorKind: std::error::Error + Send + Sync + 'static;

    fn set(
        &self,
        args: SetArguments,
    ) -> Result<(), SettingsExtensionError<Self::MigratorErrorKind>>;
    fn generate(
        &self,
        args: GenerateArguments,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn migrate(
        &self,
        args: MigrateArguments,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn flood_migrate(
        &self,
        args: FloodMigrateArguments,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>>;
    fn validate(
        &self,
        args: ValidateArguments,
    ) -> Result<(), SettingsExtensionError<Self::MigratorErrorKind>>;
    fn template_helper(
        &self,
        args: TemplateHelperArguments,
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
        args: SetArguments,
    ) -> Result<(), SettingsExtensionError<Self::MigratorErrorKind>> {
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
        args: GenerateArguments,
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
        args: MigrateArguments,
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
    fn flood_migrate(
        &self,
        args: FloodMigrateArguments,
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
            .perform_flood_migrations(self, starting_value, &args.from_version)
            .context(error::MigrateSnafu)
            .and_then(|value| serde_json::to_value(value).context(error::SerializeResultSnafu))
    }

    #[instrument(err)]
    fn validate(
        &self,
        args: ValidateArguments,
    ) -> Result<(), SettingsExtensionError<Self::MigratorErrorKind>> {
        self.model(&args.setting_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.setting_version,
            })?
            .as_model()
            .validate(args.value, args.required_settings)
            .context(error::ValidateSnafu)
    }

    fn template_helper(
        &self,
        args: TemplateHelperArguments,
    ) -> Result<serde_json::Value, SettingsExtensionError<Self::MigratorErrorKind>> {
        self.model(&args.setting_version)
            .context(error::NoSuchModelSnafu {
                setting_version: args.setting_version,
            })?
            .as_model()
            .execute_template_helper(&args.helper_name, args.arg)
            .context(error::TemplateHelperSnafu)
    }
}
