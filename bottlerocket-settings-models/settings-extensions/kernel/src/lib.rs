//! The kernel settings can be used to configure settings related to the kernel, e.g.
//! kernel modules
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::{HugepagesSettings, KmodKey, Lockdown, SysctlKey};
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use std::collections::HashMap;
use std::convert::Infallible;

#[model(impl_default = true)]
struct KernelSettingsV1 {
    lockdown: Lockdown,
    modules: HashMap<KmodKey, KmodSetting>,
    // Values are almost always a single line and often just an integer... but not always.
    sysctl: HashMap<SysctlKey, String>,
    hugepages: HugepagesSettings,
}

#[model]
struct KmodSetting {
    allowed: bool,
    autoload: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KernelSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as KernelSettingsV1
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
        Ok(())
    }
}

/// UKIKernelSettingsV1 is a restricted kernel settings model that only exposes
/// the `lockdown` and `sysctl` settings. The variants using it won't allow the
/// user to configure hugepages and modules settings.
#[model(impl_default = true)]
struct UKIKernelSettingsV1 {
    lockdown: Lockdown,
    // Values are almost always a single line and often just an integer... but not always.
    sysctl: HashMap<SysctlKey, String>,
}

impl SettingsModel for UKIKernelSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as UKIKernelSettingsV1
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bottlerocket_modeled_types::{
        HugepageAllocation, HugepageConfig, HugepageSize, HugepagesStatic, HugepagesTransparent,
        TransparentHugepageDefragPolicy, TransparentHugepagePolicy,
    };

    #[test]
    fn test_generate_kernel() {
        let generated = KernelSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(KernelSettingsV1 {
                lockdown: None,
                modules: None,
                sysctl: None,
                hugepages: None,
            })
        )
    }

    #[test]
    fn test_serde_kernel() {
        let test_json = r#"{
            "lockdown": "integrity",
            "modules": {"foo": {"allowed": true, "autoload": true}},
            "sysctl": {"key": "value"},
            "hugepages": {
                "static": {
                "essential": true,
                "2Mi": {"count": "512"},
                "1Gi": {"count": "4"}
                },
                "transparent": {"enabled": "always", "defrag": "defer+madvise"}
            }
        }"#;

        let kernel: KernelSettingsV1 = serde_json::from_str(test_json).unwrap();

        let mut modules = HashMap::new();
        modules.insert(
            KmodKey::try_from("foo").unwrap(),
            KmodSetting {
                allowed: Some(true),
                autoload: Some(true),
            },
        );
        let modules = Some(modules);

        let mut sysctl = HashMap::new();
        sysctl.insert(SysctlKey::try_from("key").unwrap(), String::from("value"));
        let sysctl = Some(sysctl);

        let mut hugepages_config = HashMap::new();

        hugepages_config.insert(
            HugepageSize::try_from("2Mi").unwrap(),
            HugepageConfig {
                count: HugepageAllocation::try_from("512").unwrap(),
            },
        );
        hugepages_config.insert(
            HugepageSize::try_from("1Gi").unwrap(),
            HugepageConfig {
                count: HugepageAllocation::try_from("4").unwrap(),
            },
        );

        let hugepages = Some(HugepagesSettings {
            transparent_hugepages: Some(HugepagesTransparent {
                enabled: Some(TransparentHugepagePolicy::try_from("always").unwrap()),
                defrag: Some(TransparentHugepageDefragPolicy::try_from("defer+madvise").unwrap()),
            }),
            static_hugepages: Some(HugepagesStatic {
                essential: true,
                hugepages_config: hugepages_config,
            }),
        });

        assert_eq!(
            kernel,
            KernelSettingsV1 {
                lockdown: Some(Lockdown::try_from("integrity").unwrap()),
                modules,
                sysctl,
                hugepages,
            }
        );

        let roundtrip = serde_json::to_value(&kernel).unwrap();
        let hugepages_out = &roundtrip["hugepages"];
        assert_eq!(
            hugepages_out["static"]["2Mi"]["count"],
            serde_json::json!("512")
        );
        assert_eq!(
            hugepages_out["static"]["1Gi"]["count"],
            serde_json::json!("4")
        );
        assert_eq!(
            hugepages_out["static"]["essential"],
            serde_json::json!(true)
        );
        assert_eq!(
            hugepages_out["transparent"]["enabled"],
            serde_json::json!("always")
        );
        assert_eq!(
            hugepages_out["transparent"]["defrag"],
            serde_json::json!("defer+madvise")
        );
    }

    #[test]
    fn test_generate_uki_kernel() {
        let generated = UKIKernelSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(UKIKernelSettingsV1 {
                lockdown: None,
                sysctl: None,
            })
        )
    }

    #[test]
    fn test_serde_uki_kernel_lockdown_only() {
        let test_json = r#"{"lockdown": "integrity"}"#;
        let kernel: UKIKernelSettingsV1 = serde_json::from_str(test_json).unwrap();

        assert_eq!(
            kernel,
            UKIKernelSettingsV1 {
                lockdown: Some(Lockdown::try_from("integrity").unwrap()),
                sysctl: None,
            }
        );
    }

    #[test]
    fn test_serde_uki_kernel_lockdown_sysctl() {
        let test_json =
            r#"{"lockdown": "integrity", "sysctl": { "key1": "value1", "key2": "value2" }}"#;
        let kernel: UKIKernelSettingsV1 = serde_json::from_str(test_json).unwrap();

        let mut sysctl = HashMap::new();
        sysctl.insert(SysctlKey::try_from("key1").unwrap(), String::from("value1"));
        sysctl.insert(SysctlKey::try_from("key2").unwrap(), String::from("value2"));

        assert_eq!(
            kernel,
            UKIKernelSettingsV1 {
                lockdown: Some(Lockdown::try_from("integrity").unwrap()),
                sysctl: Some(sysctl),
            }
        );
    }

    #[test]
    fn test_uki_kernel_rejects_removed_fields() {
        // UKIKernelSettingsV1 only exposes `lockdown` and `sysctl`; attempting to set
        // `modules` or `hugepages` must fail to deserialize.
        let with_modules =
            r#"{"lockdown": "integrity", "modules": {"foo": {"allowed": true, "autoload": true}}}"#;
        assert!(serde_json::from_str::<UKIKernelSettingsV1>(with_modules).is_err());

        let with_hugepages =
            r#"{"lockdown": "integrity", "hugepages": {"transparent": {"enabled": "always"}}}"#;
        assert!(serde_json::from_str::<UKIKernelSettingsV1>(with_hugepages).is_err());
    }
}
