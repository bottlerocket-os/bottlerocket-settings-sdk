//! This module contains data types that can be used in the model when special input/output
//! (ser/de) behavior is desired.  For example, the ValidBase64 type can be used for a model field
//! when we don't even want to accept an API call with invalid base64 data.

// The pattern in this module is to make a struct and implement TryFrom<&str> with code that does
// necessary checks and returns the struct.  Other traits that treat the struct like a string can
// be implemented for you with the string_impls_for macro.

pub mod error {
    use bottlerocket_scalar::ValidationError;
    use regex::Regex;
    use snafu::Snafu;

    // x509_parser::pem::Pem::parse_x509 returns an Err<X509Error>, which is a bit
    // verbose. Declaring a type to simplify it.
    type PEMToX509ParseError = x509_parser::nom::Err<x509_parser::error::X509Error>;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Can't create SingleLineString containing line terminator"))]
        StringContainsLineTerminator,

        #[snafu(display("Invalid base64 input: {}", source))]
        InvalidBase64 { source: base64::DecodeError },

        #[snafu(display("Invalid JSON: {}", source))]
        InvalidJson { source: serde_json::Error },

        #[snafu(display(
            "Identifiers may only contain ASCII alphanumerics plus hyphens, received '{}'",
            input
        ))]
        InvalidIdentifier { input: String },

        #[snafu(display(
            "Kernel boot config keywords may only contain ASCII alphanumerics plus hyphens and underscores, received '{}'",
            input
        ))]
        InvalidBootconfigKey { input: String },

        #[snafu(display(
            "Kernel boot config values may only contain ASCII printable characters, received '{}'",
            input
        ))]
        InvalidBootconfigValue { input: String },

        #[snafu(display(
            "Kernel module keys may only contain ASCII alphanumerics plus hyphens and underscores, received '{}'",
            input
        ))]
        InvalidKmodKey { input: String },

        #[snafu(display("Given invalid URL '{input}': {source}"))]
        InvalidUrl {
            input: String,
            source: url::ParseError,
        },

        #[snafu(display("Invalid OCI image reference '{}': {}", input, msg))]
        InvalidOciImageRef { input: String, msg: String },

        #[snafu(display(
            "Value for '{}' contains a control character (byte {:#04x}) and cannot be stored",
            kind,
            byte
        ))]
        ContainsControlChar { kind: &'static str, byte: u32 },

        #[snafu(display("Invalid version string '{}'", input))]
        InvalidVersion { input: String },

        #[snafu(display("Invalid hugepages size '{}': '{}'", input, msg))]
        InvalidHugepageSize { input: String, msg: String },

        #[snafu(display(
            "Invalid hugepages allocation '{}'. Must be either a non-negative \
            integer or a comma separated list of \"<node>:<pages>\" pairs (e.g. \"0:128,1:256\")",
            input
        ))]
        InvalidHugepageAllocation { input: String },

        #[snafu(display("{} must match '{}', given: {}", thing, pattern, input))]
        Pattern {
            thing: String,
            pattern: Regex,
            input: String,
        },

        // Some regexes are too big to usefully display in an error.
        #[snafu(display("{} given invalid input: {}", thing, input))]
        BigPattern { thing: String, input: String },

        #[snafu(display("Invalid Kubernetes cloud provider '{}'", input))]
        InvalidCloudProvider { input: String },

        #[snafu(display("Invalid Kubernetes authentication mode '{}'", input))]
        InvalidAuthenticationMode { input: String },

        #[snafu(display("Invalid bootstrap mode '{}'", input))]
        InvalidBootstrapMode { input: String },

        #[snafu(display("Given invalid cluster name '{}': {}", name, msg))]
        InvalidClusterName { name: String, msg: String },

        #[snafu(display("Invalid domain name '{}': {}", input, msg))]
        InvalidDomainName { input: String, msg: String },

        #[snafu(display("Invalid hostname '{}': {}", input, msg))]
        InvalidLinuxHostname { input: String, msg: String },

        #[snafu(display("Invalid Linux lockdown mode '{}'", input))]
        InvalidLockdown { input: String },

        #[snafu(display("Invalid Bottlerocket API Command '{:?}'", input))]
        InvalidCommand { input: Vec<String> },

        #[snafu(display("Invalid sysctl key '{}': {}", input, msg))]
        InvalidSysctlKey { input: String, msg: String },

        #[snafu(display("Invalid input for field {}: {}", field, source))]
        InvalidPlainValue {
            field: String,
            source: serde_plain::Error,
        },

        #[snafu(display("Invalid Kubernetes threshold percentage value '{}'", input))]
        InvalidThresholdPercentage { input: String },

        #[snafu(display("Invalid percentage value '{}'", input))]
        InvalidPercentage {
            input: String,
            source: std::num::ParseFloatError,
        },

        #[snafu(display("Invalid Cpu Manager policy '{}'", input))]
        InvalidCpuManagerPolicy {
            input: String,
            source: serde_plain::Error,
        },

        #[snafu(display("Invalid Kubernetes duration value '{}'", input))]
        InvalidKubernetesDurationValue { input: String },

        #[snafu(display("Invalid x509 certificate: {}", source))]
        InvalidX509Certificate { source: PEMToX509ParseError },

        #[snafu(display("Invalid PEM object: {}", source))]
        InvalidPEM {
            source: x509_parser::error::PEMError,
        },

        #[snafu(display("No valid certificate found in bundle"))]
        NoCertificatesFound {},

        #[snafu(display("Invalid topology manager scope '{}'", input))]
        InvalidTopologyManagerScope {
            input: String,
            source: serde_plain::Error,
        },

        #[snafu(display("Invalid topology manager policy '{}'", input))]
        InvalidTopologyManagerPolicy {
            input: String,
            source: serde_plain::Error,
        },

        #[snafu(display("Invalid imageGCHighThresholdPercent '{}': {}", input, msg))]
        InvalidImageGCHighThresholdPercent { input: String, msg: String },

        #[snafu(display("Invalid imageGCLowThresholdPercent '{}': {}", input, msg))]
        InvalidImageGCLowThresholdPercent { input: String, msg: String },

        #[snafu(display("Invalid ECS duration value '{}'", input))]
        InvalidECSDurationValue { input: String },

        #[snafu(display("Could not parse '{}' as an integer", input))]
        ParseInt {
            input: String,
            source: std::num::ParseIntError,
        },

        #[snafu(display("Invalid Kernel CpuSet value '{}'", input))]
        InvalidKernelCpuSetValue { input: String },

        #[snafu(display("Invalid memory swap behavior value '{}'", input))]
        InvalidMemorySwapBehavior { input: String },

        #[snafu(display("Invalid Kubernetes IDs per pod value '{}'", input))]
        InvalidKubernetesIdsPerPodValue { input: i64 },

        #[snafu(display(
            "Invalid Kubernetes max-allowable-numa-nodes value '{}': must be >= 8",
            input
        ))]
        InvalidMaxAllowableNumaNodesValue { input: u32 },
    }

    /// Creates a `ValidationError` with a consistent message for strings with regex validations
    /// where the regex is too big to display to the user.
    pub(crate) fn big_pattern_error<S1, S2>(thing: S1, input: S2) -> ValidationError
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        ValidationError::new(format!(
            "{} given invalid input: {}",
            thing.as_ref(),
            input.as_ref()
        ))
    }
}

/// This is similar to the `Snafu` `ensure` macro that we are familiar with, but it works with our
/// own `ValidationError` instead of a `Snafu` error enum.
macro_rules! require {
    ($condition:expr, $err:expr) => {
        if !($condition) {
            return Err($err);
        }
    };
}

#[cfg(test)]
mod test_reject_control_chars {
    use super::reject_control_chars;

    #[test]
    fn accepts_plain_ascii() {
        assert!(reject_control_chars("hello-world", "X").is_ok());
        assert!(reject_control_chars("", "X").is_ok());
        assert!(reject_control_chars("path with spaces", "X").is_ok());
        assert!(reject_control_chars("unicode 日本語", "X").is_ok());
    }

    #[test]
    fn rejects_every_c0_and_c1_control() {
        for b in (0u8..=0x1F).chain(std::iter::once(0x7Fu8)) {
            let s = format!("prefix{}suffix", b as char);
            let err = reject_control_chars(&s, "X").unwrap_err();
            assert!(
                err.to_string().contains("control character"),
                "byte {b:#04x} must be rejected"
            );
        }
    }

    #[test]
    fn rejects_unicode_line_and_paragraph_separators() {
        for c in ['\u{2028}', '\u{2029}', '\u{0085}'] {
            let s = format!("prefix{c}suffix");
            reject_control_chars(&s, "X").unwrap_err();
        }
    }
}

/// Rejects any input containing an ASCII or Unicode control character, or
/// U+2028 / U+2029. Called at the top of every `TryFrom<&str>` for a
/// single-line modeled type; multi-line types (`PemCertificateString`,
/// `EtcHostsEntries`) skip it.
pub(crate) fn reject_control_chars(input: &str, kind: &'static str) -> Result<(), error::Error> {
    let bad = input
        .chars()
        .find(|c| c.is_control() || matches!(*c, '\u{2028}' | '\u{2029}'));
    if let Some(c) = bad {
        return error::ContainsControlCharSnafu {
            kind,
            byte: c as u32,
        }
        .fail();
    }
    Ok(())
}

// Must be after macro definition
mod ecs;
mod hugepages;
mod kubernetes;
mod oci_defaults;
mod shared;

pub use ecs::*;
pub use hugepages::*;
pub use kubernetes::*;
pub use oci_defaults::*;
pub use shared::*;
