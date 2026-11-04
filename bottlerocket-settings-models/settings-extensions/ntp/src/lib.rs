//! The ntp settings can be used to specify time servers with which to synchronize the instance's
//! clock.
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::{SingleLineString, Url};
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct NtpSettingsV1 {
    /// Time servers to sync with. See `NtpTimeServers` for the accepted forms.
    time_servers: NtpTimeServers,
    /// Extra chrony options applied to every server, used with the URL-list form.
    /// Server objects carry their own options and do not inherit this setting.
    options: Vec<SingleLineString>,
    /// chrony log categories, rendered as a single `log` line
    /// (e.g. `["measurements","statistics"]` -> `log measurements statistics`).
    /// Empty/unset renders no `log` line.
    logging: Vec<NtpLogComponent>,
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
        // Anything that parses as one of the supported time-server forms is valid.
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
        // Anything that parses as one of the supported time-server forms is valid.
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

/// Whether a time server is a single endpoint (`server`) or a DNS name that may
/// resolve to several servers (`pool`). Renders as the leading chrony directive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum NtpDirective {
    #[default]
    Server,
    Pool,
}

/// A chrony log category, accepted by chrony's `log` directive. Restricting
/// this to the categories chrony knows keeps an unusable value out of
/// `chrony.conf`, because chronyd treats an unknown category as a fatal error
/// and exits at startup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NtpLogComponent {
    /// Raw NTP measurements, written to `measurements.log`.
    RawMeasurements,
    /// Processed NTP measurements, written to `measurements.log`.
    Measurements,
    /// Clock statistics, written to `statistics.log`.
    Statistics,
    /// Source selection, written to `selection.log`.
    Selection,
    /// Clock tracking, written to `tracking.log`.
    Tracking,
    /// Real-time clock changes, written to `rtc.log`.
    Rtc,
    /// Reference clock measurements, written to `refclocks.log`.
    Refclocks,
    /// Temperature compensation, written to `tempcomp.log`.
    Tempcomp,
}

/// A single time server. Chrony flags (prefer, minpoll, maxpoll, iburst,
/// ...) go in `options` as plain strings. Options render verbatim, so use chrony
/// syntax without "=" (e.g. "minpoll 4", not "minpoll = 4").
#[model(impl_default = true)]
pub struct NtpTimeServer {
    address: Url,
    directive: NtpDirective,
    options: Vec<SingleLineString>,
}

pub mod error {
    use snafu::Snafu;

    /// Errors from parsing the accepted `time-servers` representations.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display(
            "time-servers must be a list of URLs or a list of server objects, got {kind}"
        ))]
        WrongType { kind: String },

        #[snafu(display("invalid time-servers URL list: {source}"))]
        InvalidUrlList { source: serde_json::Error },

        #[snafu(display("time-servers list mixes URL strings and server objects"))]
        MixedList,

        #[snafu(display("invalid time-servers object list: {source}"))]
        InvalidObjectList { source: serde_json::Error },

        #[snafu(display("time-servers object at index {index} is missing address"))]
        ObjectMissingAddress { index: usize },
    }
}

/// The `time-servers` value accepts a legacy URL list and a per-server object
/// list, so both forms occupy one datastore leaf.
///
/// Deserialization selects the form from the JSON value type, validates its
/// contents, and reports a type-specific error for unsupported values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Value", into = "serde_json::Value")]
pub enum NtpTimeServers {
    /// A plain list of server URLs.
    Legacy(Vec<Url>),
    /// A list of server objects, each with its own address, directive and options.
    Objects(Vec<NtpTimeServer>),
}

impl TryFrom<serde_json::Value> for NtpTimeServers {
    type Error = error::Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            serde_json::Value::Array(ref items) => {
                let strings = items.iter().filter(|v| v.is_string()).count();
                let objects = items.iter().filter(|v| v.is_object()).count();
                if !items.is_empty() && strings > 0 && objects > 0 {
                    return error::MixedListSnafu.fail();
                }
                if objects > 0 {
                    let list: Vec<NtpTimeServer> =
                        serde_json::from_value(value).context(error::InvalidObjectListSnafu)?;
                    if let Some(index) = list.iter().position(|server| server.address.is_none()) {
                        return error::ObjectMissingAddressSnafu { index }.fail();
                    }
                    Ok(NtpTimeServers::Objects(list))
                } else {
                    let list = serde_json::from_value(value).context(error::InvalidUrlListSnafu)?;
                    Ok(NtpTimeServers::Legacy(list))
                }
            }
            other => error::WrongTypeSnafu {
                kind: json_kind(&other).to_string(),
            }
            .fail(),
        }
    }
}

impl From<NtpTimeServers> for serde_json::Value {
    fn from(servers: NtpTimeServers) -> Self {
        // Every variant contains values that can be represented as JSON.
        let result = match servers {
            NtpTimeServers::Legacy(list) => serde_json::to_value(list),
            NtpTimeServers::Objects(list) => serde_json::to_value(list),
        };
        result.expect("NtpTimeServers always serializes to JSON")
    }
}

/// A human-readable name for a JSON value's type, for error messages.
fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Array(_) => "a list",
        serde_json::Value::Object(_) => "a map",
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
                logging: None,
            }))
        )
    }

    #[test]
    fn test_serde_ntp() {
        // Url stores input verbatim; serialization must round-trip byte-for-byte.
        let test_json = r#"{"time-servers":["https://example.net","http://www.example.com"]}"#;
        let canonical_json = test_json;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.time_servers.clone().unwrap(),
            NtpTimeServers::Legacy(vec!(
                Url::try_from("https://example.net").unwrap(),
                Url::try_from("http://www.example.com").unwrap(),
            ))
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, canonical_json);
    }

    #[test]
    fn test_options_ntp() {
        let test_json = r#"{"time-servers":["https://example.net","http://www.example.com"],"options":["minpoll","1","maxpoll","2"]}"#;
        let canonical_json = test_json;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.options.clone().unwrap(),
            vec!(
                SingleLineString::try_from("minpoll").unwrap(),
                SingleLineString::try_from("1").unwrap(),
                SingleLineString::try_from("maxpoll").unwrap(),
                SingleLineString::try_from("2").unwrap(),
            )
        );

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, canonical_json);
    }

    #[test]
    fn test_serde_ntp_v1_object_list() {
        let test_json = r#"{"time-servers":[{"address":"169.254.169.123","directive":"server","options":["iburst","prefer","minpoll 4","maxpoll 4"]}],"logging":["tracking"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        match ntp.time_servers.clone().unwrap() {
            NtpTimeServers::Objects(list) => {
                let server = list.first().expect("one server");
                assert_eq!(server.directive, Some(NtpDirective::Server));
            }
            _ => panic!("object list misparsed"),
        }
        assert_eq!(
            ntp.logging.clone().unwrap(),
            vec![NtpLogComponent::Tracking]
        );

        assert_eq!(serde_json::to_string(&ntp).unwrap(), test_json);
    }

    #[test]
    fn test_tryfrom_parses_legacy_list() {
        let test_json = r#"["https://time.aws.com","https://time2.aws.com"]"#;
        let parsed: NtpTimeServers = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            parsed,
            NtpTimeServers::Legacy(vec![
                Url::try_from("https://time.aws.com").unwrap(),
                Url::try_from("https://time2.aws.com").unwrap(),
            ])
        );
    }

    #[test]
    fn test_tryfrom_parses_object_entry() {
        let test_json = r#"[{"address":"169.254.169.123","directive":"server","options":["iburst","prefer","minpoll 4","maxpoll 4"]}]"#;
        let parsed: NtpTimeServers = serde_json::from_str(test_json).unwrap();
        match parsed {
            NtpTimeServers::Objects(list) => {
                let server = list.first().expect("one server");
                assert_eq!(
                    server.address,
                    Some(Url::try_from("169.254.169.123").unwrap())
                );
                assert_eq!(server.directive, Some(NtpDirective::Server));
                assert_eq!(
                    server.options,
                    Some(vec![
                        SingleLineString::try_from("iburst").unwrap(),
                        SingleLineString::try_from("prefer").unwrap(),
                        SingleLineString::try_from("minpoll 4").unwrap(),
                        SingleLineString::try_from("maxpoll 4").unwrap(),
                    ])
                );
            }
            _ => panic!("an object list was misparsed"),
        }
    }

    #[test]
    fn test_tryfrom_no_cross_contamination() {
        let list: NtpTimeServers = serde_json::from_str(r#"["https://time.aws.com"]"#).unwrap();
        assert!(matches!(list, NtpTimeServers::Legacy(_)));

        let objects: NtpTimeServers = serde_json::from_str(
            r#"[{"address":"time.aws.com","directive":"pool","options":["iburst"]}]"#,
        )
        .unwrap();
        assert!(matches!(objects, NtpTimeServers::Objects(_)));
    }

    #[test]
    fn test_rejects_named_map() {
        // A map is no longer an accepted input shape.
        let err = serde_json::from_str::<NtpTimeServers>(
            r#"{"link-local":{"address":"169.254.169.123"}}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            err.contains("list of URLs or a list of server objects"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_tryfrom_rejects_wrong_type_with_clear_error() {
        // Values that are not lists return a type-specific error.
        let err = serde_json::from_str::<NtpTimeServers>(r#""just-a-string""#).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("list of URLs or a list of server objects"),
            "expected a clear shape error, got: {msg}"
        );
    }

    #[test]
    fn test_tryfrom_rejects_bad_url_in_list() {
        let result = serde_json::from_str::<NtpTimeServers>(r#"["not a url with spaces"]"#);
        assert!(
            result.is_err(),
            "a malformed URL in the list should be rejected"
        );
    }

    #[test]
    fn test_rejects_control_chars_in_string_inputs() {
        let inputs = [
            r#"{"options":["iburst\nserver example.com"]}"#,
            r#"{"logging":["tracking\nserver example.com"]}"#,
            r#"{"time-servers":{"example":{"address":"example.com","directive":"server","options":["iburst\npool example.com"]}}}"#,
        ];

        for input in inputs {
            assert!(
                serde_json::from_str::<NtpSettingsV1>(input).is_err(),
                "control characters should be rejected: {input}"
            );
        }
    }

    #[test]
    fn test_parses_object_list() {
        let j = r#"[{"address":"169.254.169.123","directive":"server","options":["prefer","iburst","minpoll 4","maxpoll 4"]},{"address":"time.aws.com","directive":"pool","options":["iburst"]}]"#;
        let p: NtpTimeServers = serde_json::from_str(j).unwrap();
        assert!(matches!(p, NtpTimeServers::Objects(ref v) if v.len() == 2));
        assert_eq!(serde_json::to_string(&p).unwrap(), j);
    }

    #[test]
    fn test_empty_list_is_legacy() {
        let p: NtpTimeServers = serde_json::from_str("[]").unwrap();
        assert_eq!(p, NtpTimeServers::Legacy(vec![]));
        assert_eq!(serde_json::to_string(&p).unwrap(), "[]");
    }

    #[test]
    fn test_rejects_mixed_list() {
        let j = r#"["time.aws.com",{"address":"169.254.169.123"}]"#;
        let e = serde_json::from_str::<NtpTimeServers>(j)
            .unwrap_err()
            .to_string();
        assert!(e.contains("mixes"));
    }

    #[test]
    fn test_rejects_bad_object_entries() {
        let e1 = serde_json::from_str::<NtpTimeServers>(r#"[{"address":"a b c"}]"#)
            .unwrap_err()
            .to_string();
        let e2 = serde_json::from_str::<NtpTimeServers>(r#"[{"address":"x","directive":"bogus"}]"#)
            .unwrap_err()
            .to_string();
        let e3 = serde_json::from_str::<NtpTimeServers>(
            r#"[{"address":"x","options":["iburst\nserver evil"]}]"#,
        )
        .unwrap_err()
        .to_string();
        let e4 = serde_json::from_str::<NtpTimeServers>(r#"[1,2]"#)
            .unwrap_err()
            .to_string();
        let e5 = serde_json::from_str::<NtpTimeServers>(r#"[{"directive":"pool"}]"#)
            .unwrap_err()
            .to_string();
        assert!(!e1.is_empty() && !e2.is_empty() && !e3.is_empty() && !e4.is_empty());
        assert!(e5.contains("missing address"));
    }

    #[test]
    fn test_legacy_list_unchanged() {
        let p: NtpTimeServers = serde_json::from_str(r#"["https://time.aws.com"]"#).unwrap();
        assert!(matches!(p, NtpTimeServers::Legacy(_)));
    }

    #[test]
    fn test_settings_with_object_list_and_options() {
        let j = r#"{"time-servers":[{"address":"time.aws.com","directive":"pool","options":["iburst"]}],"options":["iburst"]}"#;
        let n: NtpSettingsV1 = serde_json::from_str(j).unwrap();
        assert!(matches!(n.time_servers, Some(NtpTimeServers::Objects(_))));
    }

    #[test]
    fn user_data_arrays_of_tables_parse_as_object_lists() {
        #[derive(Deserialize)]
        struct UserData {
            settings: Settings,
        }
        #[derive(Deserialize)]
        struct Settings {
            ntp: NtpSettingsV1,
        }
        let input = r#"
            [[settings.ntp.time-servers]]
            address = "169.254.169.123"
            directive = "server"
            options = ["prefer", "iburst", "minpoll 4", "maxpoll 4"]

            [[settings.ntp.time-servers]]
            address = "time.aws.com"
            directive = "pool"
            options = ["iburst"]
        "#;
        let data: UserData = toml::from_str(input).unwrap();
        assert_eq!(
            serde_json::to_value(data.settings.ntp).unwrap(),
            serde_json::json!({"time-servers": [
                {"address": "169.254.169.123", "directive": "server",
                 "options": ["prefer", "iburst", "minpoll 4", "maxpoll 4"]},
                {"address": "time.aws.com", "directive": "pool", "options": ["iburst"]}
            ]})
        );
    }

    #[test]
    fn every_chrony_log_category_is_accepted() {
        let test_json = r#"{"logging":["rawmeasurements","measurements","statistics","selection","tracking","rtc","refclocks","tempcomp"]}"#;
        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.logging.clone().unwrap(),
            vec![
                NtpLogComponent::RawMeasurements,
                NtpLogComponent::Measurements,
                NtpLogComponent::Statistics,
                NtpLogComponent::Selection,
                NtpLogComponent::Tracking,
                NtpLogComponent::Rtc,
                NtpLogComponent::Refclocks,
                NtpLogComponent::Tempcomp,
            ]
        );
        // Round-trips to the exact tokens chrony's `log` directive expects.
        assert_eq!(serde_json::to_string(&ntp).unwrap(), test_json);
    }

    #[test]
    fn unknown_log_category_is_rejected() {
        // chronyd treats an unknown category as a fatal error and exits, so the
        // model rejects it rather than letting it reach chrony.conf.
        for bad in [
            r#"{"logging":["traking"]}"#,
            r#"{"logging":["raw-measurements"]}"#,
            r#"{"logging":["tracking","bogus"]}"#,
            r#"{"logging":["Tracking"]}"#,
            r#"{"logging":[""]}"#,
        ] {
            assert!(
                serde_json::from_str::<NtpSettingsV1>(bad).is_err(),
                "expected {bad} to be rejected"
            );
        }
    }

    #[test]
    fn empty_logging_list_is_accepted() {
        let ntp: NtpSettingsV1 = serde_json::from_str(r#"{"logging":[]}"#).unwrap();
        assert_eq!(ntp.logging.clone().unwrap(), vec![]);
    }
}
