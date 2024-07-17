//! Provides the [`SettingsExtension`] struct, which enables developers to create Bottlerocket
//! settings extensions that adhere to the settings extension CLI protocol.
use crate::cli;
use crate::migrate::{Migrator, ModelStore};
use crate::model::erased::AsTypeErasedModel;
use argh::FromArgs;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::process::ExitCode;
use tracing::{debug, info};

mod builder;
mod proto1;
pub use self::builder::SettingsExtensionBuilder;
pub use error::SettingsExtensionError;

// Type alias to clarify intent of some strings.
type Version = String;

/// The Bottlerocket settings system uses executable modules, called "settings extensions", to
/// provide different settings with customizable behavior for any given Bottlerocket variant.
/// These settings extensions respond to the Bottlerocket Settings Extensions CLI protocol.
///
/// [`SettingsExtension`] provides a mechanism to transform a set of Rust structures implementing
/// [`SettingsModel`](crate::model::SettingsModel) into an executable which follows the settings
/// extension protocol.
pub struct SettingsExtension<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    name: &'static str,
    models: HashMap<Version, Mo>,
    migrator: Mi,
}

impl<Mi, Mo> SettingsExtension<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    /// Creates a new [`SettingsExtension`].
    ///
    /// Returns an error if the given models have a version naming collision, or if any written
    /// migrations are deemed invalid.
    pub fn new(
        name: &'static str,
        models: Vec<Mo>,
        migrator: Mi,
    ) -> Result<Self, SettingsExtensionError<Mi::ErrorKind>> {
        let models = Self::build_model_map(models)?;

        let extension = Self {
            name,
            models,
            migrator,
        };

        extension.validate_migrations()?;

        Ok(extension)
    }

    /// Converts a list of models into a map of Version => Model while checking for uniqueness.
    fn build_model_map(
        models: Vec<Mo>,
    ) -> Result<HashMap<Version, Mo>, SettingsExtensionError<Mi::ErrorKind>> {
        let mut unique_models: HashSet<&str> = HashSet::new();

        debug!("Checking each model for unique versioning.");
        models
            .into_iter()
            .map(|model| {
                let version = model.as_model().get_version();

                ensure!(
                    !unique_models.contains(version),
                    error::ModelVersionCollisionSnafu {
                        version: version.to_string(),
                    }
                );
                unique_models.insert(version);

                Ok((version.to_string(), model))
            })
            .collect()
    }

    /// Runs the migrator's validator against the extension's models.
    fn validate_migrations(&self) -> Result<(), SettingsExtensionError<Mi::ErrorKind>> {
        self.migrator
            .validate_migrations(self)
            .context(error::MigrationValidationSnafu)
    }

    /// Runs the extension, collecting CLI input from `std::env::args_os()` and deferring behavior
    /// to the provided models, migrator, and helpers.
    ///
    /// Results are printed to stdout/stderr, and an ExitCode is returned which should be used for
    /// the settings extension.
    ///
    /// Users of this method should not separately write to `stdout`, as this could break adherence
    /// to the settings extension CLI protocol.
    pub fn run(self) -> ExitCode {
        let args: cli::Cli = argh::from_env();
        info!(extension = ?self, protocol = ?args.protocol, "Starting settings extensions");
        debug!(?args, "CLI arguments");

        match args.protocol {
            cli::Protocol::Proto1(p) => {
                proto1::run_extension(self, p.command, p.input_file.unwrap_or_default())
            }
        }
    }

    /// Runs the extension using the given CLI input and deferring behavior to the provided models,
    /// migrator, and helpers.
    pub fn try_run_with_args<I, T>(
        self,
        iter: I,
    ) -> Result<String, SettingsExtensionError<Mi::ErrorKind>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let all_inputs: Vec<String> = iter
            .into_iter()
            .map(|s| s.into().to_string_lossy().into_owned())
            .collect();

        let mut input_iter = all_inputs.iter().map(AsRef::as_ref);
        let command_name = [input_iter.next().context(error::ParseCLICommandSnafu)?];
        let args: Vec<&str> = input_iter.collect();

        let args = cli::Cli::from_args(&command_name, &args).map_err(|e| {
            error::SettingsExtensionError::ParseCLIArgs {
                parser_output: e.output,
            }
        })?;

        info!(cli_protocol = %args.protocol, "Starting settings extensions.");

        match args.protocol {
            cli::Protocol::Proto1(p) => {
                proto1::try_run_extension(self, p.command, p.input_file.unwrap_or_default())
            }
        }
    }

    /// Returns a settings model with the given version.
    pub fn model(&self, version: &str) -> Option<&Mo> {
        self.models.get(version)
    }

    /// Returns an iterator over all stored models, with no guaranteed order.
    pub fn iter_models(&self) -> impl Iterator<Item = (&str, &Mo)> {
        self.models.iter().map(|(k, v)| (k.as_str(), v))
    }
}

impl<Mi, Mo> ModelStore for SettingsExtension<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    type ModelKind = Mo;

    fn get_model(&self, version: &str) -> Option<&Self::ModelKind> {
        self.model(version)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &Self::ModelKind)> + '_> {
        Box::new(self.iter_models())
    }

    fn len(&self) -> usize {
        self.models.len()
    }
}

impl<Mi, Mo> std::fmt::Debug for SettingsExtension<Mi, Mo>
where
    Mo: AsTypeErasedModel,
    Mi: Migrator<ModelKind = Mo>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsExtension")
            .field("name", &self.name)
            .field("model-versions", &self.models.keys().collect::<Vec<_>>())
            .field("migrator", &self.migrator)
            .finish()
    }
}

pub mod error {
    #![allow(missing_docs)]
    // `SettingsExtensionError` needs an enum variant to carry a PhantomData marker.
    // We make that variant inconstructable using `Infallible`, but this causes code inside snafu's
    // derive macro to be unreachable.
    #![allow(unreachable_code)]

    use std::convert::Infallible;
    use std::marker::PhantomData;

    use snafu::Snafu;

    use crate::model::BottlerocketSettingError;

    /// The error type returned when running a settings extension.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum SettingsExtensionError<MigratorError>
    where
        MigratorError: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        #[snafu(display("Generate operation failed: {}", source))]
        Generate { source: BottlerocketSettingError },

        #[snafu(display("Migrate operation failed: {}", source))]
        Migrate {
            #[snafu(source(from(MigratorError, Into::into)))]
            source: Box<dyn std::error::Error + Send + Sync + 'static>,
        },

        #[snafu(display("Failed to validate model migrations: {}", source))]
        MigrationValidation {
            #[snafu(source(from(MigratorError, Into::into)))]
            source: Box<dyn std::error::Error + Send + Sync + 'static>,
        },

        #[snafu(display(
            "Failed to parse input as requested model version '{}': {}",
            setting_version,
            source
        ))]
        ModelParse {
            setting_version: String,
            source: BottlerocketSettingError,
        },

        #[snafu(display("Models have colliding version '{}'", version))]
        ModelVersionCollision { version: String },

        #[snafu(display("Requested model version '{}' not found", setting_version))]
        NoSuchModel { setting_version: String },

        #[snafu(display("Failed to parse CLI arguments: No CLI command given"))]
        ParseCLICommand,

        #[snafu(display("Failed to parse CLI arguments: {}", parser_output))]
        ParseCLIArgs { parser_output: String },

        #[snafu(display("Failed to parse to JSON: {}", source))]
        ParseJSON { source: serde_json::Error },

        #[snafu(display("Failed to read from '{}': {}", filename, source))]
        ReadInput {
            filename: String,
            source: std::io::Error,
        },

        #[snafu(display("Failed to write settings extension output as JSON: {}", source))]
        SerializeResult { source: serde_json::Error },

        #[snafu(display("Set operation failed: {}", source))]
        Set { source: BottlerocketSettingError },

        #[snafu(display("Template helper execution failed: {}", source))]
        TemplateHelper { source: BottlerocketSettingError },

        #[snafu(display("Validate operation failed: {}", source))]
        Validate { source: BottlerocketSettingError },

        _Phantom {
            _make_unconstructable: Infallible,
            _ghost: PhantomData<MigratorError>,
        },
    }
}
