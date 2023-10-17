use super::*;
use bottlerocket_settings_sdk::migrate::LinearMigratorModel;
use bottlerocket_settings_sdk::{
    BottlerocketSetting, LinearMigrator, LinearMigratorExtensionBuilder, SettingsExtension,
};
use serde_json::json;

// These modules implement two versions of the "motd" settings extension, as well as CLI tests
// for each exposed extension method.
mod v1; // models motd as a single string
mod v2; // models motd as a list of strings which don't contain whitespace

pub use v1::MotdV1;
pub use v2::MotdV2;

/// Helper to create the setting extension for these tests.
fn motd_settings_extension() -> SettingsExtension<LinearMigrator, LinearMigratorModel> {
    LinearMigratorExtensionBuilder::with_name("motd")
        .with_models(vec![
            BottlerocketSetting::<v1::MotdV1>::model(),
            BottlerocketSetting::<v2::MotdV2>::model(),
        ])
        .build()
        .expect("Failed to build motd settings extension")
}

#[test]
fn test_target_migration() {
    // When a target migration is called,
    // then an equivalent value for the target version is produced via migrations.
    assert_eq!(
        target_migrate_cli(
            motd_settings_extension(),
            json!("test target migration!"),
            "v1",
            "v2"
        )
        .unwrap(),
        json!(["test", "target", "migration!"])
    );
}

#[test]
fn test_flood_migration() {
    // When flood is called,
    // equivalent values for all versions are produced via migrations.
    let expected_flood_results = json!([
        {
            "version": "v1",
            "value": "test flood migration!"
        },
        {
            "version": "v2",
            "value": ["test", "flood", "migration!"]
        }
    ]);
    assert_eq!(
        flood_migrate_cli(
            motd_settings_extension(),
            json!("test flood migration!"),
            "v1"
        )
        .unwrap(),
        expected_flood_results
    );
    assert_eq!(
        flood_migrate_cli(
            motd_settings_extension(),
            json!(["test", "flood", "migration!"]),
            "v2"
        )
        .unwrap(),
        expected_flood_results
    );
}

#[test]
fn test_migration_types_mutually_exclusive() {
    // When a migration is called with both a target and flood,
    // an error is returned.

    let extension = motd_settings_extension();
    let args = vec![
        "extension",
        "proto1",
        "migrate",
        "--value",
        "test",
        "--from-version",
        "v1",
        "--target-version",
        "v2",
        "--flood",
    ];

    assert!(extension.try_run_with_args(args).is_err())
}
