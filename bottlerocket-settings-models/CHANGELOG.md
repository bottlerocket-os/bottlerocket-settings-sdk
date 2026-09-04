# `bottlerocket-settings-models` Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## Model Changes

### Added

- Added `RegistryMirrorCapabilityV1` enum (`pull`, `resolve`, `push`) and optional `capabilities` field to `RegistryMirrorV1`, allowing per-mirror capability configuration for containerd registry mirrors

[unreleased changes here]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.27.0...HEAD

## [0.27.0] - 2026-09-02

### Added

- Added `memory-reservation-policy` and `memory-throttling-factor` Kubernetes settings for configuring kubelet MemoryQoS policy ([#147])

[#147]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/147

[0.27.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.26.0...bottlerocket-settings-models-v0.27.0

## [0.26.0] - 2026-08-12

### Changed

- Added support for a reduced set of kernel settings called `UKIKernelSettingsV1` ([#144])

[#144]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/144

[0.26.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.25.0...bottlerocket-settings-models-v0.26.0

## [0.25.0] - 2026-07-15

### Changed

- Added support for `hugepages.static` and `hugepages.transparent` in `settings.kernel` ([#139])
- Added `settings.measurement` to exclude settings from PCR8 measurement ([#140])
- Added `container-runtime-endpoint` kubernetes settings ([#141]) - Thanks @shvbsle!

[#139]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/139
[#140]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/140
[#141]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/141

[0.25.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.24.0...bottlerocket-settings-models-v0.25.0

## [0.24.0] - 2026-06-09

### Changed

- Add `enabled` field to `NvidiaDevicePluginSettings` to allow disabling the NVIDIA k8s device plugin via API ([#135])

[#135]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/135

[0.24.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.23.0...bottlerocket-settings-models-v0.24.0

## [0.23.0] - 2026-04-06

### Changed

- Reverted `toml` dependency from 1.1 back to 0.8
- Reverted `snafu` dependency from 0.9 back to 0.8

[0.23.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.22.0...bottlerocket-settings-models-v0.23.0

## [0.22.0] - 2026-04-02

## Model Changes

### Added

- Add support for  `prefer-closest-numa-nodes` and `max-allowable-numa-nodes` in Kubernetes' Topology Manager Policy options ([#117])
- Added `container-runtime.max-concurrent-unpacks` setting for containerd 2.2 parallel unpack support ([#119])

[#117]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/117
[#119]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/119

[0.22.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.21.0...bottlerocket-settings-models-v0.22.0

## [0.21.0] - 2026-02-24

## Model Changes

### Added

- Added support for image-verifier plugins ([#114])

[#114]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/114

[0.21.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.20.0...bottlerocket-settings-models-v0.21.0

## [0.20.0] - 2026-01-14

## Model Changes

### Added

- Added MPS GPU sharing settings model with `NvidiaMpsSettings` struct and `Mps` variant to `NvidiaDeviceSharingStrategy` ([#107])

### Removed

- Reverted `fail-cgroup-v1` kubernetes setting ([#111])

[#107]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/107
[#111]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/111

[0.20.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.19.0...bottlerocket-settings-models-v0.20.0

## [0.19.0] - 2026-01-07

## Model Changes

### Fixed

- Fixed `fail-cgroup-v1` kubernetes setting serialization to skip when None ([#110])

[#110]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/110

[0.19.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.18.0...bottlerocket-settings-models-v0.19.0

## [0.18.0] - 2026-01-05

## Model Changes

### Added

- Added `fail-cgroup-v1` kubernetes setting ([#108])

### Changed

- Updated regex for `NvidiaGpuModel` validation ([#105])

[#105]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/105
[#108]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/108

[0.18.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.17.0...bottlerocket-settings-models-v0.18.0

## [0.17.0] - 2025-11-05

## Model Changes

### Added

- Added `image-minimum-gc-age` and `image-maximum-gc-age` kubernetes settings ([#87]) - Thanks @parnniti!
- Added `ids-per-pod` and `max-parallel-image-pulls` kubernetes settings ([#104])
- Added beta options for `cpu-manager-policy-options` kubernetes settings ([#104])
- Added `image-verifier-plugins` settings extension with initial support for notation trustpolicy document ([#106])

[#87]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/87
[#104]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/104
[#106]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/106

[0.17.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.16.0...bottlerocket-settings-models-v0.17.0

## [0.16.0] - 2025-09-19

## Model Changes

### Added

- Added `pid` setting to the `kube-reserved` and `system-reserved` kubernetes settings ([#98])

[#98]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/98

[0.16.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.15.0...bottlerocket-settings-models-v0.16.0

## [0.15.0] - 2025-09-11

## Model Changes

### Fixed

- Fixed `concurrent-download-chunk-size` in `container-runtime` being optional during deserialization ([#102])

[#102]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/102

[0.15.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.14.0...bottlerocket-settings-models-v0.15.0

## [0.14.0] - 2025-09-04

## Model Changes

### Added

- Added `concurrent-download-chunk-size` setting to `container-runtime` setting extension ([#99])
- Added `command` setting to `host-containers` and `bootstrap-containers` setting extensions ([#100]) - Thanks @kasimeka!

[#99]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/99
[#100]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/100

[0.14.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.13.0...bottlerocket-settings-models-v0.14.0

## [0.13.0] - 2025-08-25

## Model Changes

### Added

- Added `static_pods_enabled` settings extension for kubernetes pods ([#93])

[#93]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/93
[0.13.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.12.0...bottlerocket-settings-models-v0.13.0

## [0.12.0] - 2025-07-23

## Model Changes

### Added

- Added `container-runtime-plugins` settings extension with SOCI snapshotter configuration support ([#91])
- Added `snapshotter` setting to `container-runtime` settings extension ([#91])

[#91]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/91
[0.12.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.11.0...bottlerocket-settings-models-v0.12.0

## [0.11.0] - 2025-06-13

## Model Changes

### Added

- Added `memory-swap-behavior` to kubernetes settings model ([#88])

[#88]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/88

[0.11.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.10.0...bottlerocket-settings-models-v0.11.0

## [0.10.0] - 2025-05-19

## Model Changes

### Added

- Modified `nvidia-device-list-strategy` to accept either a list or a string and added `cdi-cri` as an accepted value ([#83])

[#83]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/83

[0.10.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.10.0...bottlerocket-settings-models-v0.9.0

## [0.9.0] - 2025-05-02

## Model Changes

### Added

- Added `container-log-max-workers` and `container-log-monitor-interval` to kubernetes settings model ([#80])
- Added `single-process-oom-kill` to kubernetes settings model ([#81])

[#80]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/80
[#81]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/81

[0.9.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.9.0...bottlerocket-settings-models-v0.8.0

## [0.8.0] - 2025-02-05

## Model Changes

### Added

- Added NVIDIA MIG to kubernetes device plugins settings extension ([#63])

[#63]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/63

[0.8.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.8.0...bottlerocket-settings-models-v0.7.0

## [0.7.0] - 2024-12-24

## Model Changes

### Added

- Add kubernetes device ownership settings ([#69])

### Changed

- Align kubernetes cluster name validation with EKS ([#64]) Thanks @cartermckinnon

[#64]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/64
[#69]:https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/69

[0.7.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.7.0...bottlerocket-settings-models-v0.6.0

## [0.6.0] - 2024-10-02

## Model Changes

### Added

- Added nvidia time-slicing to kubernetes device plugins settings extension ([#62])

[#62]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/62

[0.6.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.6.0...bottlerocket-settings-models-v0.5.0

## [0.5.0] - 2024-09-10

## Model Changes

### Added

- Added kubernetes device plugins settings extension ([#60])

### Changed

- Drop `nvidia-device-plugin` cargo feature ([#60])

[#60]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/60

[0.5.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.5.0...bottlerocket-settings-models-v0.4.0

## [0.4.0] - 2024-09-04

## Model Changes

### Added

- Added the bootstrap-commands settings extension and related shared models ([#46])

### Changed

- Changed `bottlerocket-modeled-types::BootstrapContainerMode` to `bottlerocket-modeled-types::BootstrapMode` ([#46])

[#46]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/46

[0.4.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.4.0...bottlerocket-settings-models-v0.3.0

## [0.3.0] - 2024-08-14

## Model Changes

### Added

- Added the nvidia-container-runtime settings extension ([#43])
- Added optional nvidia device-plugins settings to kubernetes model ([#43])

### Changed

- Skipped serializing credential provider fields if they are None ([#51])
- Moved kubernetes models to a kubernetes settings extension ([#53])
- Updated dependencies ([#50], [#47])

[#43]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/43
[#47]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/47
[#50]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/50
[#51]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/51
[#53]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/53

[0.3.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.3.0...bottlerocket-settings-models-v0.2.0

## [0.2.0] - 2024-07-29

### Changed

- Added `hostname_override_source` to kubernetes settings model ([#42])

[#42]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/pull/42

[0.2.0]: https://github.com/bottlerocket-os/bottlerocket-settings-sdk/compare/bottlerocket-settings-models-v0.2.0...bottlerocket-settings-models-v0.1.0

