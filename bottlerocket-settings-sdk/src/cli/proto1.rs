//! Bottlerocket Settings Extension CLI proto1 definition.
#![allow(missing_docs)]
use argh::FromArgs;

/// Use Settings Extension CLI protocol proto1.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "proto1")]
pub struct Protocol1 {
    /// the command to invoke against the settings extension
    #[argh(subcommand)]
    pub command: Proto1Command,
}

/// The command to invoke against the settings extension.
#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum Proto1Command {
    /// Modify values owned by this setting
    Set(SetCommand),

    /// Generate default values for this setting
    Generate(GenerateCommand),

    /// Validate values created by external settings
    Validate(ValidateCommand),

    /// Migrate this setting from one given version to another
    Migrate(MigrateCommand),

    /// Migrate this setting from one given version to all other known versions
    FloodMigrate(FloodMigrateCommand),

    ///  Execute a helper. Typically this is used to render config templates
    Helper(TemplateHelperCommand),
}

impl Proto1Command {}

/// Validates that a new setting value can be persisted to the Bottlerocket datastore.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "set")]
pub struct SetCommand {
    /// the version of the setting which should be used
    #[argh(option)]
    pub setting_version: String,

    /// the requested value to be set for the incoming setting
    #[argh(option)]
    pub value: serde_json::Value,

    /// the current value of this settings tree
    #[argh(option)]
    pub current_value: Option<serde_json::Value>,
}

/// Dynamically generates a value for this setting given, possibly from other settings.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "generate")]
pub struct GenerateCommand {
    /// the version of the setting which should be used
    #[argh(option)]
    pub setting_version: String,

    /// a json value containing any partially generated data for this setting
    #[argh(option)]
    pub existing_partial: Option<serde_json::Value>,

    /// a json value containing any requested settings partials needed to generate this one
    #[argh(option)]
    pub required_settings: Option<serde_json::Value>,
}

/// Validates an incoming setting, possibly cross-validated with other settings.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "validate")]
pub struct ValidateCommand {
    /// the version of the setting which should be used
    #[argh(option)]
    pub setting_version: String,

    /// a json value containing any partially generated data for this setting
    #[argh(option)]
    pub value: serde_json::Value,

    /// a json value containing any requested settings partials needed to generate this one
    #[argh(option)]
    pub required_settings: Option<serde_json::Value>,
}

/// Migrates a setting value from one version to another.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "migrate")]
pub struct MigrateCommand {
    /// a json value containing the current value of the setting
    #[argh(option)]
    pub value: serde_json::Value,

    /// the version of the settings data being migrated
    #[argh(option)]
    pub from_version: String,

    /// the desired resulting version for the settings data
    #[argh(option)]
    pub target_version: String,
}

/// Migrates a setting value from one version to all other known versions.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "flood-migrate")]
pub struct FloodMigrateCommand {
    /// a json value containing the current value of the setting
    #[argh(option)]
    pub value: serde_json::Value,

    /// the version of the settings data being migrated
    #[argh(option)]
    pub from_version: String,
}

/// Executes a template helper to assist in rendering values to a configuration file.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "helper")]
pub struct TemplateHelperCommand {
    /// the version of the setting which should be used
    #[argh(option)]
    pub setting_version: String,

    /// the name of the helper to call
    #[argh(option)]
    pub helper_name: String,

    /// the arguments for the given helper
    #[argh(option)]
    pub arg: Vec<serde_json::Value>,
}
