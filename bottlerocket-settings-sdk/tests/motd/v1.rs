use std::convert::Infallible;

use super::*;
use bottlerocket_settings_sdk::{
    provide_template_helpers, GenerateResult, HelperDef, LinearlyMigrateable, NoMigration,
    SettingsModel,
};
use bottlerocket_template_helper::template_helper;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

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
        _target: Self,
    ) -> Result<()> {
        // Allow anything that parses as MotdV1
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        // We do not depend on any settings
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        // We generate a default motd if there is none.
        Ok(GenerateResult::Complete(
            existing_partial.unwrap_or(MotdV1::default()),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        // No need to do any additional validation, any MotdV1 is acceptable
        Ok(())
    }

    fn template_helpers() -> Result<HashMap<String, Box<dyn HelperDef>>> {
        Ok(provide_template_helpers! {
            "exclaim" => exclaim_helper,
        })
    }
}

impl LinearlyMigrateable for MotdV1 {
    type ForwardMigrationTarget = MotdV2;
    type BackwardMigrationTarget = NoMigration;

    /// We migrate forward by splitting the motd on whitespace
    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        let Self(inner_value) = self;

        let v2_value = inner_value
            .as_ref()
            .map(|inner_value| inner_value.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();

        Ok(MotdV2(v2_value))
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}

#[template_helper(ident = exclaim_helper)]
fn exclaim(i: String) -> Result<String> {
    Ok(i + "!")
}

#[test]
fn test_motdv1_set_success() {
    // When set is called on motdv1 with a string input,
    // Then that input is successfully set.
    vec![json!("Hello!"), json!("")]
        .into_iter()
        .for_each(
            |value| assert!(set_cli(motd_settings_extension(), "v1", value.clone()).is_ok(),),
        );
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
    assert!(validate_cli(motd_settings_extension(), "v1", json!("test"), None).is_ok(),);
}

#[test]
fn test_motdv1_failure() {
    assert!(validate_cli(motd_settings_extension(), "v1", json!([1, 2, 3]), None).is_err(),);
}

#[test]
fn test_no_such_helper() {
    assert!(template_helper_cli(motd_settings_extension(), "v1", "no_such_helper", vec![]).is_err())
}

#[test]
fn test_run_exclaim_helper() {
    assert_eq!(
        template_helper_cli(
            motd_settings_extension(),
            "v1",
            "exclaim",
            vec![json!("Hello")]
        )
        .unwrap(),
        json!("Hello!")
    );
}

#[test]
fn test_helper_too_many_args() {
    assert!(template_helper_cli(
        motd_settings_extension(),
        "v1",
        "exclaim",
        vec![json!("too"), json!("many"), json!("arguments")]
    )
    .is_err());
}
