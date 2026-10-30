//! Byte quantities for kube-reserved service reclaim protection.
use crate::error;
use crate::iec::parse_iec_quantity;
use bottlerocket_string_impls_for::string_impls_for;
use snafu::ensure;

/// An integer byte quantity with an optional Ki, Mi, Gi or Ti unit. Zero disables protection.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubeReservedMemoryMinValue {
    inner: String,
}

impl KubeReservedMemoryMinValue {
    /// The largest MemoryMin= value systemd accepts in a unit file; it ignores u64::MAX.
    pub const MAX_BYTES: u64 = u64::MAX - 1;

    pub fn as_bytes(&self) -> u64 {
        Self::parse(&self.inner).expect("KubeReservedMemoryMinValue is validated on construction")
    }

    fn parse(input: &str) -> Option<u64> {
        parse_iec_quantity(input)
            .map(|quantity| quantity.bytes)
            .filter(|&bytes| bytes <= Self::MAX_BYTES)
    }
}

impl TryFrom<&str> for KubeReservedMemoryMinValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "KubeReservedMemoryMinValue")?;
        ensure!(
            Self::parse(input).is_some(),
            error::InvalidKubeReservedMemoryMinSnafu { input }
        );
        Ok(Self {
            inner: input.to_owned(),
        })
    }
}

string_impls_for!(KubeReservedMemoryMinValue, "KubeReservedMemoryMinValue");

#[cfg(test)]
mod test_kube_reserved_memory_min_value {
    use super::KubeReservedMemoryMinValue;
    use test_case::test_case;

    #[test_case("0", 0; "zero")]
    #[test_case("0Mi", 0; "zero with unit")]
    #[test_case("1", 1; "one byte")]
    #[test_case("1Ki", 1 << 10; "kibibytes")]
    #[test_case("128Mi", 128 << 20; "mebibytes")]
    #[test_case("2Gi", 2 << 30; "gibibytes")]
    #[test_case("1Ti", 1 << 40; "tebibytes")]
    #[test_case("18446744073709551614", u64::MAX - 1; "largest systemd value")]
    fn valid(input: &str, bytes: u64) {
        let value = KubeReservedMemoryMinValue::try_from(input).unwrap();
        assert_eq!(value.as_bytes(), bytes);
        assert_eq!(serde_json::to_value(&value).unwrap(), input);
        assert_eq!(
            serde_json::from_value::<KubeReservedMemoryMinValue>(serde_json::json!(input)).unwrap(),
            value
        );
    }

    #[test_case(""; "empty")]
    #[test_case("001"; "leading zeros")]
    #[test_case("01Mi"; "leading zero with unit")]
    #[test_case("-1Mi"; "negative")]
    #[test_case("-0"; "negative zero")]
    #[test_case("+1"; "plus sign")]
    #[test_case("1..2Mi"; "two dots")]
    #[test_case("1.5Gi"; "fraction")]
    #[test_case(".5"; "leading dot")]
    #[test_case("1."; "trailing dot")]
    #[test_case("1e3"; "exponent")]
    #[test_case("1E3"; "uppercase exponent")]
    #[test_case("1G"; "SI unit")]
    #[test_case("1ki"; "lowercase unit")]
    #[test_case("1KiB"; "trailing B")]
    #[test_case("16Ei"; "unsupported unit")]
    #[test_case("10%"; "percent")]
    #[test_case("1 Mi"; "space before unit")]
    #[test_case(" 1"; "leading space")]
    #[test_case("1 "; "trailing space")]
    #[test_case("1\n"; "trailing newline")]
    #[test_case("１"; "fullwidth digit")]
    #[test_case("18446744073709551615"; "u64 max")]
    #[test_case("18446744073709551616"; "u64 overflow")]
    #[test_case("16777216Ti"; "unit overflow")]
    fn invalid(input: &str) {
        assert!(KubeReservedMemoryMinValue::try_from(input).is_err());
        assert!(
            serde_json::from_value::<KubeReservedMemoryMinValue>(serde_json::json!(input)).is_err()
        );
    }

    #[test_case(serde_json::json!(128); "number")]
    #[test_case(serde_json::json!(true); "bool")]
    #[test_case(serde_json::Value::Null; "null")]
    fn rejects_non_string_json(value: serde_json::Value) {
        assert!(serde_json::from_value::<KubeReservedMemoryMinValue>(value).is_err());
    }
}
