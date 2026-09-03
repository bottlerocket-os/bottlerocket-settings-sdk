use super::error;
use base64::Engine;
use bottlerocket_scalar::traits::{Scalar, Validate};
use bottlerocket_scalar::ValidationError;
use bottlerocket_scalar_derive::Scalar;
use bottlerocket_string_impls_for::string_impls_for;
use byte_unit::Byte;
use lazy_static::lazy_static;
use regex::Regex;
use semver::Version;
use serde::{de::IntoDeserializer, Deserialize, Deserializer, Serialize};
use snafu::{ensure, ResultExt};
use std::convert::TryFrom;
use std::net::IpAddr;
use std::str::FromStr;
use url::Host;

/// ValidBase64 can only be created by deserializing from valid base64 text.  It stores the
/// original text, not the decoded form.  Its purpose is input validation, namely being used as a
/// field in a model structure so that you don't even accept a request with a field that has
/// invalid base64.
// Note: we use the default base64::STANDARD config which uses/allows "=" padding.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidBase64 {
    inner: String,
}

/// Validate base64 format before we accept the input.
impl TryFrom<&str> for ValidBase64 {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "ValidBase64")?;
        base64::engine::general_purpose::STANDARD
            .decode(input)
            .context(error::InvalidBase64Snafu)?;
        Ok(ValidBase64 {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidBase64, "ValidBase64");

#[cfg(test)]
mod test_valid_base64 {
    use super::ValidBase64;
    use base64::Engine;
    use std::convert::TryFrom;

    #[test]
    fn valid_base64() {
        let v = ValidBase64::try_from("aGk=").unwrap();
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(v.as_ref())
            .unwrap();
        let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
        assert_eq!(decoded, "hi");
    }

    #[test]
    fn invalid_base64() {
        assert!(ValidBase64::try_from("invalid base64").is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ValidBase64Json can only be created by deserializing from valid base64 text that decodes to
/// valid JSON. It stores the original base64 text, not the decoded form. Its purpose is input
/// validation for base64-encoded JSON configuration.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidBase64Json {
    inner: String,
}

impl TryFrom<&str> for ValidBase64Json {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "ValidBase64Json")?;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(input)
            .context(error::InvalidBase64Snafu)?;
        serde_json::from_slice::<serde_json::Value>(&decoded).context(error::InvalidJsonSnafu)?;
        Ok(ValidBase64Json {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidBase64Json, "ValidBase64Json");

#[cfg(test)]
mod test_valid_base64_json {
    use super::ValidBase64Json;
    use std::convert::TryFrom;

    #[test]
    fn valid_base64_json() {
        let v = ValidBase64Json::try_from("eyJ0ZXN0IjoidmFsdWUifQ==").unwrap();
        assert_eq!(v.as_ref(), "eyJ0ZXN0IjoidmFsdWUifQ==");
        assert!(ValidBase64Json::try_from("e30=").is_ok());
    }

    #[test]
    fn invalid_base64() {
        assert!(ValidBase64Json::try_from("not base64!").is_err());
    }

    #[test]
    fn valid_base64_invalid_json() {
        assert!(ValidBase64Json::try_from("bm90IGpzb24=").is_err());
    }

    #[test]
    fn empty_string() {
        assert!(ValidBase64Json::try_from("").is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// SingleLineString can only be created by deserializing from a string that contains at most one
/// line.  It stores the original form and makes it accessible through standard traits.  Its
/// purpose is input validation, for example in cases where you want to accept input for a
/// configuration file and want to ensure a user can't create a new line with extra configuration.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SingleLineString {
    inner: String,
}

impl TryFrom<&str> for SingleLineString {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "SingleLineString")?;
        // Rust does not treat all Unicode line terminators as starting a new line, so we check for
        // specific characters here, rather than just counting from lines().
        // https://en.wikipedia.org/wiki/Newline#Unicode
        let line_terminators = [
            '\n',       // newline (0A)
            '\r',       // carriage return (0D)
            '\u{000B}', // vertical tab
            '\u{000C}', // form feed
            '\u{0085}', // next line
            '\u{2028}', // line separator
            '\u{2029}', // paragraph separator
        ];

        ensure!(
            !input.contains(&line_terminators[..]),
            error::StringContainsLineTerminatorSnafu
        );

        Ok(Self {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(SingleLineString, "SingleLineString");

#[cfg(test)]
mod test_single_line_string {
    use super::SingleLineString;
    use std::convert::TryFrom;

    #[test]
    fn valid_single_line_string() {
        assert!(SingleLineString::try_from("").is_ok());
        assert!(SingleLineString::try_from("hi").is_ok());
        let long_string = " ".repeat(9999);
        assert!(SingleLineString::try_from(long_string).is_ok());
    }

    #[test]
    fn invalid_single_line_string() {
        assert!(SingleLineString::try_from("Hello\nWorld").is_err());

        assert!(SingleLineString::try_from("\n").is_err());
        assert!(SingleLineString::try_from("\r").is_err());
        assert!(SingleLineString::try_from("\r\n").is_err());

        assert!(SingleLineString::try_from("\u{000B}").is_err()); // vertical tab
        assert!(SingleLineString::try_from("\u{000C}").is_err()); // form feed
        assert!(SingleLineString::try_from("\u{0085}").is_err()); // next line
        assert!(SingleLineString::try_from("\u{2028}").is_err()); // line separator
        assert!(SingleLineString::try_from("\u{2029}").is_err());
        // paragraph separator
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ValidLinuxHostname represents a string that contains a valid Linux hostname as defined by
/// https://man7.org/linux/man-pages/man7/hostname.7.html.  It stores the original form and makes
/// it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ValidLinuxHostname {
    inner: String,
}

lazy_static! {
    // According to the man page above, hostnames must be between 1-253 characters long consisting
    // of characters [0-9a-z.-].
    pub(crate) static ref VALID_LINUX_HOSTNAME: Regex = Regex::new(r"^[0-9a-z.-]{1,253}$").unwrap();
}

impl TryFrom<&str> for ValidLinuxHostname {
    type Error = error::Error;

    #[allow(clippy::len_zero)]
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "ValidLinuxHostname")?;
        ensure!(
            VALID_LINUX_HOSTNAME.is_match(input),
            error::InvalidLinuxHostnameSnafu {
                input,
                msg: "must only be [0-9a-z.-], and 1-253 chars long"
            }
        );

        // Though the man page doesn't explicitly disallow hostnames that start with dots, dots are
        // used as separators so starting with a separator would imply an empty domain, which isn't
        // allowed (must be at least one character).
        ensure!(
            !input.starts_with('-') && !input.starts_with('.'),
            error::InvalidLinuxHostnameSnafu {
                input,
                msg: "must not start with '-' or '.'"
            }
        );

        // Each segment must be from 1-63 chars long and shouldn't start with "-"

        ensure!(
            input
                .split('.')
                .all(|x| x.len() >= 1 && x.len() <= 63 && !x.starts_with('-')),
            error::InvalidLinuxHostnameSnafu {
                input,
                msg: "segment is less than 1 or greater than 63 chars"
            }
        );

        Ok(Self {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(ValidLinuxHostname, "ValidLinuxHostname");

#[cfg(test)]
mod test_valid_linux_hostname {
    use super::ValidLinuxHostname;
    use std::convert::TryFrom;

    #[test]
    fn valid_linux_hostname() {
        assert!(ValidLinuxHostname::try_from("hello").is_ok());
        assert!(ValidLinuxHostname::try_from("hello1234567890").is_ok());

        let segment_limit = "a".repeat(63);
        assert!(ValidLinuxHostname::try_from(segment_limit.clone()).is_ok());

        let segment = "a".repeat(61);
        let long_name = format!(
            "{}.{}.{}.{}",
            &segment_limit, &segment_limit, &segment_limit, &segment
        );
        assert!(ValidLinuxHostname::try_from(long_name).is_ok());
    }

    #[test]
    fn invalid_linux_hostname() {
        assert!(ValidLinuxHostname::try_from(" ").is_err());
        assert!(ValidLinuxHostname::try_from("-a").is_err());
        assert!(ValidLinuxHostname::try_from(".a").is_err());
        assert!(ValidLinuxHostname::try_from("@a").is_err());
        assert!(ValidLinuxHostname::try_from("a..a").is_err());
        assert!(ValidLinuxHostname::try_from("a.a.-a.a1234").is_err());

        let long_segment = "a".repeat(64);
        assert!(ValidLinuxHostname::try_from(long_segment.clone()).is_err());

        let long_name = format!(
            "{}.{}.{}.{}",
            &long_segment, &long_segment, &long_segment, &long_segment
        );
        assert!(ValidLinuxHostname::try_from(long_name).is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// EtcHostsEntries represents a mapping of IP Address to hostname aliases that can apply to those
/// addresses.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EtcHostsEntries(
    // Ordering matters in /etc/hosts, and this setting directly maps to that file and its behavior in glibc.
    // Repeated IP Addresses have their host aliases merged to a single line, respecting the order as they appear in this list.
    Vec<(IpAddr, Vec<ValidLinuxHostname>)>,
);

impl EtcHostsEntries {
    pub fn iter_merged(&self) -> impl Iterator<Item = (IpAddr, Vec<ValidLinuxHostname>)> {
        let mut merged: indexmap::IndexMap<IpAddr, Vec<ValidLinuxHostname>> =
            indexmap::IndexMap::with_capacity(self.0.len());

        for (ip_address, aliases) in &self.0 {
            merged
                .entry(*ip_address)
                .or_default()
                .append(&mut (aliases.clone()));
        }

        merged.into_iter()
    }
}

#[cfg(test)]
mod test_etc_hosts_entries {
    use super::{EtcHostsEntries, ValidLinuxHostname};
    use std::net::IpAddr;

    #[test]
    fn test_valid_etc_hosts_entries() {
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["127.0.0.1", ["localhost", "localhost4"]],
            ["::1", ["localhost", "localhost6"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["127.0.0.1", ["localhost", "localhost4"]],
            ["::1", ["localhost", "localhost6"]],
            ["127.0.0.1", ["test.example.com"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[
            ["::1", ["localhost", "localhost6"]],
            ["0000:0000:0000:0000:0000:0000:0000:0001", ["test6.example.com"]]
        ]"#
        )
        .is_ok());
        assert!(serde_json::from_str::<EtcHostsEntries>(r#"[]"#).is_ok());
    }

    #[test]
    fn test_invalid_etc_hosts_entries() {
        assert!(
            serde_json::from_str::<EtcHostsEntries>(r#"[["127.0.0.0/24", ["localhost"]]"#).is_err()
        );
        assert!(
            serde_json::from_str::<EtcHostsEntries>(r#"[["not_an_ip", ["localhost"]]"#).is_err()
        );
        assert!(serde_json::from_str::<EtcHostsEntries>(
            r#"[["127.0.0.1", ["not_a_valid_hostname!"]]"#
        )
        .is_err());
    }

    #[test]
    fn test_iter_merged() {
        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["127.0.0.1", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]]
                ]"#,
            )
            .unwrap()
            .0
        );

        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["127.0.0.3", ["test.example.com"]],
                    ["127.0.0.2", ["test.example.com"]],
                    ["127.0.0.1", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]],
                    ["127.0.0.3", ["test.example.com"]],
                    ["127.0.0.2", ["test.example.com"]]
                ]"#,
            )
            .unwrap()
            .0
        );

        assert_eq!(
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4"]],
                    ["::1", ["localhost", "localhost6"]],
                    ["127.0.0.1", ["test.example.com"]],
                    ["0000:0000:0000:0000:0000:0000:0000:0001", ["test6.example.com"]],
                    ["10.0.0.1", ["example.bottlerocket.aws"]]
                ]"#,
            )
            .unwrap()
            .iter_merged()
            .collect::<Vec<(IpAddr, Vec<ValidLinuxHostname>)>>(),
            serde_json::from_str::<EtcHostsEntries>(
                r#"[
                    ["127.0.0.1", ["localhost", "localhost4", "test.example.com"]],
                    ["::1", ["localhost", "localhost6", "test6.example.com"]],
                    ["10.0.0.1", ["example.bottlerocket.aws"]]
                ]"#,
            )
            .unwrap()
            .0
        );
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Identifier can only be created by deserializing from a string that contains
/// ASCII alphanumeric characters, plus hyphens, which we use as our standard word separator
/// character in user-facing identifiers. It stores the original form and makes it accessible
/// through standard traits. Its purpose is to validate input for identifiers like container names
/// that might be used to create files/directories.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Identifier {
    inner: String,
}

const CONTAINERD_ID_LENGTH: usize = 76;

impl TryFrom<&str> for Identifier {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "Identifier")?;
        let valid_identifier = input
            .chars()
            .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-')
            && input.len() <= CONTAINERD_ID_LENGTH;
        ensure!(valid_identifier, error::InvalidIdentifierSnafu { input });
        Ok(Identifier {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Identifier, "Identifier");

#[cfg(test)]
mod test_valid_identifier {
    use super::{Identifier, CONTAINERD_ID_LENGTH};
    use std::convert::TryFrom;

    #[test]
    fn valid_identifier() {
        assert!(Identifier::try_from("hello-world").is_ok());
        assert!(Identifier::try_from("helloworld").is_ok());
        assert!(Identifier::try_from("123321hello").is_ok());
        assert!(Identifier::try_from("hello-1234").is_ok());
        assert!(Identifier::try_from("--------").is_ok());
        assert!(Identifier::try_from("11111111").is_ok());
        assert!(Identifier::try_from(vec!["X"; CONTAINERD_ID_LENGTH].join("")).is_ok());
    }

    #[test]
    fn invalid_identifier() {
        assert!(Identifier::try_from("../").is_err());
        assert!(Identifier::try_from("{}").is_err());
        assert!(Identifier::try_from("hello|World").is_err());
        assert!(Identifier::try_from("hello\nWorld").is_err());
        assert!(Identifier::try_from("hello_world").is_err());
        assert!(Identifier::try_from("タール").is_err());
        assert!(Identifier::try_from("💝").is_err());
        assert!(Identifier::try_from(vec!["X"; CONTAINERD_ID_LENGTH + 1].join("")).is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Url represents a string that contains a valid URL, according to url::Url, though it also
/// allows URLs without a scheme (e.g. without "http://") because it's common.  It stores the
/// original string and makes it accessible through standard traits. Its purpose is to validate
/// input for any field containing a network address.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Url {
    inner: String,
}

impl TryFrom<&str> for Url {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        // `url::Url::parse` silently strips ASCII tab/CR/LF from its input
        // copy, so a raw newline would validate; reject control chars first.
        crate::reject_control_chars(input, "Url")?;

        let first_err = match input.parse::<url::Url>() {
            Ok(parsed) => {
                return Ok(Url {
                    inner: parsed.to_string(),
                });
            }
            Err(e) => e,
        };
        // Scheme-less inputs like `example.com` are common; retry with `http://`.
        let prefixed = format!("http://{input}");
        if let Ok(parsed) = prefixed.parse::<url::Url>() {
            return Ok(Url {
                inner: parsed.to_string(),
            });
        }
        Err(first_err).context(error::InvalidUrlSnafu { input })
    }
}

string_impls_for!(Url, "Url");

#[cfg(test)]
mod test_url {
    use super::Url;
    use std::convert::TryFrom;

    #[test]
    fn good_urls() {
        for ok in &[
            "https://example.com/path",
            "https://example.com",
            "example.com/path",
            "example.com",
            "ntp://127.0.0.1/path",
            "ntp://127.0.0.1",
            "127.0.0.1/path",
            "127.0.0.1",
            "http://localhost/path",
            "http://localhost",
            "localhost/path",
            "localhost",
            "localhost:8080",
            ".internal",
            ".cluster.local",
        ] {
            Url::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_urls() {
        for err in &["how are you", "weird@", "not a url"] {
            Url::try_from(*err).unwrap_err();
        }
    }

    #[test]
    fn canonicalizes_scheme_present() {
        let cases = [
            ("  https://example.com/path  ", "https://example.com/path"),
            ("https://example.com", "https://example.com/"),
        ];
        for (input, want) in cases {
            let got =
                Url::try_from(input).unwrap_or_else(|e| panic!("expected {input:?} to parse: {e}"));
            let stored: &str = got.as_ref();
            assert_eq!(
                stored, want,
                "canonicalization mismatch for input {input:?}"
            );
            assert!(
                !stored.contains(|c: char| c.is_control() || c == ' '),
                "control/whitespace survived canonicalization: {stored:?}"
            );
        }
    }

    #[test]
    fn rejects_control_characters() {
        let payloads = [
            "unix:///run/containerd/containerd.sock\nother",
            "https://example.com/path\n",
            "https://example.com/path\r\n",
            "https://example.com/path\t",
            "https://example.com/path\x00",
            "https://example.com/path\x1b",
            "https://example.com/path\x7f",
        ];
        for payload in payloads {
            let err = Url::try_from(payload).unwrap_err();
            let msg = err.to_string();
            assert!(
                msg.contains("control character"),
                "expected control-char rejection for {payload:?}, got: {msg}"
            );
        }
    }

    #[test]
    fn canonicalizes_scheme_less() {
        let cases = [
            ("example.com", "http://example.com/"),
            ("example.com/path", "http://example.com/path"),
            (".internal", "http://.internal/"),
        ];
        for (input, want) in cases {
            let got =
                Url::try_from(input).unwrap_or_else(|e| panic!("expected {input:?} to parse: {e}"));
            let stored: &str = got.as_ref();
            assert_eq!(
                stored, want,
                "canonicalization mismatch for scheme-less input {input:?}"
            );
        }
    }

    #[test]
    fn deserialize_rejects_control_characters() {
        let json = r#""https://mirror.example.com/path\n""#;
        let err = serde_json::from_str::<Url>(json).unwrap_err();
        assert!(
            err.to_string().contains("control character"),
            "expected control-char rejection through serde, got: {err}"
        );
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// A container image reference (e.g. `public.ecr.aws/eks/foo:1.0.0`).
///
/// Unlike [`Url`], this type does not parse or rewrite the input; the
/// stored value round-trips byte-for-byte. Rejects empty strings,
/// whitespace, and non-ASCII / control characters. Full OCI-spec
/// validation is left to the runtime that pulls the image.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OciImageRef {
    inner: String,
}

impl TryFrom<&str> for OciImageRef {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "OciImageRef")?;
        if input.is_empty() {
            return error::InvalidOciImageRefSnafu {
                input,
                msg: "must not be empty",
            }
            .fail();
        }
        if let Some(bad) = input.chars().find(|c| c.is_whitespace() || !c.is_ascii()) {
            return error::InvalidOciImageRefSnafu {
                input,
                msg: format!("contains disallowed character {bad:?}"),
            }
            .fail();
        }
        Ok(OciImageRef {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(OciImageRef, "OciImageRef");

#[cfg(test)]
mod test_oci_image_ref {
    use super::OciImageRef;
    use std::convert::TryFrom;

    #[test]
    fn accepts_common_refs() {
        for ok in &[
            "public.ecr.aws/eks/eks-anywhere/foo:1.0.0",
            "docker.io/library/nginx:latest",
            "uri.to.container.in.oci-compatible-registry.example.com/foo:1.0.0",
            "public.ecr.aws/example/example",
            "registry.local:5000/team/img@sha256:abc123",
            "localhost/simple",
        ] {
            let r = OciImageRef::try_from(*ok).unwrap();
            let stored: &str = r.as_ref();
            assert_eq!(stored, *ok, "OCI ref must round-trip byte-for-byte");
        }
    }

    #[test]
    fn rejects_bad_refs() {
        for bad in &[
            "",
            "has space/name:tag",
            "line\nbreak/name",
            "tab\there/name",
            "unicode/日本語",
        ] {
            OciImageRef::try_from(*bad).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// FriendlyVersion represents a version string that can optionally be prefixed with 'v'.
/// It can also be set to 'latest' to represent the latest version. It stores the original string
/// and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FriendlyVersion {
    inner: String,
}

impl TryFrom<&str> for FriendlyVersion {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "FriendlyVersion")?;
        if input == "latest" {
            return Ok(FriendlyVersion {
                inner: input.to_string(),
            });
        }
        // If the string begins with a 'v', skip it before checking if it is valid semver.
        let version = if let Some(stripped) = input.strip_prefix('v') {
            stripped
        } else {
            input
        };

        if version.parse::<semver::Version>().is_ok() {
            return Ok(FriendlyVersion {
                inner: input.to_string(),
            });
        }
        error::InvalidVersionSnafu { input }.fail()
    }
}

impl TryFrom<FriendlyVersion> for semver::Version {
    type Error = semver::Error;

    fn try_from(input: FriendlyVersion) -> Result<semver::Version, Self::Error> {
        // If the string begins with a 'v', skip it before conversion
        let version = if input.inner.starts_with('v') {
            input.inner.get(1..).unwrap_or_default()
        } else {
            &input.inner
        };
        Version::from_str(version)
    }
}

string_impls_for!(FriendlyVersion, "FriendlyVersion");

#[cfg(test)]
mod test_version {
    use super::FriendlyVersion;
    use semver::Version;
    use std::convert::TryFrom;
    use std::convert::TryInto;

    #[test]
    fn good_version_strings() {
        for ok in &[
            "1.0.0",
            "v1.0.0",
            "1.0.1-alpha",
            "v1.0.1-alpha",
            "1.0.2-alpha+1.0",
            "v1.0.2-alpha+1.0",
            "1.1.0-rc.1.1",
            "v1.1.0-rc.1.1",
            "latest",
        ] {
            FriendlyVersion::try_from(*ok).unwrap();
            // Test conversion to semver::Version
            if *ok != "latest" {
                let _: Version = FriendlyVersion {
                    inner: ok.to_string(),
                }
                .try_into()
                .unwrap();
            }
        }
    }

    #[test]
    fn bad_version_string() {
        for err in &[
            "hi",
            "1.0",
            "1",
            "v",
            "v1",
            "v1.0",
            "vv1.1.0",
            "1.0.3-beta.1.01",
            "v1.0.3-beta.1.01",
        ] {
            FriendlyVersion::try_from(*err).unwrap_err();
            let res: Result<Version, semver::Error> = Version::try_from(FriendlyVersion {
                inner: err.to_string(),
            });
            res.unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// DNSDomain represents a string that is a valid DNS domain. It stores the
/// original string and makes it accessible through standard traits. Its purpose
/// is input validation, for example validating the kubelet's "clusterDomain"
/// config value.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DNSDomain {
    inner: String,
}

impl TryFrom<&str> for DNSDomain {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "DNSDomain")?;
        ensure!(
            !input.starts_with('.'),
            error::InvalidDomainNameSnafu {
                input,
                msg: "must not start with '.'",
            }
        );

        let host = Host::parse(input).or_else(|e| {
            error::InvalidDomainNameSnafu {
                input,
                msg: e.to_string(),
            }
            .fail()
        })?;
        match host {
            Host::Ipv4(_) | Host::Ipv6(_) => error::InvalidDomainNameSnafu {
                input,
                msg: "IP address is not a valid domain name",
            }
            .fail(),
            Host::Domain(_) => Ok(Self {
                inner: input.to_string(),
            }),
        }
    }
}

string_impls_for!(DNSDomain, "DNSDomain");

#[cfg(test)]
mod test_dns_domain {
    use super::DNSDomain;
    use std::convert::TryFrom;

    #[test]
    fn valid_dns_domain() {
        for ok in &["cluster.local", "dev.eks", "stage.eks", "prod.eks"] {
            assert!(DNSDomain::try_from(*ok).is_ok());
        }
    }

    #[test]
    fn invalid_dns_domain() {
        for err in &[
            "foo/com",
            ".a",
            "123.123.123.123",
            "[2001:db8::ff00:42:8329]",
        ] {
            assert!(DNSDomain::try_from(*err).is_err());
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// SysctlKey represents a string that is a valid Linux sysctl key; keys must be representable as
/// filesystem paths, and are generally kept to lowercase_underscored_names separated with '.' or
/// '/'.  SysctlKey stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SysctlKey {
    inner: String,
}

lazy_static! {
    /// Pattern matching the name of a sysctl key.  Must be representable as a path; we'll go a bit
    /// further and enforce a basic pattern that would match all known keys, plus some leeway.
    pub(crate) static ref SYSCTL_KEY: Regex = Regex::new(r"^[a-zA-Z0-9./_-]{1,128}$").unwrap();
}

impl TryFrom<&str> for SysctlKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "SysctlKey")?;
        // Basic directory traversal checks; corndog also checks
        ensure!(
            !input.contains(".."),
            error::InvalidSysctlKeySnafu {
                input,
                msg: "must not contain '..'".to_string(),
            }
        );
        ensure!(
            !input.starts_with('.') && !input.starts_with('/'),
            error::InvalidSysctlKeySnafu {
                input,
                msg: "must not start with '.' or '/'".to_string(),
            }
        );
        ensure!(
            SYSCTL_KEY.is_match(input),
            error::InvalidSysctlKeySnafu {
                input,
                msg: format!("must match pattern {}", *SYSCTL_KEY),
            }
        );
        Ok(SysctlKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(SysctlKey, "SysctlKey");

#[cfg(test)]
mod test_sysctl_key {
    use super::SysctlKey;
    use std::convert::TryFrom;

    #[test]
    fn valid_sysctl_key() {
        for ok in &[
            // Longest real one
            "net/ipv4/conf/enp0s42f3/igmpv3_unsolicited_report_interval",
            // Dots or slashes OK
            "net.ipv4.conf.enp0s42f3.igmpv3_unsolicited_report_interval",
            // Mixed dots/slashes isn't supported by sysctl(8), but it's unambiguous
            "net/ipv4.conf.enp0s42f3/igmpv3_unsolicited_report_interval",
            // Shortest real one
            "fs/aio-nr",
            // Shortest allowed
            "a",
            // Longest allowed
            &"a".repeat(128),
            // All allowed characters
            "-./0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ] {
            SysctlKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn invalid_sysctl_key() {
        for err in &[
            // Too long
            &"a".repeat(129),
            // Too short,
            "",
            // Sneaky sneaky
            "hi/../../there",
            "../hi",
            "/../hi",
            // Invalid characters
            "!",
            "@",
            "#",
            "$",
            "%",
            "^",
            "&",
            "*",
            "(",
            ")",
            "\"",
            "'",
            "\\",
            "|",
            "~",
            "`",
        ] {
            SysctlKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// BootConfigKey represents a string that is a valid Kernel boot config key; each key word must
/// contain only alphabets, numbers, dash (-) or underscore (_).
/// BootConfigKey stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootConfigKey {
    inner: String,
}

impl TryFrom<&str> for BootConfigKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "BootConfigKey")?;
        // Each individual keyword must be valid
        let valid_key = input.split('.').all(|keyword| {
            !keyword.is_empty()
                && keyword
                    .chars()
                    .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-' || c == '_')
        });
        ensure!(valid_key, error::InvalidBootconfigKeySnafu { input });
        Ok(BootConfigKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(BootConfigKey, "BootConfigKey");

#[cfg(test)]
mod test_bootconfig_key {
    use super::BootConfigKey;
    use std::convert::TryFrom;

    #[test]
    fn valid_bootconfig_key() {
        for ok in &[
            "keyword1.keyword2",
            "-keyword1.keyword2",
            "_keyword.1.2.3",
            "key_word",
            "key-word",
            "keyword1",
            "keyword1-",
            "keyword2_",
        ] {
            BootConfigKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn invalid_bootconfig_key() {
        for err in &[
            "", "①", ".", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "\"", "'", "\\", "|",
            "~", "`",
        ] {
            BootConfigKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// BootConfigValue represents a string that is a valid Kernel boot config value; each value only
/// contains printable characters or spaces except for delimiters such as semicolon, newline, comma,
/// hash, and closing brace. These delimiters are only usable if the value itself is quoted with
/// single-quotes or double-quotes. Here we treat the value as if they're always quoted in the context
/// of Bottlerocket settings. This means the value just has to be printable ASCII.
/// BootConfigValue stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootConfigValue {
    inner: String,
}

impl TryFrom<&str> for BootConfigValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "BootConfigValue")?;
        ensure!(
            input.chars().all(|c| c.is_ascii() && !c.is_ascii_control())
            // Values containing both single quotes and double quotes are inherently invalid since quotes
            // cannot be escaped.
                && !(input.contains('"') && input.contains('\'')),
            error::InvalidBootconfigValueSnafu { input }
        );
        Ok(BootConfigValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(BootConfigValue, "BootConfigValue");

#[cfg(test)]
mod test_bootconfig_value {
    use super::BootConfigValue;
    use std::convert::TryFrom;

    #[test]
    fn valid_bootconfig_value() {
        for ok in &[
            "plain",
            "@yogurt@",
            "\"abc",
            " !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}",
            "1",
            "value1",
            "hello.goodbye",
            "",
        ] {
            BootConfigValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn invalid_bootconfig_value() {
        for err in &["'\"", "bottlerocket①", "💝", "Ï", "—"] {
            BootConfigValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Lockdown represents a string that is a valid Linux kernel lockdown mode name.  It stores the
/// original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Lockdown {
    inner: String,
}

impl TryFrom<&str> for Lockdown {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "Lockdown")?;
        ensure!(
            matches!(input, "none" | "integrity" | "confidentiality"),
            error::InvalidLockdownSnafu { input }
        );
        Ok(Lockdown {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(Lockdown, "Lockdown");

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// ApiclientCommand represents a valid Bootstrap Command. It stores the command as a vector of
/// strings and ensures that the first argument is apiclient.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, Serialize)]
pub struct ApiclientCommand(Vec<String>);

impl ApiclientCommand {
    pub fn get_command_and_args(&self) -> (&str, &[String]) {
        self.0
            .split_first()
            .map(|(command, rest)| (command.as_str(), rest))
            .unwrap_or_default()
    }
}

// Custom deserializer added to enforce rules to make sure the command is valid.
impl<'de> serde::Deserialize<'de> for ApiclientCommand {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let original: Vec<String> = serde::Deserialize::deserialize(deserializer)?;
        Self::try_from(original).map_err(|e| {
            <D::Error as serde::de::Error>::custom(format!(
                "Unable to deserialize into ApiclientCommand: {e}"
            ))
        })
    }
}

impl TryFrom<Vec<String>> for ApiclientCommand {
    type Error = error::Error;

    fn try_from(input: Vec<String>) -> std::result::Result<Self, error::Error> {
        let first_word = input.first().map(String::as_str);
        ensure!(
            matches!(first_word, Some("apiclient")),
            error::InvalidCommandSnafu { input },
        );

        Ok(ApiclientCommand(input))
    }
}

#[cfg(test)]
mod test_valid_apiclient_command {
    use super::ApiclientCommand;
    use std::convert::TryFrom;

    #[test]
    fn valid_apiclient_command() {
        assert!(ApiclientCommand::try_from(vec![
            "apiclient".to_string(),
            "set".to_string(),
            "motd=helloworld".to_string()
        ])
        .is_ok());
    }

    #[test]
    fn empty_apiclient_command() {
        assert!(ApiclientCommand::try_from(Vec::new()).is_err());
    }

    #[test]
    fn invalid_apiclient_command() {
        assert!(ApiclientCommand::try_from(vec![
            "/usr/bin/touch".to_string(),
            "helloworld".to_string()
        ])
        .is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BootstrapMode {
    inner: String,
}

impl TryFrom<&str> for BootstrapMode {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        crate::reject_control_chars(input, "BootstrapMode")?;
        ensure!(
            matches!(input, "off" | "once" | "always"),
            error::InvalidBootstrapModeSnafu { input }
        );
        Ok(BootstrapMode {
            inner: input.to_string(),
        })
    }
}

impl Default for BootstrapMode {
    fn default() -> Self {
        BootstrapMode {
            inner: "off".to_string(),
        }
    }
}

string_impls_for!(BootstrapMode, "BootstrapMode");

#[cfg(test)]
mod test_valid_container_mode {
    use super::BootstrapMode;
    use std::convert::TryFrom;

    #[test]
    fn valid_container_mode() {
        for ok in &["off", "once", "always"] {
            assert!(BootstrapMode::try_from(*ok).is_ok());
        }
    }

    #[test]
    fn invalid_container_mode() {
        assert!(BootstrapMode::try_from("invalid").is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PemCertificateString {
    inner: String,
}

impl TryFrom<&str> for PemCertificateString {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        // Empty strings are valid to allow deleting bundles
        if input.trim().is_empty() {
            return Ok(PemCertificateString {
                inner: input.to_string(),
            });
        }
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(input)
            .context(error::InvalidBase64Snafu)?;
        // Flag to check if the bundle doesn't contain any valid certificate
        let mut certs_found = false;
        // Validate each certificate in the bundle
        for pem in x509_parser::pem::Pem::iter_from_buffer(&decoded_bytes) {
            // Parse buffer into a PEM object, then to a x509 certificate
            let pem = pem.context(error::InvalidPEMSnafu)?;
            pem.parse_x509()
                .context(error::InvalidX509CertificateSnafu)?;
            certs_found = true;
        }

        // No valid certificate found
        if !certs_found {
            return error::NoCertificatesFoundSnafu {}.fail();
        }

        Ok(PemCertificateString {
            inner: input.to_string(),
        })
    }
}

impl Default for PemCertificateString {
    fn default() -> Self {
        PemCertificateString {
            inner: "".to_string(),
        }
    }
}

string_impls_for!(PemCertificateString, "PemCertificateString");

#[cfg(test)]
mod test_valid_pem_certificate_string {
    use super::PemCertificateString;
    use std::convert::TryFrom;

    static TEST_PEM: &str = include_str!("../../tests/data/test-pem");
    static TEST_INCOMPLETE_PEM: &str = include_str!("../../tests/data/test-incomplete-pem");

    #[test]
    fn valid_pem_certificate() {
        assert!(PemCertificateString::try_from(TEST_PEM).is_ok());
        assert!(PemCertificateString::try_from("").is_ok());
    }

    #[test]
    fn invalid_pem_certificate() {
        // PEM with valid markers but with invalid content
        assert!(PemCertificateString::try_from(
            "LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tIGJhZCAtLS0tLUVORCBDRVJUSUZJQ0FURS0tLS0tCg=="
        )
        .is_err());
        // PEM with valid content but without footer marker
        assert!(PemCertificateString::try_from(TEST_INCOMPLETE_PEM).is_err());

        // PEM without any valid certificate
        assert!(PemCertificateString::try_from(
            "77yc44Kz77ya44OfIOOBj+OCszrlvaEg77yc44Kz77ya44OfIOOBj+OCszrlvaEg77yc44Kz77ya44OfCg=="
        )
        .is_err())
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KmodKey can only be created by deserializing from a string that contains ASCII
/// alphanumeric characters, plus hyphens, plus underscores. It stores the original
/// form and makes it accessible through standard traits. Its purpose is to validate
/// input that will be treated as a potential kernel module name.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KmodKey {
    inner: String,
}

// This limit is based on the kernel definition, and assumes a 64-bit host.
//   include/linux/module.h
//     #define MODULE_NAME_LEN MAX_PARAM_PREFIX_LEN
//   include/linux/moduleparam.h
//     #define MAX_PARAM_PREFIX_LEN (64 - sizeof(unsigned long))
const KMOD_KEY_LENGTH: usize = 56;

impl TryFrom<&str> for KmodKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "KmodKey")?;
        // The kernel allows modules to have any name that's a valid filename,
        // but real module names seem to be limited to this character set.
        let valid_key = input
            .chars()
            .all(|c| (c.is_ascii() && c.is_alphanumeric()) || c == '-' || c == '_')
            && input.len() <= KMOD_KEY_LENGTH;
        ensure!(valid_key, error::InvalidKmodKeySnafu { input });
        Ok(KmodKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KmodKey, "KmodKey");

#[cfg(test)]
mod test_valid_kmod_key {
    use super::{KmodKey, KMOD_KEY_LENGTH};
    use std::convert::TryFrom;

    #[test]
    fn valid_kmod_key() {
        assert!(KmodKey::try_from("kmod").is_ok());
        assert!(KmodKey::try_from("i8042").is_ok());
        assert!(KmodKey::try_from("xt_XT").is_ok());
        assert!(KmodKey::try_from("dm-block").is_ok());
        assert!(KmodKey::try_from("blowfish-x86_64").is_ok());
        assert!(KmodKey::try_from(vec!["a"; KMOD_KEY_LENGTH].join("")).is_ok());
    }

    #[test]
    fn invalid_kmod_key() {
        assert!(KmodKey::try_from("../").is_err());
        assert!(KmodKey::try_from("{}").is_err());
        assert!(KmodKey::try_from("kernel|Module").is_err());
        assert!(KmodKey::try_from("kernel\nModule").is_err());
        assert!(KmodKey::try_from("🐡").is_err());
        assert!(KmodKey::try_from(vec!["z"; KMOD_KEY_LENGTH + 1].join("")).is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Input value that needs to be a positive value, but should not be greater
/// than an i32::MAX.
#[derive(Clone, Debug, PartialEq, Scalar)]
pub struct NonNegativeInteger {
    inner: i32,
}

impl Validate for NonNegativeInteger {
    fn validate<I: Into<i32>>(input: I) -> Result<NonNegativeInteger, ValidationError> {
        let inner: i32 = input.into();
        if inner < 0 {
            Err(ValidationError::new(
                "number must be great than or equal to 0",
            ))
        } else {
            Ok(Self { inner })
        }
    }
}

#[cfg(test)]
mod test_positive_integer {
    use super::NonNegativeInteger;
    use std::convert::TryFrom;

    #[test]
    fn valid_positive_integer() {
        assert!(NonNegativeInteger::try_from(0).is_ok());
        assert!(NonNegativeInteger::try_from(i32::MAX).is_ok());
        assert!(NonNegativeInteger::try_from(42).is_ok());
    }

    #[test]
    fn invalid_positive_integer() {
        assert!(NonNegativeInteger::try_from(i32::MIN).is_err());
        assert!(NonNegativeInteger::try_from(-1).is_err());
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KernelCpuSetValue represents a string that contains a valid Kernel CpuSet Value from
/// here: https://man7.org/linux/man-pages/man7/cpuset.7.html#FORMATS. This matches the
/// logic from https://github.com/kubernetes/utils/blob/d93618cff8a22d3aea7bf78d9d528fd859720c2d/cpuset/cpuset.go#L203
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KernelCpuSetValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KERNAL_CPU_SET_VALUE: Regex =
        Regex::new(r"^([0-9]+(-[0-9]+)?,?)*([0-9]+(-[0-9]+)?)+$").unwrap();
}

impl TryFrom<&str> for KernelCpuSetValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "KernelCpuSetValue")?;
        ensure!(
            !input.is_empty(),
            error::InvalidKernelCpuSetValueSnafu { input }
        );
        ensure!(
            KERNAL_CPU_SET_VALUE.is_match(input),
            error::InvalidKernelCpuSetValueSnafu { input }
        );
        Ok(KernelCpuSetValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KernelCpuSetValue, "KernelCpuSetValue");

#[cfg(test)]
mod test_kernel_cpu_set_value {
    use super::KernelCpuSetValue;
    use std::convert::TryFrom;

    #[test]
    fn good_tokens() {
        for ok in &[
            "1",
            "0,1,2,3",
            "1-4",
            "1,6-9",
            "1-4,6",
            "1-3,6-9,10-15",
            "100,101",
            "1,2,3,4,5,6,7,8,9,10",
        ] {
            KernelCpuSetValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &[
            "",
            "100.0",
            "0...4",
            "1..5",
            "ten",
            "1-",
            "-3",
            "9,-10",
            "1,2,3,4,5,6,7,8,9,10,",
            &"a".repeat(23),
        ] {
            KernelCpuSetValue::try_from(*err).unwrap_err();
        }
    }
}
// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Deserializes a size string (e.g., "64mb") or integer into an i64 value representing bytes.
pub fn deserialize_chunk_size<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }

    let value = StringOrInt::deserialize(deserializer)?;

    match value {
        StringOrInt::String(s) => {
            let s = s.trim();

            // Try parsing as a plain integer (bytes)
            if let Ok(bytes) = s.parse::<i64>() {
                if bytes < 0 {
                    return Err(serde::de::Error::custom(format!(
                        "Size cannot be negative: {s}"
                    )));
                }
                return Ok(bytes);
            }

            if s.contains(char::is_whitespace) {
                return Err(serde::de::Error::custom(format!(
                    "Size string cannot contain whitespace: '{s}'"
                )));
            }

            // Handle the special case of ".5gb" format (byte_unit expects "0.5gb")
            let normalized_s = if s.starts_with('.') {
                format!("0{s}")
            } else {
                s.to_string()
            };

            // Try parsing as a size string with units using byte_unit, ignoring case.
            match Byte::parse_str(&normalized_s, true) {
                Ok(byte) => {
                    // Convert to i64, ensuring we don't overflow
                    let bytes = byte.as_u64();
                    i64::try_from(bytes).map_err(|e| {
                        serde::de::Error::custom(format!("Unable to convert bytes u64 to i64: {e}"))
                    })
                }
                Err(e) => Err(serde::de::Error::custom(format!(
                    "Invalid size string format '{s}': {e}"
                ))),
            }
        }
        StringOrInt::Int(i) => {
            if i < 0 {
                return Err(serde::de::Error::custom(format!(
                    "Size cannot be negative: {i}"
                )));
            }
            Ok(i)
        }
    }
}
/// Deserializes an optional size string or integer into Option<i64> for model fields
pub fn deserialize_optional_chunk_size<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None => Ok(None),
        Some(v) => deserialize_chunk_size(v.into_deserializer())
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}
