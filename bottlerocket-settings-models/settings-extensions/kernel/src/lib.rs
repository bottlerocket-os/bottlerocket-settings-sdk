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
}
