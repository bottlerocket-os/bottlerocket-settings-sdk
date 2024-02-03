//! Bottlerocket Settings Extension CLI proto1 definition.
#![allow(missing_docs)]
use argh::FromArgs;
use serde::{Deserialize, Serialize};

/// Use Settings Extension CLI protocol proto1.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "proto1")]
pub struct Protocol1 {
    /// the command to invoke against the settings extension
    #[argh(subcommand)]
    pub command: Proto1Command,

    #[argh(
        option,
        description = "file that contains input json for the proto1 command"
    )]
    pub input_file: Option<input::InputFile>,
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
pub struct SetCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SetArguments {
    /// the version of the setting which should be used
    pub setting_version: String,

    /// the requested value to be set for the incoming setting
    pub value: serde_json::Value,

    /// the current value of this settings tree
    pub current_value: Option<serde_json::Value>,
}

/// Dynamically generates a value for this setting given, possibly from other settings.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "generate")]
pub struct GenerateCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GenerateArguments {
    /// the version of the setting which should be used
    pub setting_version: String,

    /// a json value containing any partially generated data for this setting
    pub existing_partial: Option<serde_json::Value>,

    /// a json value containing any requested settings partials needed to generate this one
    pub required_settings: Option<serde_json::Value>,
}

/// Validates an incoming setting, possibly cross-validated with other settings.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "validate")]
pub struct ValidateCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ValidateArguments {
    /// the version of the setting which should be used
    pub setting_version: String,

    /// a json value containing any partially generated data for this setting
    pub value: serde_json::Value,

    /// a json value containing any requested settings partials needed to generate this one
    pub required_settings: Option<serde_json::Value>,
}

/// Migrates a setting value from one version to another.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "migrate")]
pub struct MigrateCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MigrateArguments {
    /// a json value containing the current value of the setting
    pub value: serde_json::Value,

    /// the version of the settings data being migrated
    pub from_version: String,

    /// the desired resulting version for the settings data
    pub target_version: String,
}

/// Migrates a setting value from one version to all other known versions.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "flood-migrate")]
pub struct FloodMigrateCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FloodMigrateArguments {
    /// a json value containing the current value of the setting
    pub value: serde_json::Value,

    /// the version of the settings data being migrated
    pub from_version: String,
}

/// Executes a template helper to assist in rendering values to a configuration file.
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "helper")]
pub struct TemplateHelperCommand {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplateHelperArguments {
    /// the version of the setting which should be used
    pub setting_version: String,

    /// the name of the helper to call
    pub helper_name: String,

    /// the arguments for the given helper
    pub arg: Vec<serde_json::Value>,
}

pub mod input {
    use core::fmt::Display;
    use core::str::FromStr;
    use std::convert::Infallible;
    use std::path::Path;

    #[derive(Debug)]
    pub enum InputFile {
        Stdin,
        NormalFile(String),
    }

    impl Display for InputFile {
        fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            match self {
                Self::Stdin => formatter.write_str("stdin"),
                Self::NormalFile(filename) => formatter.write_str(&filename),
            }
        }
    }

    impl Default for InputFile {
        fn default() -> InputFile {
            InputFile::Stdin
        }
    }

    impl AsRef<Path> for InputFile {
        fn as_ref(&self) -> &Path {
            match self {
                Self::Stdin => Path::new("/dev/stdin"),
                Self::NormalFile(filename) => Path::new(filename),
            }
        }
    }

    impl FromStr for InputFile {
        type Err = Infallible;

        fn from_str(input: &str) -> Result<Self, Self::Err> {
            match input {
                "/dev/stdin" => Ok(Self::Stdin),
                "-" => Ok(Self::Stdin),
                "stdin" => Ok(Self::Stdin),
                x => Ok(Self::NormalFile(String::from(x))),
            }
        }
    }
}
