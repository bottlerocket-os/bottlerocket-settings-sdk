/*!
# Settings models

Bottlerocket supports building different variants with their own features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations via
Settings Plugins.

Settings Plugins are composed of Rust structs which, at minimum, provide `Default` and `Deserialize`
implementations.
This package provides settings structs for common settings used in Bottlerocket, which can be added
to Settings Plugins.
The package also exports a set of `modeled_types` which can be helpful in creating additional
settings structures that helpfully validate inputs on deserialize.
*/

mod boot;

// Expose types for creating new settings structs
pub use bottlerocket_model_derive as model_derive;
pub use bottlerocket_modeled_types as modeled_types;
pub use bottlerocket_scalar as scalar;
pub use bottlerocket_scalar_derive as scalar_derive;
pub use bottlerocket_string_impls_for as string_impls_for;

// Expose common settings structs
pub use crate::boot::BootSettingsV1;
pub use settings_extension_autoscaling::{self, AutoScalingSettingsV1};
pub use settings_extension_aws::{self, AwsSettingsV1};
pub use settings_extension_bootstrap_commands::{self, BootstrapCommandsSettingsV1};
pub use settings_extension_bootstrap_containers::{self, BootstrapContainersSettingsV1};
pub use settings_extension_cloudformation::{self, CloudFormationSettingsV1};
pub use settings_extension_container_registry::{self, RegistrySettingsV1};
pub use settings_extension_container_runtime::{self, ContainerRuntimeSettingsV1};
pub use settings_extension_container_runtime_plugins::{self, ContainerRuntimePluginsSettingsV1};
pub use settings_extension_dns::{self, DnsSettingsV1};
pub use settings_extension_ecs::{self, ECSSettingsV1};
pub use settings_extension_host_containers::{self, HostContainersSettingsV1};
pub use settings_extension_image_verifier_plugins::{self, ImageVerifierPluginsSettingsV1};
pub use settings_extension_kernel::{self, KernelSettingsV1, UKIKernelSettingsV1};
pub use settings_extension_kubelet_device_plugins::{self, KubeletDevicePluginsV1};
pub use settings_extension_kubelet_dra_drivers::{self, KubeletDraDriversV1};
pub use settings_extension_kubernetes::{self, KubernetesSettingsV1};
pub use settings_extension_measurement::{self, MeasurementSettingsV1};
pub use settings_extension_metrics::{self, MetricsSettingsV1};
pub use settings_extension_motd::{self, MotdV1};
pub use settings_extension_network::{self, NetworkSettingsV1};
pub use settings_extension_ntp::{self, NtpSettingsV1};
pub use settings_extension_nvidia_container_runtime::{self, NvidiaContainerRuntimeSettingsV1};
pub use settings_extension_oci_defaults::{self, OciDefaultsV1};
pub use settings_extension_oci_hooks::{self, OciHooksSettingsV1};
pub use settings_extension_pki::{self, PkiSettingsV1};
pub use settings_extension_updates::{self, UpdatesSettingsV1};
