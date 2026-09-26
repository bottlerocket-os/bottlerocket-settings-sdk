//! Settings related to Container Runtime
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::deserialize_optional_chunk_size;
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct ContainerRuntimeSettingsV1 {
    max_container_log_line_size: i32,
    max_concurrent_downloads: i32,
    max_concurrent_unpacks: i32,
    #[serde(
        alias = "concurrent-layer-fetch-buffer",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_chunk_size"
    )]
    concurrent_download_chunk_size: i64,
    enable_unprivileged_ports: bool,
    enable_unprivileged_icmp: bool,
    cgroup_writable: bool,
    snapshotter: Snapshotter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
enum Snapshotter {
    #[default]
    Overlayfs,
    Soci,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for ContainerRuntimeSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Set anything that can be parsed as ContainerRuntimeSettingsV1.
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
        // ContainerRuntimeSettingsV1 is validated during deserialization.
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_container_runtime_settings() {
        assert_eq!(
            ContainerRuntimeSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(ContainerRuntimeSettingsV1 {
                max_container_log_line_size: None,
                max_concurrent_downloads: None,
                max_concurrent_unpacks: None,
                concurrent_download_chunk_size: None,
                enable_unprivileged_ports: None,
                enable_unprivileged_icmp: None,
                cgroup_writable: None,
                snapshotter: None,
            }))
        )
    }

    #[test]
    fn test_serde_container_runtime() {
        let test_json = json!({
            "max-container-log-line-size": 1024,
            "max-concurrent-downloads": 5,
            "max-concurrent-unpacks": 5,
            "concurrent-download-chunk-size": "64mb",
            "enable-unprivileged-ports": true,
            "enable-unprivileged-icmp": false,
            "cgroup-writable": true,
            "snapshotter": "soci",
        });

        let test_json_str = test_json.to_string();

        let container_runtime_settings: ContainerRuntimeSettingsV1 =
            serde_json::from_str(&test_json_str).unwrap();

        assert_eq!(
            container_runtime_settings,
            ContainerRuntimeSettingsV1 {
                max_container_log_line_size: Some(1024),
                max_concurrent_downloads: Some(5),
                max_concurrent_unpacks: Some(5),
                concurrent_download_chunk_size: Some(64000000), // 64mb in bytes
                enable_unprivileged_ports: Some(true),
                enable_unprivileged_icmp: Some(false),
                cgroup_writable: Some(true),
                snapshotter: Some(Snapshotter::Soci),
            }
        );

        let serialized_json: serde_json::Value = serde_json::to_string(&container_runtime_settings)
            .map(|s| serde_json::from_str(&s).unwrap())
            .unwrap();

        let expected_json = json!({
            "max-container-log-line-size": 1024,
            "max-concurrent-downloads": 5,
            "max-concurrent-unpacks": 5,
            "concurrent-download-chunk-size": 64000000, // Serialized as number
            "enable-unprivileged-ports": true,
            "enable-unprivileged-icmp": false,
            "cgroup-writable": true,
            "snapshotter": "soci",
        });

        assert_eq!(serialized_json, expected_json);
    }

    #[test]
    fn test_serde_container_runtime_alias() {
        let test_json = json!({
            "max-container-log-line-size": 2048,
            "max-concurrent-downloads": 10,
            "max-concurrent-unpacks": 3,
            "concurrent-layer-fetch-buffer": "128mb",
            "enable-unprivileged-ports": false,
            "enable-unprivileged-icmp": true,
            "cgroup-writable": false,
            "snapshotter": "overlayfs",
        });

        let container_runtime_settings: ContainerRuntimeSettingsV1 =
            serde_json::from_str(&test_json.to_string()).unwrap();

        assert_eq!(
            container_runtime_settings,
            ContainerRuntimeSettingsV1 {
                max_container_log_line_size: Some(2048),
                max_concurrent_downloads: Some(10),
                max_concurrent_unpacks: Some(3),
                concurrent_download_chunk_size: Some(128000000), // 128mb in bytes
                enable_unprivileged_ports: Some(false),
                enable_unprivileged_icmp: Some(true),
                cgroup_writable: Some(false),
                snapshotter: Some(Snapshotter::Overlayfs),
            }
        );
    }

    #[test]
    fn test_optional_concurrent_download_chunk_size() {
        let test_json = json!({
            "snapshotter": "overlayfs",
        });

        let container_runtime_settings: ContainerRuntimeSettingsV1 =
            serde_json::from_str(&test_json.to_string()).unwrap();

        assert_eq!(
            container_runtime_settings.concurrent_download_chunk_size,
            None
        );
    }
}
