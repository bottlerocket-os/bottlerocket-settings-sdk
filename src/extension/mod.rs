//! Provides the [`SettingsExtension`] struct, which enables developers to create Bottlerocket
//! settings extensions that adhere to the settings extension CLI protocol.
use crate::cli;
use crate::migrate::{Migrator, ModelStore};
use crate::model::erased::AsModel;
use clap::Parser;
use snafu::ResultExt;
use std::collections::HashMap;
use std::ffi::OsString;
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
    Mo: AsModel,
    Mi: Migrator<ModelKind = Mo>,
{
    name: &'static str,
    models: HashMap<Version, Mo>,
    migrator: Mi,
}

impl<Mi, Mo> SettingsExtension<Mi, Mo>
where
    Mo: AsModel,
    Mi: Migrator<ModelKind = Mo>,
{
    /// Returns a builder used to construct a `SettingsExtension`.
    pub fn with_name(name: &'static str) -> SettingsExtensionBuilder<Mi, Mo> {
        SettingsExtensionBuilder::new(name)
    }

    /// Runs the extension, collecting CLI input from `std::env::args_os()` and deferring behavior
    /// to the provided models, migrator, and helpers.
    ///
    /// Results are printed to stdout/stderr, and the program exits if an error is encountered.
    pub fn run(self) -> ! {
        let args = cli::Cli::parse();
        info!(extension = ?self, protocol = ?args.protocol, "Starting settings extensions");
        debug!(?args, "CLI arguments");

        match args.protocol {
            cli::Protocol::Proto1(p) => proto1::run_extension(self, p.command),
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
        let args = cli::Cli::try_parse_from(iter).context(error::ParseCLIArgsSnafu)?;
        info!(cli_protocol = %args.protocol, "Starting settings extensions.");

        match args.protocol {
            cli::Protocol::Proto1(p) => proto1::try_run_extension(self, p.command),
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
    Mo: AsModel,
    Mi: Migrator<ModelKind = Mo>,
{
    type ModelKind = Mo;

    fn get_model(&self, version: &str) -> Option<&Self::ModelKind> {
        self.model(version)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &Self::ModelKind)> + '_> {
        Box::new(self.iter_models())
    }
}

impl<Mi, Mo> std::fmt::Debug for SettingsExtension<Mi, Mo>
where
    Mo: AsModel,
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

        #[snafu(display(
            "Failed to parse input as requested model version '{}': {}",
            setting_version,
            source
        ))]
        ModelParse {
            setting_version: String,
            source: BottlerocketSettingError,
        },

        #[snafu(display("Requested model version '{}' not found", setting_version))]
        NoSuchModel { setting_version: String },

        #[snafu(display("Failed to parse CLI arguments: {}", source))]
        ParseCLIArgs { source: clap::Error },

        #[snafu(display("Failed to write settings extension output as JSON: {}", source))]
        SerializeResult { source: serde_json::Error },

        #[snafu(display("Set operation failed: {}", source))]
        Set { source: BottlerocketSettingError },

        #[snafu(display("Validate operation failed: {}", source))]
        Validate { source: BottlerocketSettingError },

        _Phantom {
            _make_unconstructable: Infallible,
            _ghost: PhantomData<MigratorError>,
        },
    }
}
