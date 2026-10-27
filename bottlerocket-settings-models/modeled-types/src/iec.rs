//! Shared parser for integer byte quantities with IEC binary units.

/// A byte quantity parsed by [`parse_iec_quantity`].
pub(crate) struct IecQuantity {
    pub(crate) bytes: u64,
    pub(crate) has_unit: bool,
}

const UNITS: [(&str, u64); 4] = [
    ("Ki", 1 << 10),
    ("Mi", 1 << 20),
    ("Gi", 1 << 30),
    ("Ti", 1 << 40),
];

/// Parses an unsigned decimal integer without a sign or leading zeros.
pub(crate) fn parse_decimal(digits: &str) -> Option<u64> {
    // u64::from_str accepts a leading '+' and leading zeros, so check the digits first.
    let well_formed = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'));
    if !well_formed {
        return None;
    }
    digits.parse().ok()
}

pub(crate) fn parse_iec_quantity(input: &str) -> Option<IecQuantity> {
    let (digits, multiplier) = UNITS
        .into_iter()
        .find_map(|(unit, multiplier)| input.strip_suffix(unit).map(|d| (d, Some(multiplier))))
        .unwrap_or((input, None));
    let bytes = parse_decimal(digits)?.checked_mul(multiplier.unwrap_or(1))?;
    Some(IecQuantity {
        bytes,
        has_unit: multiplier.is_some(),
    })
}

#[cfg(test)]
mod test_parse_iec_quantity {
    use super::{parse_decimal, parse_iec_quantity};
    use test_case::test_case;

    #[test_case("0", 0; "zero")]
    #[test_case("7", 7; "single digit")]
    #[test_case("100", 100; "multiple digits")]
    #[test_case("18446744073709551615", u64::MAX; "u64 max")]
    fn decimal(input: &str, value: u64) {
        assert_eq!(parse_decimal(input), Some(value));
    }

    #[test_case(""; "empty")]
    #[test_case("00"; "double zero")]
    #[test_case("07"; "leading zero")]
    #[test_case("+7"; "plus sign")]
    #[test_case("-7"; "minus sign")]
    #[test_case("7.0"; "fraction")]
    #[test_case(" 7"; "whitespace")]
    #[test_case("7Ki"; "unit")]
    #[test_case("１"; "fullwidth digit")]
    #[test_case("18446744073709551616"; "u64 overflow")]
    fn invalid_decimal(input: &str) {
        assert_eq!(parse_decimal(input), None);
    }

    #[test_case("0", 0, false; "zero")]
    #[test_case("1", 1, false; "one byte")]
    #[test_case("1048576", 1 << 20, false; "bare bytes")]
    #[test_case("18446744073709551615", u64::MAX, false; "u64 max")]
    #[test_case("0Ki", 0, true; "zero with unit")]
    #[test_case("1Ki", 1 << 10, true; "kibibytes")]
    #[test_case("512Mi", 512 << 20, true; "mebibytes")]
    #[test_case("2Gi", 2 << 30, true; "gibibytes")]
    #[test_case("1Ti", 1 << 40, true; "tebibytes")]
    #[test_case("16777215Ti", ((1 << 24) - 1) << 40, true; "largest tebibytes")]
    fn valid(input: &str, bytes: u64, has_unit: bool) {
        let quantity = parse_iec_quantity(input).unwrap();
        assert_eq!(quantity.bytes, bytes);
        assert_eq!(quantity.has_unit, has_unit);
    }

    #[test_case(""; "empty")]
    #[test_case("Mi"; "no number")]
    #[test_case("00"; "double zero")]
    #[test_case("01"; "leading zero")]
    #[test_case("01Gi"; "leading zero with unit")]
    #[test_case("00Mi"; "double zero with unit")]
    #[test_case("+1"; "plus sign")]
    #[test_case("-1"; "minus sign")]
    #[test_case("1.5Gi"; "fraction")]
    #[test_case("1e9"; "exponent")]
    #[test_case(" 1Gi"; "leading space")]
    #[test_case("1Gi "; "trailing space")]
    #[test_case("1 Gi"; "space before unit")]
    #[test_case("1mi"; "lowercase unit")]
    #[test_case("1M"; "SI suffix")]
    #[test_case("1Pi"; "unsupported unit")]
    #[test_case("1Gib"; "trailing b")]
    #[test_case("1GiGi"; "repeated unit")]
    #[test_case("１Gi"; "fullwidth digit")]
    #[test_case("18446744073709551616"; "u64 overflow")]
    #[test_case("16777216Ti"; "multiplication overflow")]
    fn invalid(input: &str) {
        assert!(parse_iec_quantity(input).is_none());
    }
}
