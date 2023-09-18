use std::convert::Infallible;

use super::*;
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MotdV1(pub Option<String>);

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for MotdV1 {
    /// We only have one value, so there's no such thing as a partial
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(
        // We allow any transition from current value to target, so we don't need the current value
        _current_value: Option<Self>,
        target: Self,
    ) -> Result<Self> {
        // Allow anything that parses as MotdV1
        Ok(target)
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        // We do not depend on any settings
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<bottlerocket_settings_sdk::GenerateResult<Self::PartialKind, Self>> {
        // We generate a default motd if there is none.
        Ok(bottlerocket_settings_sdk::GenerateResult::Complete(
            existing_partial.unwrap_or(MotdV1::default()),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<bool> {
        // No need to do any additional validation, any MotdV1 is acceptable
        Ok(true)
    }
}

impl LinearlyMigrateable for MotdV1 {
    type ForwardMigrationTarget = MotdV2;
    type BackwardMigrationTarget = NoMigration;

    /// We migrate forward by splitting the motd on whitespace
    fn migrate_forward(self) -> Result<Self::ForwardMigrationTarget> {
        let Self(inner_value) = self;

        let v2_value = inner_value
            .map(|inner_value| inner_value.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();

        Ok(MotdV2(v2_value))
    }

    fn migrate_backward(self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}

#[test]
fn test_motdv1_set_success() {
    // When set is called on motdv1 with a string input,
    // Then that input is successfully set.
    vec![json!("Hello!"), json!("")]
        .into_iter()
        .for_each(|value| {
            assert_eq!(
                set_cli(motd_settings_extension(), "v1", value.clone()).unwrap(),
                value
            )
        });
}

#[test]
fn test_motdv1_set_failure() {
    // When set is called on motdv1 with a non-string input,
    // Then that set operation fails.
    vec![json!(123456789), json!({"motd": "Hello!"})]
        .into_iter()
        .for_each(
            |value| assert!(set_cli(motd_settings_extension(), "v1", value.clone()).is_err()),
        );
}

#[test]
fn test_motdv1_generate() {
    // When generate is called on motdv1,
    // an empty settings object is returned.
    assert_eq!(
        generate_cli(motd_settings_extension(), "v1", None, None).unwrap(),
        GenerateResult::<MotdV1, MotdV1>::Complete(MotdV1(None))
    );
}

#[test]
fn test_motdv1_validate() {
    // When validate is called on motdv1,
    // it is successful for any value that parses
    assert_eq!(
        validate_cli(motd_settings_extension(), "v1", json!("test"), None).unwrap(),
        json!(true),
    );
}

#[test]
fn test_motdv1_failure() {
    assert!(validate_cli(motd_settings_extension(), "v1", json!([1, 2, 3]), None).is_err(),);
}
