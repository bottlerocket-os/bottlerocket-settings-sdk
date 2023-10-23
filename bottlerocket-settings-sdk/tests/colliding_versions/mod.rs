use bottlerocket_settings_sdk::{
    BottlerocketSetting, GenerateResult, LinearMigratorExtensionBuilder, LinearlyMigrateable,
    NoMigration, SettingsModel,
};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[test]
fn test_no_colliding_model_versions() {
    // Given two models with the same version string,
    // When an extension is built with those models,
    // The extension will fail to build.

    assert!(
        LinearMigratorExtensionBuilder::with_name("colliding-versions")
            .with_models(vec![
                BottlerocketSetting::<ModelA>::model(),
                BottlerocketSetting::<ModelB>::model(),
            ])
            .build()
            .is_err()
    );
}

#[derive(Debug, Snafu)]
pub struct MyError;

type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ModelA;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ModelB;

impl SettingsModel for ModelA {
    type PartialKind = Self;
    type ErrorKind = MyError;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_: Option<Self>, _: Self) -> Result<()> {
        unimplemented!()
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        unimplemented!()
    }

    fn validate(_: Self, _: Option<serde_json::Value>) -> Result<()> {
        unimplemented!()
    }
}

impl LinearlyMigrateable for ModelA {
    type ForwardMigrationTarget = NoMigration;

    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}

impl SettingsModel for ModelB {
    type PartialKind = Self;
    type ErrorKind = MyError;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_: Option<Self>, _: Self) -> Result<()> {
        unimplemented!()
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        unimplemented!()
    }

    fn validate(_: Self, _: Option<serde_json::Value>) -> Result<()> {
        unimplemented!()
    }
}

impl LinearlyMigrateable for ModelB {
    type ForwardMigrationTarget = NoMigration;

    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}
