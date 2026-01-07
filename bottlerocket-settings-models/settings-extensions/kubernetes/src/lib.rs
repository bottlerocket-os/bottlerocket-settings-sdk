//! Modeled types for creating Kubernetes settings.
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::{
    CpuManagerPolicy, CredentialProvider, DNSDomain, Identifier, IntegerPercent, KernelCpuSetValue,
    KubernetesAuthenticationMode, KubernetesBootstrapToken, KubernetesCPUManagerPolicyOption,
    KubernetesCloudProvider, KubernetesClusterDnsIp, KubernetesClusterName,
    KubernetesDurationValue, KubernetesEvictionKey, KubernetesHostnameOverrideSource,
    KubernetesIdsPerPodValue, KubernetesLabelKey, KubernetesLabelValue,
    KubernetesMemoryManagerPolicy, KubernetesMemoryReservation, KubernetesMemorySwapBehavior,
    KubernetesQuantityValue, KubernetesReservedResourceKey, KubernetesTaintValue,
    KubernetesThresholdValue, NonNegativeInteger, SingleLineString, TopologyManagerPolicy,
    TopologyManagerScope, Url, ValidBase64, ValidLinuxHostname,
};
use bottlerocket_settings_sdk::{GenerateResult, SettingsModel};

use self::de::deserialize_node_taints;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::IpAddr;

mod de;

// Kubernetes static pod manifest settings
#[model]
pub struct StaticPod {
    enabled: bool,
    manifest: ValidBase64,
}

#[model(impl_default = true)]
pub struct KubernetesSettingsV1 {
    // Settings that must be specified via user data or through API requests.  Not all settings are
    // useful for all modes. For example, in standalone mode the user does not need to specify any
    // cluster information, and the bootstrap token is only needed for TLS authentication mode.
    cluster_name: KubernetesClusterName,
    cluster_certificate: ValidBase64,
    api_server: Url,
    node_labels: HashMap<KubernetesLabelKey, KubernetesLabelValue>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_node_taints"
    )]
    node_taints: HashMap<KubernetesLabelKey, Vec<KubernetesTaintValue>>,
    static_pods: HashMap<Identifier, StaticPod>,
    authentication_mode: KubernetesAuthenticationMode,
    bootstrap_token: KubernetesBootstrapToken,
    standalone_mode: bool,
    eviction_hard: HashMap<KubernetesEvictionKey, KubernetesThresholdValue>,
    eviction_soft: HashMap<KubernetesEvictionKey, KubernetesThresholdValue>,
    eviction_soft_grace_period: HashMap<KubernetesEvictionKey, KubernetesDurationValue>,
    eviction_max_pod_grace_period: NonNegativeInteger,
    kube_reserved: HashMap<KubernetesReservedResourceKey, KubernetesQuantityValue>,
    system_reserved: HashMap<KubernetesReservedResourceKey, KubernetesQuantityValue>,
    allowed_unsafe_sysctls: Vec<SingleLineString>,
    server_tls_bootstrap: bool,
    cloud_provider: KubernetesCloudProvider,
    registry_qps: i32,
    registry_burst: i32,
    event_qps: i32,
    event_burst: i32,
    kube_api_qps: i32,
    kube_api_burst: i32,
    container_log_max_size: KubernetesQuantityValue,
    container_log_max_files: i32,
    container_log_max_workers: i32,
    container_log_monitor_interval: KubernetesDurationValue,
    cpu_cfs_quota_enforced: bool,
    cpu_manager_policy: CpuManagerPolicy,
    cpu_manager_reconcile_period: KubernetesDurationValue,
    cpu_manager_policy_options: Vec<KubernetesCPUManagerPolicyOption>,
    topology_manager_scope: TopologyManagerScope,
    topology_manager_policy: TopologyManagerPolicy,
    pod_pids_limit: i64,
    image_gc_high_threshold_percent: IntegerPercent,
    image_gc_low_threshold_percent: IntegerPercent,
    image_minimum_gc_age: KubernetesDurationValue,
    image_maximum_gc_age: KubernetesDurationValue,
    provider_id: Url,
    log_level: u8,
    credential_providers: HashMap<Identifier, CredentialProvider>,
    server_certificate: ValidBase64,
    server_key: ValidBase64,
    shutdown_grace_period: KubernetesDurationValue,
    shutdown_grace_period_for_critical_pods: KubernetesDurationValue,
    memory_manager_reserved_memory: HashMap<Identifier, KubernetesMemoryReservation>,
    memory_manager_policy: KubernetesMemoryManagerPolicy,
    reserved_cpus: KernelCpuSetValue,
    memory_swap_behavior: KubernetesMemorySwapBehavior,
    hostname_override_source: KubernetesHostnameOverrideSource,
    seccomp_default: bool,
    device_ownership_from_security_context: bool,
    single_process_oom_kill: bool,
    static_pods_enabled: bool,
    max_pods: u32,
    cluster_dns_ip: KubernetesClusterDnsIp,
    cluster_domain: DNSDomain,
    node_ip: IpAddr,
    pod_infra_container_image: SingleLineString,
    hostname_override: ValidLinuxHostname,
    ids_per_pod: KubernetesIdsPerPodValue,
    max_parallel_image_pulls: i32,
    #[serde(alias = "fail-cgroupv1", skip_serializing_if = "Option::is_none")]
    fail_cgroup_v1: bool,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for KubernetesSettingsV1 {
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // allow anything that parses as MetricsSettingsV1
        Ok(())
    }

    fn generate(
        existing_partial: Option<Self::PartialKind>,
        _dependent_settings: Option<serde_json::Value>,
    ) -> Result<GenerateResult<Self::PartialKind, Self>> {
        // TODO this should eventually replace `pluto`
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
    use bottlerocket_modeled_types::KubernetesHostnameOverrideSource;

    #[test]
    fn test_generate_kubernetes() {
        let generated = KubernetesSettingsV1::generate(None, None).unwrap();

        assert_eq!(
            generated,
            GenerateResult::Complete(KubernetesSettingsV1 {
                cluster_name: None,
                cluster_certificate: None,
                api_server: None,
                node_labels: None,
                node_taints: None,
                static_pods: None,
                authentication_mode: None,
                bootstrap_token: None,
                standalone_mode: None,
                eviction_hard: None,
                eviction_soft: None,
                eviction_soft_grace_period: None,
                eviction_max_pod_grace_period: None,
                kube_reserved: None,
                system_reserved: None,
                allowed_unsafe_sysctls: None,
                server_tls_bootstrap: None,
                cloud_provider: None,
                registry_qps: None,
                registry_burst: None,
                event_qps: None,
                event_burst: None,
                kube_api_qps: None,
                kube_api_burst: None,
                container_log_max_size: None,
                container_log_max_files: None,
                container_log_max_workers: None,
                container_log_monitor_interval: None,
                cpu_cfs_quota_enforced: None,
                cpu_manager_policy: None,
                cpu_manager_reconcile_period: None,
                cpu_manager_policy_options: None,
                topology_manager_scope: None,
                topology_manager_policy: None,
                pod_pids_limit: None,
                image_gc_high_threshold_percent: None,
                image_gc_low_threshold_percent: None,
                image_maximum_gc_age: None,
                image_minimum_gc_age: None,
                provider_id: None,
                log_level: None,
                credential_providers: None,
                server_certificate: None,
                server_key: None,
                shutdown_grace_period: None,
                shutdown_grace_period_for_critical_pods: None,
                memory_manager_reserved_memory: None,
                memory_manager_policy: None,
                reserved_cpus: None,
                memory_swap_behavior: None,
                max_pods: None,
                cluster_dns_ip: None,
                cluster_domain: None,
                node_ip: None,
                pod_infra_container_image: None,
                hostname_override: None,
                hostname_override_source: None,
                seccomp_default: None,
                device_ownership_from_security_context: None,
                single_process_oom_kill: None,
                static_pods_enabled: None,
                ids_per_pod: None,
                max_parallel_image_pulls: None,
                fail_cgroup_v1: None,
            })
        );
    }


    #[test]
    fn test_fail_cgroup_v1_skip_serializing_if() {
        let k8s = KubernetesSettingsV1 {
            fail_cgroup_v1: None,
            ..Default::default()
        };
        let json = serde_json::to_string(&k8s).unwrap();
        assert!(!json.contains("fail_cgroup_v1"));
        assert!(!json.contains("fail-cgroupv1"));
    }

    #[test]
    fn test_serde_kubernetes() {
        let test_json = r#"{
            "cluster-name": "my-cluster",
            "api-server": "https://example.com",
            "hostname-override-source": "private-dns-name"
        }"#;

        let kubernetes: KubernetesSettingsV1 = serde_json::from_str(test_json).unwrap();

        assert_eq!(
            kubernetes,
            KubernetesSettingsV1 {
                cluster_name: Some("my-cluster".try_into().unwrap()),
                api_server: Some("https://example.com".try_into().unwrap()),
                hostname_override_source: Some(KubernetesHostnameOverrideSource::PrivateDNSName),
                ..Default::default()
            }
        );
    }
}
