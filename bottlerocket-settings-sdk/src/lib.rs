#![deny(missing_docs)]
/*!
This crate provides a Rust SDK for building Bottlerocket Settings Extensions.

TODO: docstring (<https://github.com/bottlerocket-os/bottlerocket-settings-sdk/issues/4>)

# Crate Features

By default, all features are enabled; however, the crate allows for disabling types which are used
to build extensions in favor of only providing model definitions. This is useful for cases where a
tool wishes to invoke a settings extension and parse the output.

* **extension** -
  When enabled, this causes the SDK library to expose the `SettingsExtension` type, as well as all
  other utilities required to build a `SettingsExtension` or serve it on the CLI.

* **proto1** -
  When enabled, this allows extensions built against the SDK to serve the Settings Extension CLI
  protocol version "proto1".
*/
#[cfg(feature = "extension")]
pub mod cli;
#[cfg(feature = "extension")]
pub mod extension;
pub mod helper;
#[cfg(feature = "extension")]
pub mod migrate;
pub mod model;

#[cfg(feature = "extension")]
pub use crate::extension::SettingsExtension;
pub use helper::{HelperDef, HelperError};
#[cfg(feature = "extension")]
pub use migrate::{
    LinearMigrator, LinearMigratorExtensionBuilder, LinearMigratorModel, LinearlyMigrateable,
    Migrator, NoMigration,
};

pub use model::{BottlerocketSetting, GenerateResult, SettingsModel};

#[doc(hidden)]
#[cfg(feature = "extension")]
pub mod example;
