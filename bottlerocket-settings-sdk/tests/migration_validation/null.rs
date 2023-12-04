use super::common::define_model;
use anyhow::Result;
use bottlerocket_settings_sdk::{
    BottlerocketSetting, GenerateResult, NullMigratorExtensionBuilder, SettingsModel,
};
use serde::{Deserialize, Serialize};

define_model!(NullModelA, "v1");
define_model!(NullModelB, "v2");

#[test]
fn test_single_model() {
    NullMigratorExtensionBuilder::with_name("null-migrator")
        .with_models(vec![BottlerocketSetting::<NullModelA>::model()])
        .build()
        .unwrap();
}

#[test]
fn test_multiple_models() {
    assert!(NullMigratorExtensionBuilder::with_name("multiple-models")
        .with_models(vec![
            BottlerocketSetting::<NullModelA>::model(),
            BottlerocketSetting::<NullModelB>::model(),
        ])
        .build()
        .is_err());
}
