//! The ntp settings can be used to specify time servers with which to synchronize the instance's
//! clock.
use bottlerocket_model_derive::model;
use bottlerocket_modeled_types::{Identifier, Url};
use bottlerocket_settings_sdk::{GenerateResult, LinearlyMigrateable, NoMigration, SettingsModel};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::collections::HashMap;
use std::convert::Infallible;

#[model(impl_default = true)]
pub struct NtpSettingsV1 {
    /// Time servers to sync with: either a plain list of URLs, or a map of named
    /// servers each with their own config. See `NtpTimeServers`.
    time_servers: NtpTimeServers,
    /// Extra chrony options applied to every server, used with the URL-list form.
    /// With the named-server map, each server carries its own options instead.
    options: Vec<String>,
    /// chrony log categories, rendered as a single `log` line
    /// (e.g. `["measurements","statistics"]` -> `log measurements statistics`).
    /// Empty/unset renders no `log` line.
    logging: Vec<String>,
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
        // Anything that parses as time-servers (a URL list or a named map) is ok
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
        // Anything that parses as time-servers (a URL list or a named map) is ok
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

/// A single named time server. Chrony flags (prefer, minpoll, maxpoll, iburst,
/// ...) go in `options` as plain strings. Options render verbatim, so use chrony
/// syntax without "=" (e.g. "minpoll 4", not "minpoll = 4").
#[model(impl_default = true)]
pub struct NtpTimeServer {
    address: Url,
    directive: NtpDirective,
    options: Vec<String>,
}

pub mod error {
    use snafu::Snafu;

    /// Errors from parsing the `time-servers` value into one of the two accepted
    /// shapes (a URL list or a named server map).
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display(
            "time-servers must be a list of URLs or a map of named servers, got {kind}"
        ))]
        WrongType { kind: String },

        #[snafu(display("invalid time-servers URL list: {source}"))]
        InvalidUrlList { source: serde_json::Error },

        #[snafu(display("invalid time-servers named map: {source}"))]
        InvalidNamedMap { source: serde_json::Error },
    }
}

/// The `time-servers` value accepts two forms, kept backwards compatible: a plain
/// list of URLs, or a map of named servers each with their own config.
///
/// Deserialization selects the form from the JSON value type, validates its
/// contents, and reports a type-specific error for unsupported values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Value", into = "serde_json::Value")]
pub enum NtpTimeServers {
    /// A plain list of server URLs.
    Legacy(Vec<Url>),
    /// A map of named servers, each with its own address and options.
    Named(HashMap<Identifier, NtpTimeServer>),
}

impl TryFrom<serde_json::Value> for NtpTimeServers {
    type Error = error::Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            serde_json::Value::Array(_) => {
                let list = serde_json::from_value(value).context(error::InvalidUrlListSnafu)?;
                Ok(NtpTimeServers::Legacy(list))
            }
            serde_json::Value::Object(_) => {
                let map = serde_json::from_value(value).context(error::InvalidNamedMapSnafu)?;
                Ok(NtpTimeServers::Named(map))
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
        // Both variants contain values that can be represented as JSON.
        let result = match servers {
            NtpTimeServers::Legacy(list) => serde_json::to_value(list),
            NtpTimeServers::Named(map) => serde_json::to_value(map),
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
        let test_json = r#"{"time-servers":["https://example.net","http://www.example.com"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            ntp.time_servers.clone().unwrap(),
            NtpTimeServers::Legacy(vec!(
                Url::try_from("https://example.net").unwrap(),
                Url::try_from("http://www.example.com").unwrap(),
            ))
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
    fn test_serde_ntp_v1_named_map() {
        let test_json = r#"{"time-servers":{"link-local":{"address":"169.254.169.123","directive":"server","options":["iburst","prefer","minpoll 4","maxpoll 4"]}},"logging":["tracking"]}"#;

        let ntp: NtpSettingsV1 = serde_json::from_str(test_json).unwrap();
        match ntp.time_servers.clone().unwrap() {
            NtpTimeServers::Named(map) => {
                let server = map
                    .get(&Identifier::try_from("link-local").unwrap())
                    .unwrap();
                assert_eq!(server.directive, Some(NtpDirective::Server));
            }
            NtpTimeServers::Legacy(_) => panic!("named map misparsed as Legacy"),
        }
        assert_eq!(ntp.logging.clone().unwrap(), vec!["tracking".to_string()]);

        let results = serde_json::to_string(&ntp).unwrap();
        assert_eq!(results, test_json);
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
    fn test_tryfrom_parses_named_map() {
        let test_json = r#"{"link-local":{"address":"169.254.169.123","directive":"server","options":["iburst","prefer","minpoll 4","maxpoll 4"]}}"#;
        let parsed: NtpTimeServers = serde_json::from_str(test_json).unwrap();
        match parsed {
            NtpTimeServers::Named(map) => {
                let server = map
                    .get(&Identifier::try_from("link-local").unwrap())
                    .unwrap();
                assert_eq!(
                    server.address,
                    Some(Url::try_from("169.254.169.123").unwrap())
                );
                assert_eq!(server.directive, Some(NtpDirective::Server));
                assert_eq!(
                    server.options,
                    Some(vec![
                        "iburst".to_string(),
                        "prefer".to_string(),
                        "minpoll 4".to_string(),
                        "maxpoll 4".to_string(),
                    ])
                );
            }
            NtpTimeServers::Legacy(_) => {
                panic!("a named map was misparsed as the Legacy list variant")
            }
        }
    }

    #[test]
    fn test_tryfrom_no_cross_contamination() {
        let list: NtpTimeServers = serde_json::from_str(r#"["https://time.aws.com"]"#).unwrap();
        assert!(matches!(list, NtpTimeServers::Legacy(_)));

        let map: NtpTimeServers = serde_json::from_str(
            r#"{"amazon-pool":{"address":"time.aws.com","directive":"pool","options":["iburst"]}}"#,
        )
        .unwrap();
        assert!(matches!(map, NtpTimeServers::Named(_)));
    }

    #[test]
    fn test_tryfrom_rejects_wrong_type_with_clear_error() {
        // Values that are neither lists nor maps return a type-specific error.
        let err = serde_json::from_str::<NtpTimeServers>(r#""just-a-string""#).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("list of URLs or a map of named servers"),
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
}
