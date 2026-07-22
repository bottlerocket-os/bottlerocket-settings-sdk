//! The ntp settings can be used to specify time servers with which to synchronize the instance's
//! clock. They can also be used to point chrony at hardware or software reference clocks (such as
//! a PTP hardware clock) and to grant remote access to the chrony daemon.
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::{SingleLineString, Url};
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct NtpSettingsV1 {
    time_servers: Vec<Url>,
    options: Vec<String>,
    refclocks: Vec<NtpRefClock>,
    allow: Vec<SingleLineString>,
    cmdallow: Vec<SingleLineString>,
    // chrony's `bindcmdaddress` accepts either an IP address or a Unix domain socket path (e.g.
    // `/run/chrony/chronyd.sock`), so this is a single-line string rather than an `IpAddr`.
    bindcmdaddress: Vec<SingleLineString>,
}

/// A hardware or software reference clock that chrony can use as a time source.
///
/// This maps directly to chrony's `refclock` directive:
/// `refclock <driver> <parameter> [<option>...]`
///
/// For example, to use the PTP hardware clock exposed by the Amazon ENA driver, the following
/// chrony configuration:
///
/// `refclock PHC /dev/ptp_ena poll 0 delay 0.000010 prefer`
///
/// is expressed as:
///
/// ```text
/// [[settings.ntp.refclocks]]
/// driver = "PHC"
/// parameter = "/dev/ptp_ena"
/// options = ["poll", "0", "delay", "0.000010", "prefer"]
/// ```
//
// This is intentionally not a `#[model]` struct: because refclocks are stored as a single list
// value in the datastore (rather than exploded into individual keys), the fields can be required
// rather than optional, letting deserialization reject a refclock that is missing its driver or
// parameter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct NtpRefClock {
    /// The refclock driver name, e.g. `PHC`.
    pub driver: SingleLineString,
    /// The driver-specific parameter, e.g. the device path `/dev/ptp_ena`.
    pub parameter: SingleLineString,
    /// Additional refclock options, e.g. `["poll", "0", "delay", "0.000010", "prefer"]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<SingleLineString>,
}

type Result<T> = std::result::Result<T, Infallible>;

impl SettingsModel for NtpSettingsV1 {
    /// the `model` macro makes every field of the `NtpSettingsV1` struct an `Option`, so we can use
    /// the type as its own `PartialKind`.
    type PartialKind = Self;
    type ErrorKind = Infallible;

    fn get_version() -> &'static str {
        "v1"
    }

    fn set(_current_value: Option<Self>, _target: Self) -> Result<()> {
        // Anything that parses as a list of URLs is ok
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
        // Anything that parses as a list of URLs is ok
        Ok(())
    }
}

impl LinearlyMigrateable for NtpSettingsV1 {
    type ForwardMigrationTarget = NoMigration;
    type BackwardMigrationTarget = NoMigration;

    fn migrate_forward(&self) -> Result<Self::ForwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }

    fn migrate_backward(&self) -> Result<Self::BackwardMigrationTarget> {
        NoMigration::no_defined_migration()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_ntp_settings() {
        assert_eq!(
            NtpSettingsV1::generate(None, None),
            Ok(GenerateResult::Complete(NtpSettingsV1 {
                time_servers: None,
                options: None,
                refclocks: None,
                allow: None,
                cmdallow: None,
                bindcmdaddress: None,
            }))
        )
    }

    #[test]
    fn test_serde_ntp() {
        let test_json = r#"{"time-servers":["https://example.net","http://www.example.com"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.time_servers.clone().unwrap(),
            vec!(
                Url::try_from("https://example.net").unwrap(),
                Url::try_from("http://www.example.com").unwrap(),
            )
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_options_ntp() {
        let test_json = r#"{"time-servers":["https://example.net","http://www.example.com"],"options":["minpoll","1","maxpoll","2"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.options.clone().unwrap(),
            vec!("minpoll", "1", "maxpoll", "2",)
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_refclocks_ntp() {
        // A refclock matching the AWS PTP hardware clock example:
        // `refclock PHC /dev/ptp_ena poll 0 delay 0.000010 prefer`
        let test_json = r#"{"refclocks":[{"driver":"PHC","parameter":"/dev/ptp_ena","options":["poll","0","delay","0.000010","prefer"]}]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.refclocks.clone().unwrap(),
            vec![NtpRefClock {
                driver: SingleLineString::try_from("PHC").unwrap(),
                parameter: SingleLineString::try_from("/dev/ptp_ena").unwrap(),
                options: vec![
                    SingleLineString::try_from("poll").unwrap(),
                    SingleLineString::try_from("0").unwrap(),
                    SingleLineString::try_from("delay").unwrap(),
                    SingleLineString::try_from("0.000010").unwrap(),
                    SingleLineString::try_from("prefer").unwrap(),
                ],
            }]
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_refclock_options_default_empty() {
        // `options` is optional and defaults to an empty list when omitted.
        let test_json = r#"{"refclocks":[{"driver":"PHC","parameter":"/dev/ptp0"}]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.refclocks.clone().unwrap(),
            vec![NtpRefClock {
                driver: SingleLineString::try_from("PHC").unwrap(),
                parameter: SingleLineString::try_from("/dev/ptp0").unwrap(),
                options: Vec::new(),
            }]
        );

        // An empty options list should not be serialized back out.
        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_refclock_requires_driver_and_parameter() {
        // A refclock without a driver or parameter is rejected during deserialization.
        assert!(
            serde_json::from_str::<NtpSettingsV1>(r#"{"refclocks":[{"driver":"PHC"}]}"#).is_err()
        );
        assert!(serde_json::from_str::<NtpSettingsV1>(
            r#"{"refclocks":[{"parameter":"/dev/ptp0"}]}"#
        )
        .is_err());
    }

    #[test]
    fn test_daemon_access_ntp() {
        // The chrony daemon access directives from the feature request:
        // `allow`, `cmdallow`, and `bindcmdaddress`. chrony's `bindcmdaddress` accepts both IP
        // addresses and a Unix domain socket path, so both forms are exercised here.
        let test_json = r#"{"allow":["192.0.2.0/24"],"cmdallow":["198.51.100.0/24"],"bindcmdaddress":["10.0.0.5","/run/chrony/chronyd.sock"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.allow.clone().unwrap(),
            vec![SingleLineString::try_from("192.0.2.0/24").unwrap()]
        );
        assert_eq!(
            ntp.cmdallow.clone().unwrap(),
            vec![SingleLineString::try_from("198.51.100.0/24").unwrap()]
        );
        assert_eq!(
            ntp.bindcmdaddress.clone().unwrap(),
            vec![
                SingleLineString::try_from("10.0.0.5").unwrap(),
                SingleLineString::try_from("/run/chrony/chronyd.sock").unwrap(),
            ]
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_daemon_access_rejects_line_terminator() {
        // Values are rendered directly into chrony.conf, so a line terminator (which could inject
        // additional directives) must be rejected.
        assert!(serde_json::from_str::<NtpSettingsV1>(
            "{\"bindcmdaddress\":[\"10.0.0.5\nallow all\"]}"
        )
        .is_err());
        assert!(
            serde_json::from_str::<NtpSettingsV1>("{\"allow\":[\"192.0.2.0/24\nfoo\"]}").is_err()
        );
    }
}
