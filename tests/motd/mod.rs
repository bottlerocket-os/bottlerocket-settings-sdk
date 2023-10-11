use super::*;
use bottlerocket_settings_sdk::migrate::LinearMigratorModel;
use bottlerocket_settings_sdk::{BottlerocketSetting, LinearMigrator, SettingsExtension};

// These modules implement two versions of the "motd" settings extension, as well as CLI tests
// for each exposed extension method.
mod v1; // models motd as a single string
mod v2; // models motd as a list of strings which don't contain whitespace

pub use v1::MotdV1;
pub use v2::MotdV2;

/// Helper to create the setting extension for these tests.
fn motd_settings_extension() -> SettingsExtension<LinearMigrator, LinearMigratorModel> {
    SettingsExtension::with_name("motd")
        .with_migrator(LinearMigrator)
        .with_models(vec![
            BottlerocketSetting::<v1::MotdV1>::model(),
            BottlerocketSetting::<v2::MotdV2>::model(),
        ])
        .build()
        .expect("Failed to build motd settings extension")
}
