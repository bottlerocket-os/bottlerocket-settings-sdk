//! An example setting extension for use in doc comments which demonstrate how to implement
//! migrators.
use super::{EmptyError, Result};
use crate::{GenerateResult, SettingsModel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A setting with no defined migrator for use in doc comments.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ScoreV1 {
    scores: HashMap<String, i64>,
}

/// A setting with no defined migrator for use in doc comments.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ScoreV2 {
    all_scores: HashMap<String, i64>,
}

impl SettingsModel for ScoreV1 {
    type PartialKind = Self;
    type ErrorKind = EmptyError;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, target: Self) -> Result<Self> {
        Ok(target)
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(Self::default()))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<bool> {
        Ok(true)
    }
}

impl SettingsModel for ScoreV2 {
    type PartialKind = Self;
    type ErrorKind = EmptyError;

    fn get_version() -> &'static str {
        "v2"
    }

    fn set(_current_value: Option<Self>, target: Self) -> Result<Self> {
        Ok(target)
    }

    fn generate(
        _: Option<Self::PartialKind>,
        _: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(Self::default()))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<bool> {
        Ok(true)
    }
}
