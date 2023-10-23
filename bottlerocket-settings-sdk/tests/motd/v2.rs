use std::collections::HashMap;

use super::*;
use anyhow::Result;
use bottlerocket_settings_sdk::{
    provide_template_helpers, GenerateResult, HelperDef, LinearlyMigrateable, NoMigration,
    SettingsModel,
};
use bottlerocket_template_helper::template_helper;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MotdV2(#[serde(default)] pub Vec<String>);

impl SettingsModel for MotdV2 {
    /// We only have one value, so there's no such thing as a partial
    type PartialKind = Self;
    type ErrorKind = anyhow::Error;

    fn get_version() -> &'static str {
        "v2"
    }

    fn set(
        // We allow any transition from current value to target, so we don't need the current value
        _current_value: Option<Self>,
        _target: Self,
    ) -> anyhow::Result<()> {
        // Allow anything that parses as MotdV2
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        // We do not depend on any settings
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<bottlerocket_settings_sdk::GenerateResult<Self::PartialKind, Self>> {
        // We generate a default motd if there is none.
        Ok(bottlerocket_settings_sdk::GenerateResult::Complete(
            existing_partial.unwrap_or(MotdV2(vec![])),
        ))
    }

    fn validate(value: Self, _validated_settings: Option<serde_json::Value>) -> anyhow::Result<()> {
        let Self(inner_strings) = value;

        // No whitespace allowed in any of the substrings
        anyhow::ensure!(!inner_strings
            .iter()
            .any(|i| i.contains(char::is_whitespace)),);

        Ok(())
    }

    fn template_helpers() -> Result<HashMap<String, Box<dyn HelperDef>>> {
        Ok(provide_template_helpers! {
            "exclaim" => exclaim_helper,
            "question" => question_helper,
        })
    }
}

impl LinearlyMigrateable for MotdV2 {
    type ForwardMigrationTarget = NoMigration;
    type BackwardMigrationTarget = MotdV1;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        // Join with a single space character
        let Self(inner_value) = self;

        let v1_value = if inner_value.is_empty() {
            None
        } else {
            Some(inner_value.join(" "))
        };

        Ok(MotdV1(v1_value))
    }
}

#[template_helper(ident = exclaim_helper)]
fn exclaim(i: String) -> Result<String> {
    Ok(i + "!!")
}

#[template_helper(ident = question_helper)]
fn question(one: String, two: String) -> Result<String> {
    Ok(format!("{}? {}??", one, two))
}

#[test]
fn test_motdv2_set_success() {
    // When set is called on motdv2 with allowed input,
    // Then that input is successfully set.
    vec![
        json!(["several,", "strings", "no", "whitespace"]),
        json!([]),
    ]
    .into_iter()
    .for_each(|value| assert!(set_cli(motd_settings_extension(), "v2", value.clone()).is_ok()));
}

#[test]
fn test_motdv2_set_failure() {
    // When set is called on motdv2 with a non-list-of-string input,
    // Then that set operation fails.
    vec![
        json!("Hello!"),
        json!(123456789),
        json!({"motd": "Hello!'"}),
        json!(null),
    ]
    .into_iter()
    .for_each(|value| assert!(set_cli(motd_settings_extension(), "v2", value).is_err()));
}

#[test]
fn test_motdv2_generate() {
    assert_eq!(
        generate_cli(motd_settings_extension(), "v2", None, None).unwrap(),
        GenerateResult::<MotdV2, MotdV2>::Complete(MotdV2(vec![]))
    );
}

#[test]
fn test_run_exclaim_helper() {
    assert_eq!(
        template_helper_cli(
            motd_settings_extension(),
            "v2",
            "exclaim",
            vec![json!("Hello")]
        )
        .unwrap(),
        json!("Hello!!")
    );
}

#[test]
fn test_run_question_helper() {
    assert_eq!(
        template_helper_cli(
            motd_settings_extension(),
            "v2",
            "question",
            vec![json!("two args"), json!("really")]
        )
        .unwrap(),
        json!("two args? really??")
    );
}
