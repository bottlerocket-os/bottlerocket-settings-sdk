//! Settings for kubelet DRA (Dynamic Resource Allocation) drivers.
//!
//! Parallels `kubelet-device-plugins`: a per-vendor subtree of settings for the
//! DRA drivers that run on the node. Currently only the NVIDIA GPU DRA driver
//! (`gpu-kubelet-plugin`) is modeled, via `settings.kubelet-dra-drivers.nvidia`.
use bottlerocket_model_derive::model;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use std::convert::Infallible;

/// Settings for the NVIDIA GPU DRA driver. `enabled` toggles the
/// `nvidia-dra-driver-gpu` host service (opt-in; unset is treated as disabled).
/// Nested so future DRA configuration can be added as sibling fields.
#[model(impl_default = true)]
pub struct NvidiaDraDriverSettings {
    enabled: bool,
}

/// KubeletDraDriversV1 holds per-vendor DRA driver settings.
#[model(impl_default = true)]
pub struct KubeletDraDriversV1 {
    nvidia: NvidiaDraDriverSettings,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KubeletDraDriversV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as KubeletDraDriversV1.
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        Ok(GenerateResult::Complete(
            existing_partial.unwrap_or_default(),
        ))
    }

    fn validate(_value: Self, _validated_settings: Option<serde_json::Value>) -> Result<()> {
        // KubeletDraDriversV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default() {
        assert_eq!(
            KubeletDraDriversV1::generate(None, None).unwrap(),
            GenerateResult::Complete(KubeletDraDriversV1 { nvidia: None })
        )
    }

    #[test]
    fn test_serde_nvidia_enabled() {
        let test_json = r#"{"nvidia":{"enabled":true}}"#;
        let settings: KubeletDraDriversV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            settings,
            KubeletDraDriversV1 {
                nvidia: Some(NvidiaDraDriverSettings {
                    enabled: Some(true),
                }),
            }
        );
        assert_eq!(serde_json::to_string(&settings).unwrap(), test_json);
    }

    #[test]
    fn test_serde_empty() {
        let test_json = r#"{}"#;
        let settings: KubeletDraDriversV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(settings, KubeletDraDriversV1 { nvidia: None });
    }
}
