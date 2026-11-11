//! Settings for the network DRA (Dynamic Resource Allocation) driver, DRANET.
//!
//! This is a settings extension of its own, not a subtree of
//! `kubelet-dra-drivers`, so that a variant can carry the network driver's
//! settings without also carrying the NVIDIA GPU driver's settings. A settings
//! plugin takes or leaves a whole extension, and the extension name is the
//! top-level settings key, so the two drivers need two keys to be independent.
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::DranetAdvertisedDevices;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use std::convert::Infallible;
use std::net::SocketAddr;

/// KubeletDraDriversNetV1 holds the settings for the network DRA driver
/// (DRANET). `enabled` toggles the `dranet` host service (unset is treated as
/// enabled). `bind_address` is the IP address and port of the driver's metrics
/// and healthz server. `advertised_devices` selects which network devices the
/// node advertises as allocatable, and the host maps the choice to the driver's
/// device filter.
#[model(impl_default = true)]
pub struct KubeletDraDriversNetV1 {
    enabled: bool,
    bind_address: SocketAddr,
    advertised_devices: DranetAdvertisedDevices,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KubeletDraDriversNetV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as KubeletDraDriversNetV1.
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
        // KubeletDraDriversNetV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default() {
        assert_eq!(
            KubeletDraDriversNetV1::generate(None, None).unwrap(),
            GenerateResult::Complete(KubeletDraDriversNetV1 {
                enabled: None,
                bind_address: None,
                advertised_devices: None,
            })
        )
    }

    #[test]
    fn test_serde_all_fields() {
        let test_json =
            r#"{"enabled":true,"bind-address":"127.0.0.1:9177","advertised-devices":"efa"}"#;
        let settings: KubeletDraDriversNetV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            settings,
            KubeletDraDriversNetV1 {
                enabled: Some(true),
                bind_address: Some("127.0.0.1:9177".parse().unwrap()),
                advertised_devices: Some(DranetAdvertisedDevices::Efa),
            }
        );
        assert_eq!(serde_json::to_string(&settings).unwrap(), test_json);
    }

    #[test]
    fn test_serde_rejects_unknown_advertised_devices() {
        for test_json in [
            r#"{"advertised-devices":"all"}"#,
            r#"{"advertised-devices":"ena"}"#,
        ] {
            assert!(serde_json::from_str::<KubeletDraDriversNetV1>(test_json).is_err());
        }
    }

    #[test]
    fn test_serde_rejects_bad_bind_address() {
        for test_json in [
            r#"{"bind-address":"one\ntwo"}"#,
            r#"{"bind-address":"9177"}"#,
            r#"{"bind-address":" 127.0.0.1:9177"}"#,
            r#"{"bind-address":"localhost:9177"}"#,
        ] {
            assert!(serde_json::from_str::<KubeletDraDriversNetV1>(test_json).is_err());
        }
    }

    #[test]
    fn test_serde_empty() {
        let test_json = r#"{}"#;
        let settings: KubeletDraDriversNetV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            settings,
            KubeletDraDriversNetV1 {
                enabled: None,
                bind_address: None,
                advertised_devices: None,
            }
        );
    }
}
