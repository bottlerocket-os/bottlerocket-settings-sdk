/*!
Serialization tests for the settings models exported by this crate.

The `model` macro wraps each field in `Option` and adds
`#[serde(skip_serializing_if = "Option::is_none")]`, but only for fields that
do not already carry a `serde` attribute of their own.  A field that sets, say,
`deserialize_with` therefore loses the skip unless it restates it, and then
serializes as `"field": null` instead of being omitted.  That is how
`fail_cgroup_v1` came to serialize as null before it was fixed in #110.

The existing model tests only cover deserialization (JSON to struct), never the
other direction, so nothing failed when the skip went missing.  The tests here
pin the serialized shape instead:

- a default model serializes to `{}`, not to a set of explicit nulls
- deserializing `{}` and serializing it back is a no-op
- fields with a custom `deserialize_with` or `alias` round trip as expected,
  including the ones that deliberately normalize their input

A field added without the skip now fails here rather than in a downstream kit.
*/

use bottlerocket_settings_models::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

/// Asserts that a default-constructed model serializes to an empty object.
///
/// Every field of a model is an `Option` and `Default` leaves them all `None`,
/// so there is nothing to emit.
fn assert_default_serializes_empty<T>()
where
    T: Default + Serialize,
{
    let serialized = serde_json::to_value(T::default()).expect("failed to serialize default model");

    assert_eq!(
        serialized,
        json!({}),
        "default model serialized to {} instead of {{}}; a field is likely \
         missing `skip_serializing_if = \"Option::is_none\"`",
        serialized,
    );
}

/// Asserts that deserializing an empty object and serializing it back is a
/// no-op.
///
/// This catches fields that get filled in on deserialize but are then emitted
/// unconditionally.
fn assert_empty_object_round_trips<T>()
where
    T: Serialize + DeserializeOwned,
{
    let deserialized: T =
        serde_json::from_value(json!({})).expect("failed to deserialize empty object");
    let serialized =
        serde_json::to_value(&deserialized).expect("failed to re-serialize empty object");

    assert_eq!(
        serialized,
        json!({}),
        "empty object round tripped to {} instead of {{}}",
        serialized,
    );
}

/// Asserts that `input` deserializes and then serializes back to `expected`.
///
/// Pass the same value twice to require an exact round trip; pass a different
/// `expected` to pin a deserializer that normalizes its input, so the
/// normalization is checked rather than just tolerated.
fn assert_serializes_to<T>(input: Value, expected: Value)
where
    T: Serialize + DeserializeOwned,
{
    let deserialized: T = serde_json::from_value(input.clone())
        .unwrap_or_else(|e| panic!("failed to deserialize {input}: {e}"));
    let serialized = serde_json::to_value(&deserialized).expect("failed to re-serialize model");

    assert_eq!(
        serialized, expected,
        "{input} serialized back to unexpected JSON"
    );
}

/// Generates the default-serialization and empty-object round trip tests for
/// each model listed.
macro_rules! test_empty_serialization {
    ($($name:ident: $model:ty,)*) => {
        $(
            mod $name {
                use super::*;

                #[test]
                fn default_serializes_to_empty_object() {
                    assert_default_serializes_empty::<$model>();
                }

                #[test]
                fn empty_object_round_trips() {
                    assert_empty_object_round_trips::<$model>();
                }
            }
        )*
    };
}

test_empty_serialization! {
    autoscaling: AutoScalingSettingsV1,
    aws: AwsSettingsV1,
    boot: BootSettingsV1,
    bootstrap_commands: BootstrapCommandsSettingsV1,
    bootstrap_containers: BootstrapContainersSettingsV1,
    cloudformation: CloudFormationSettingsV1,
    container_registry: RegistrySettingsV1,
    container_runtime: ContainerRuntimeSettingsV1,
    container_runtime_plugins: ContainerRuntimePluginsSettingsV1,
    dns: DnsSettingsV1,
    ecs: ECSSettingsV1,
    host_containers: HostContainersSettingsV1,
    image_verifier_plugins: ImageVerifierPluginsSettingsV1,
    kernel: KernelSettingsV1,
    kubelet_device_plugins: KubeletDevicePluginsV1,
    kubernetes: KubernetesSettingsV1,
    measurement: MeasurementSettingsV1,
    metrics: MetricsSettingsV1,
    network: NetworkSettingsV1,
    ntp: NtpSettingsV1,
    nvidia_container_runtime: NvidiaContainerRuntimeSettingsV1,
    oci_defaults: OciDefaultsV1,
    oci_hooks: OciHooksSettingsV1,
    pki: PkiSettingsV1,
    uki_kernel: UKIKernelSettingsV1,
    updates: UpdatesSettingsV1,
}

/// `MotdV1` is a string setting rather than a struct of fields, so it is not
/// covered by the empty-object tests above.
mod motd {
    use super::*;

    #[test]
    fn default_serializes_to_empty_string() {
        assert_eq!(serde_json::to_value(MotdV1::default()).unwrap(), json!(""),);
    }

    #[test]
    fn round_trips() {
        assert_serializes_to::<MotdV1>(
            json!("welcome to Bottlerocket"),
            json!("welcome to Bottlerocket"),
        );
    }
}

/// Fields whose `serde` attribute replaces the one the `model` macro would have
/// added.  These are the fields where a missing `skip_serializing_if` hides,
/// and where serialization can drift from deserialization.
mod custom_serde_fields {
    use super::*;

    #[test]
    fn kubernetes_node_taints_round_trip() {
        let taints = json!({
            "node-taints": {
                "key1": ["value1:NoSchedule", "value1:NoExecute"],
            }
        });
        assert_serializes_to::<KubernetesSettingsV1>(taints.clone(), taints);
    }

    #[test]
    fn kubernetes_single_node_taint_normalizes_to_list() {
        assert_serializes_to::<KubernetesSettingsV1>(
            json!({"node-taints": {"key1": "value1:NoSchedule"}}),
            json!({"node-taints": {"key1": ["value1:NoSchedule"]}}),
        );
    }

    #[test]
    fn container_registry_mirrors_round_trip() {
        let mirrors = json!({
            "mirrors": [{
                "registry": "docker.io",
                "endpoint": ["https://example.net/"],
            }]
        });
        assert_serializes_to::<RegistrySettingsV1>(mirrors.clone(), mirrors);
    }

    #[test]
    fn container_registry_mirrors_table_normalizes_to_array() {
        assert_serializes_to::<RegistrySettingsV1>(
            json!({"mirrors": {"docker.io": ["https://example.net/"]}}),
            json!({"mirrors": [{"registry": "docker.io", "endpoint": ["https://example.net/"]}]}),
        );
    }

    #[test]
    fn container_registry_creds_alias_serializes_as_credentials() {
        assert_serializes_to::<RegistrySettingsV1>(
            json!({"creds": [{"registry": "docker.io", "username": "user", "password": "pass"}]}),
            json!({"credentials": [{"registry": "docker.io", "username": "user", "password": "pass"}]}),
        );
    }

    #[test]
    fn container_runtime_chunk_size_round_trips_as_bytes() {
        let chunk_size = json!({"concurrent-download-chunk-size": 10485760});
        assert_serializes_to::<ContainerRuntimeSettingsV1>(chunk_size.clone(), chunk_size);
    }

    #[test]
    fn container_runtime_chunk_size_normalizes_units_to_bytes() {
        assert_serializes_to::<ContainerRuntimeSettingsV1>(
            json!({"concurrent-download-chunk-size": "10MiB"}),
            json!({"concurrent-download-chunk-size": 10485760}),
        );
    }

    #[test]
    fn container_runtime_chunk_size_alias_serializes_canonically() {
        assert_serializes_to::<ContainerRuntimeSettingsV1>(
            json!({"concurrent-layer-fetch-buffer": "10MiB"}),
            json!({"concurrent-download-chunk-size": 10485760}),
        );
    }

    #[test]
    fn oci_defaults_resource_limits_round_trip() {
        let limits = json!({
            "resource-limits": {
                "max-open-files": {"hard-limit": 1024, "soft-limit": 512},
            }
        });
        assert_serializes_to::<OciDefaultsV1>(limits.clone(), limits);
    }
}
