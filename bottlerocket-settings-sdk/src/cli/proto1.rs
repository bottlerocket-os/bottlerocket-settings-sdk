//! Bottlerocket Settings Extension CLI proto1 definition.
#![allow(missing_docs)]
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct Protocol1 {
    #[command(subcommand)]
    pub command: Proto1Command,
}

#[derive(Subcommand, Debug)]
pub enum Proto1Command {
    /// Modify values owned by this setting
    Set(SetCommand),

    /// Generate default values for this setting
    Generate(GenerateCommand),

    /// Validate values created by external settings
    Validate(ValidateCommand),

    /// Migrate this setting from one given version to another
    Migrate(MigrateCommand),

    ///  Execute a helper. Typically this is used to render config templates
    Helper(TemplateHelperCommand),
}

impl Proto1Command {}

#[derive(Args, Debug)]
pub struct SetCommand {
    /// The version of the setting which should be used
    #[arg(long)]
    pub setting_version: String,

    /// The requested value to be set for the incoming setting
    #[arg(long, value_parser = parse_json)]
    pub value: serde_json::Value,

    /// The current value of this settings tree
    #[arg(long, value_parser = parse_json)]
    pub current_value: Option<serde_json::Value>,
}

#[derive(Args, Debug)]
pub struct GenerateCommand {
    /// The version of the setting which should be used
    #[arg(long)]
    pub setting_version: String,

    /// A json value containing any partially generated data for this setting
    #[arg(long, value_parser = parse_json)]
    pub existing_partial: Option<serde_json::Value>,

    /// A json value containing any requested settings partials needed to generate this one
    #[arg(long, value_parser = parse_json)]
    pub required_settings: Option<serde_json::Value>,
}

#[derive(Args, Debug)]
pub struct ValidateCommand {
    /// The version of the setting which should be used
    #[arg(long)]
    pub setting_version: String,

    /// A json value containing any partially generated data for this setting
    #[arg(long, value_parser = parse_json)]
    pub value: serde_json::Value,

    /// A json value containing any requested settings partials needed to generate this one
    #[arg(long, value_parser = parse_json)]
    pub required_settings: Option<serde_json::Value>,
}

#[derive(Args, Debug)]
pub struct MigrateCommand {
    /// A json value containing the current value of the setting
    #[arg(long, value_parser = parse_json)]
    pub value: serde_json::Value,

    /// The version of the settings data being migrated
    #[arg(long)]
    pub from_version: String,

    /// The desired resulting version for the settings data
    #[arg(long, group = "migration-type")]
    pub target_version: Option<String>,

    /// Triggers a batch migration to all known setting versions
    #[arg(long, group = "migration-type")]
    pub flood: bool,
}

#[derive(Args, Debug)]
pub struct TemplateHelperCommand {
    /// The version of the setting which should be used
    #[arg(long)]
    pub setting_version: String,

    /// The name of the helper to call
    #[arg(long)]
    pub helper_name: String,

    /// The arguments for the given helper
    #[arg(long, value_parser = parse_json)]
    pub arg: Vec<serde_json::Value>,
}

/// Helper for `clap` to parse JSON values.
fn parse_json(arg: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(arg)
}
