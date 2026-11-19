//! Settings related to Kubelet Device Plugins
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::NvidiaDevicePluginSettings;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct KubeletDevicePluginsV1 {
    nvidia: NvidiaDevicePluginSettings,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KubeletDevicePluginsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as KubeletDevicePluginsV1.
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
        // KubeletDevicePluginsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bottlerocket_modeled_types::{
        MigProfile, NvidiaDeviceIdStrategy, NvidiaDeviceListStrategy,
        NvidiaDeviceListStrategyValues, NvidiaDevicePartitioningStrategy,
        NvidiaDeviceSharingStrategy, NvidiaDisableHealthChecks, NvidiaGpuModel, NvidiaMigSettings,
        NvidiaMpsSettings, NvidiaTimeSlicingSettings,
    };
    use bounded_integer::BoundedI32;
    use std::collections::HashMap;

    #[test]
    fn test_generate_kubelet_device_plugins() {
        let generated = KubeletDevicePluginsV1::generate(None, None).unwrap();
        assert_eq!(
            generated,
            GenerateResult::Complete(KubeletDevicePluginsV1 { nvidia: None })
        );
    }

    #[test]
    fn test_serde_kubelet_device_plugins_with_mps() {
        let test_json = r#"{"nvidia":{"pass-device-specs":true,"device-id-strategy":"index","device-list-strategy":"volume-mounts","device-sharing-strategy":"mps","mps":{"replicas":4},"device-partitioning-strategy":"none"}}"#;

        let device_plugins: KubeletDevicePluginsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            device_plugins
                .nvidia
                .as_ref()
                .unwrap()
                .device_sharing_strategy,
            Some(NvidiaDeviceSharingStrategy::Mps)
        );
    }

    #[test]
    fn test_serde_kubelet_device_plugins_with_disable_health_checks() {
        let test_json = r#"{"nvidia":{"enabled":true,"pass-device-specs":true,"device-id-strategy":"index","device-list-strategy":"volume-mounts","disable-health-checks":"94"}}"#;
        let device_plugins: KubeletDevicePluginsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            device_plugins,
            KubeletDevicePluginsV1 {
                nvidia: Some(NvidiaDevicePluginSettings {
                    enabled: Some(true),
                    pass_device_specs: Some(true),
                    device_id_strategy: Some(NvidiaDeviceIdStrategy::Index),
                    device_list_strategy: Some(NvidiaDeviceListStrategy::Scalar(
                        NvidiaDeviceListStrategyValues::VolumeMounts
                    )),
                    disable_health_checks: Some(NvidiaDisableHealthChecks::try_from("94").unwrap()),
                    ..NvidiaDevicePluginSettings::default()
                })
            }
        );
        let serialized = serde_json::to_string(&device_plugins).unwrap();
        assert_eq!(serialized, test_json);

        // No cross-field rules apply; the value is accepted as-is.
        assert!(KubeletDevicePluginsV1::validate(device_plugins, None).is_ok());
    }

    #[test]
    fn test_serde_kubelet_device_plugins_disable_health_checks_all() {
        let test_json = r#"{"nvidia":{"disable-health-checks":"all"}}"#;
        let device_plugins: KubeletDevicePluginsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            device_plugins
                .nvidia
                .as_ref()
                .unwrap()
                .disable_health_checks
                .as_ref()
                .map(|v| v.as_ref()),
            Some("all")
        );
        assert_eq!(serde_json::to_string(&device_plugins).unwrap(), test_json);
    }

    #[test]
    fn test_serde_kubelet_device_plugins_rejects_bad_disable_health_checks() {
        for value in &[
            "", "0", "94,", "94, 95", "94,94", "xids", "all,94", "garbage",
        ] {
            let test_json = format!(r#"{{"nvidia":{{"disable-health-checks":"{value}"}}}}"#);
            let result: std::result::Result<KubeletDevicePluginsV1, _> =
                serde_json::from_str(&test_json);
            assert!(result.is_err(), "{value:?} should be rejected");
        }
    }

    #[test]
    fn test_serde_kubelet_device_plugins_numeric_disable_health_checks() {
        // `apiclient set ...disable-health-checks=94` and unquoted TOML send a number.
        let device_plugins: KubeletDevicePluginsV1 =
            serde_json::from_str(r#"{"nvidia":{"disable-health-checks":94}}"#).unwrap();
        assert_eq!(
            device_plugins
                .nvidia
                .as_ref()
                .unwrap()
                .disable_health_checks
                .as_ref()
                .map(|v| v.as_ref()),
            Some("94")
        );
        assert_eq!(
            serde_json::to_string(&device_plugins).unwrap(),
            r#"{"nvidia":{"disable-health-checks":"94"}}"#
        );
        assert!(KubeletDevicePluginsV1::validate(device_plugins, None).is_ok());

        for value in &["0", "-94", "94.5", "true"] {
            let test_json = format!(r#"{{"nvidia":{{"disable-health-checks":{value}}}}}"#);
            let result: std::result::Result<KubeletDevicePluginsV1, _> =
                serde_json::from_str(&test_json);
            assert!(result.is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn test_serde_kubelet_device_plugins_vec() {
        let test_json = r#"{"nvidia":{"pass-device-specs":true,"device-id-strategy":"index","device-list-strategy":["volume-mounts","envvar"],"device-sharing-strategy":"time-slicing","time-slicing":{"replicas":2,"rename-by-default":true,"fail-requests-greater-than-one":true},"mps":{},"device-partitioning-strategy":"mig","mig":{"profile":{"a100.40gb":"1g.5gb"}}}}"#;

        let device_plugins: KubeletDevicePluginsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            device_plugins,
            KubeletDevicePluginsV1 {
                nvidia: Some(NvidiaDevicePluginSettings {
                    enabled: None,
                    pass_device_specs: Some(true),
                    device_id_strategy: Some(NvidiaDeviceIdStrategy::Index),
                    device_list_strategy: Some(NvidiaDeviceListStrategy::Vector(vec![
                        NvidiaDeviceListStrategyValues::VolumeMounts,
                        NvidiaDeviceListStrategyValues::Envvar,
                    ])),
                    device_sharing_strategy: Some(NvidiaDeviceSharingStrategy::TimeSlicing),
                    time_slicing: Some(NvidiaTimeSlicingSettings {
                        replicas: Some(BoundedI32::new(2).unwrap()),
                        rename_by_default: Some(true),
                        fail_requests_greater_than_one: Some(true),
                    }),
                    mps: Some(NvidiaMpsSettings::default()),
                    device_partitioning_strategy: Some(NvidiaDevicePartitioningStrategy::MIG),
                    mig: Some(NvidiaMigSettings {
                        profile: Some(HashMap::from([(
                            NvidiaGpuModel::try_from("a100.40gb").unwrap(),
                            MigProfile::try_from("1g.5gb").unwrap()
                        )]))
                    }),
                    disable_health_checks: None,
                })
            }
        );
        let serialized = serde_json::to_string(&device_plugins).unwrap();
        assert_eq!(serialized, test_json);
    }

    #[test]
    fn test_serde_kubelet_device_plugins_scalar() {
        let test_json = r#"{"nvidia":{"pass-device-specs":true,"device-id-strategy":"index","device-list-strategy":"volume-mounts","device-sharing-strategy":"time-slicing","time-slicing":{"replicas":2,"rename-by-default":true,"fail-requests-greater-than-one":true},"mps":{},"device-partitioning-strategy":"mig","mig":{"profile":{"a100.40gb":"1g.5gb"}}}}"#;

        let device_plugins: KubeletDevicePluginsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            device_plugins,
            KubeletDevicePluginsV1 {
                nvidia: Some(NvidiaDevicePluginSettings {
                    enabled: None,
                    pass_device_specs: Some(true),
                    device_id_strategy: Some(NvidiaDeviceIdStrategy::Index),
                    device_list_strategy: Some(NvidiaDeviceListStrategy::Scalar(
                        NvidiaDeviceListStrategyValues::VolumeMounts
                    )),
                    device_sharing_strategy: Some(NvidiaDeviceSharingStrategy::TimeSlicing),
                    time_slicing: Some(NvidiaTimeSlicingSettings {
                        replicas: Some(BoundedI32::new(2).unwrap()),
                        rename_by_default: Some(true),
                        fail_requests_greater_than_one: Some(true),
                    }),
                    mps: Some(NvidiaMpsSettings::default()),
                    device_partitioning_strategy: Some(NvidiaDevicePartitioningStrategy::MIG),
                    mig: Some(NvidiaMigSettings {
                        profile: Some(HashMap::from([(
                            NvidiaGpuModel::try_from("a100.40gb").unwrap(),
                            MigProfile::try_from("1g.5gb").unwrap()
                        )]))
                    }),
                    disable_health_checks: None,
                })
            }
        );
        let serialized = serde_json::to_string(&device_plugins).unwrap();
        assert_eq!(serialized, test_json);
    }
}
