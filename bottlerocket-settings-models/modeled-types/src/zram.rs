use crate::error;
use crate::iec::{parse_decimal, parse_iec_quantity};
use bottlerocket_string_impls_for::string_impls_for;
use serde::{Deserialize, Serialize};
use snafu::ensure;

/// Only variants that include zram swap apply these settings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZramSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ZramSize>,
}

/// A byte quantity with an optional Ki, Mi, Gi or Ti unit (e.g. `512Mi`), or an
/// integer percentage from `0%` to `100%`. Zero disables zram swap.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ZramSize {
    inner: String,
}

/// A swap area needs 10 pages: 640 KiB with 64 KiB pages, rounded up to 1 MiB.
const MIN_BYTES: u64 = 1 << 20;

enum Size {
    Bytes(u64),
    Percent(u8),
}

impl ZramSize {
    /// Percentages resolve against `total_bytes`, rounding down.
    pub fn as_bytes(&self, total_bytes: u64) -> u64 {
        match Self::parse(&self.inner).expect("ZramSize is validated on construction") {
            Size::Bytes(bytes) => bytes,
            Size::Percent(percent) => {
                u64::try_from(u128::from(total_bytes) * u128::from(percent) / 100)
                    .expect("a percentage of at most 100 is at most total_bytes")
            }
        }
    }

    fn parse(input: &str) -> Option<Size> {
        if let Some(number) = input.strip_suffix('%') {
            let percent = u8::try_from(parse_decimal(number)?)
                .ok()
                .filter(|percent| *percent <= 100)?;
            return Some(Size::Percent(percent));
        }
        let bytes = parse_iec_quantity(input)?.bytes;
        (bytes == 0 || bytes >= MIN_BYTES).then_some(Size::Bytes(bytes))
    }
}

impl TryFrom<&str> for ZramSize {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "ZramSize")?;
        ensure!(
            Self::parse(input).is_some(),
            error::InvalidZramSizeSnafu { input }
        );
        Ok(Self {
            inner: input.to_owned(),
        })
    }
}
string_impls_for!(ZramSize, "ZramSize");

#[cfg(test)]
mod test_zram_size {
    use super::*;
    use test_case::test_case;

    #[test_case("0", 0; "zero")]
    #[test_case("0Ki", 0; "zero kibibytes")]
    #[test_case("0Mi", 0; "zero mebibytes")]
    #[test_case("0Gi", 0; "zero gibibytes")]
    #[test_case("0Ti", 0; "zero tebibytes")]
    #[test_case("512Mi", 512 << 20; "mebibytes")]
    #[test_case("2Gi", 2 << 30; "gibibytes")]
    #[test_case("1Ti", 1 << 40; "tebibytes")]
    #[test_case("1Mi", 1 << 20; "minimum")]
    #[test_case("1024Ki", 1 << 20; "minimum in kibibytes")]
    #[test_case("1024Gi", 1 << 40; "tebibyte in gibibytes")]
    #[test_case("1048576", 1 << 20; "bare bytes")]
    #[test_case("18446744073709551615", u64::MAX; "u64 max")]
    fn absolute_sizes_and_round_trip(input: &str, bytes: u64) {
        let size = ZramSize::try_from(input).unwrap();
        assert_eq!(size.as_bytes(123), bytes);
        let json = serde_json::to_string(&size).unwrap();
        assert_eq!(json, format!("\"{input}\""));
        assert_eq!(serde_json::from_str::<ZramSize>(&json).unwrap(), size);
    }

    #[test_case("0%", 4099, 0; "zero")]
    #[test_case("25%", 4099, 1024; "rounds down")]
    #[test_case("100%", 4099, 4099; "all")]
    #[test_case("100%", u64::MAX, u64::MAX; "no overflow")]
    #[test_case("1%", 100, 1; "no minimum")]
    fn percentages(input: &str, total_bytes: u64, bytes: u64) {
        let size = ZramSize::try_from(input).unwrap();
        assert_eq!(size.as_bytes(total_bytes), bytes);
    }

    #[test_case("1"; "one byte")]
    #[test_case("655360"; "640 KiB")]
    #[test_case("1048575"; "just below 1 MiB")]
    #[test_case("1Ki"; "one kibibyte")]
    #[test_case("1023Ki"; "just below 1 MiB in kibibytes")]
    #[test_case("1Pi"; "pebibytes")]
    #[test_case("1Ei"; "exbibytes")]
    #[test_case("0Pi"; "zero pebibytes")]
    #[test_case("2000k"; "SI k")]
    #[test_case("128M"; "SI M")]
    #[test_case("0M"; "zero SI M")]
    #[test_case("1G"; "SI G")]
    #[test_case("1T"; "SI T")]
    #[test_case("1P"; "SI P")]
    #[test_case("1E"; "SI E")]
    fn rejects_absolute_sizes_below_one_mib_and_unsupported_suffixes(input: &str) {
        assert!(ZramSize::try_from(input).is_err());
    }

    #[test_case(""; "empty")]
    #[test_case("-1"; "negative")]
    #[test_case("+1Gi"; "plus sign")]
    #[test_case("1.5Gi"; "fraction")]
    #[test_case("1e9"; "exponent")]
    #[test_case("100m"; "lowercase SI m")]
    #[test_case(" 1Gi"; "leading space")]
    #[test_case("1Gi "; "trailing space")]
    #[test_case("1Gi\n"; "trailing newline")]
    #[test_case("101%"; "over 100 percent")]
    #[test_case("1.5%"; "fractional percent")]
    #[test_case("%"; "bare percent")]
    #[test_case("NaN"; "nan")]
    #[test_case("1Gib"; "trailing b")]
    #[test_case("1GB"; "decimal GB")]
    #[test_case("1mi"; "lowercase unit")]
    #[test_case("18446744073709551616"; "u64 overflow")]
    #[test_case("16Ei"; "exbibyte overflow")]
    #[test_case("18446744073709551615Gi"; "gibibyte overflow")]
    #[test_case("16777216Ti"; "tebibyte overflow")]
    #[test_case("00"; "double zero")]
    #[test_case("01Gi"; "leading zero")]
    #[test_case("0512Mi"; "leading zero mebibytes")]
    #[test_case("05%"; "leading zero percent")]
    #[test_case("00%"; "double zero percent")]
    #[test_case("1Gi%"; "unit with percent")]
    #[test_case("1Gi\"; reboot"; "quote injection")]
    #[test_case("１Gi"; "fullwidth digit")]
    fn rejects_ambiguous_invalid_and_overflowing_sizes(input: &str) {
        assert!(ZramSize::try_from(input).is_err());
    }

    #[test_case("1"; "number")]
    #[test_case("true"; "bool")]
    #[test_case("{}"; "object")]
    #[test_case("[]"; "array")]
    fn rejects_non_string_json(json: &str) {
        assert!(serde_json::from_str::<ZramSize>(json).is_err());
    }
}
