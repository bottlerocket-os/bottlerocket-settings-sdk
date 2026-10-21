use crate::error;
use bottlerocket_string_impls_for::string_impls_for;
use serde::{Deserialize, Serialize};

/// Zram configuration. Only variants that include zram swap apply these settings.
/// Changing the size takes effect after reboot, never by resizing live swap.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZramSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ZramSize>,
}

/// A non-negative integer byte quantity (e.g. `512Mi`, `2Gi`) or an integer
/// percentage of MemTotal (`0%` through `100%`). Bare integers represent bytes.
/// Supported suffixes are Ki/Mi/Gi; fractional quantities,
/// signs, exponents and whitespace are not accepted. Zero disables zram swap.
/// Nonzero absolute sizes must be at least 1 MiB. Percentages are checked at boot.
/// At boot, sizes are rounded down to a page boundary and must fit a swap header.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ZramSize {
    inner: String,
    value: Size,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Size {
    Bytes(u64),
    Percent(u8),
}

impl ZramSize {
    /// Resolve percentages against the supplied MemTotal in bytes, rounding down.
    /// Page alignment is left to the caller, which knows the running kernel.
    pub fn as_bytes(&self, mem_total_bytes: u64) -> u64 {
        match self.value {
            Size::Bytes(bytes) => bytes,
            Size::Percent(percent) => ((mem_total_bytes as u128 * percent as u128) / 100) as u64,
        }
    }
}

fn parse(input: &str) -> Option<Size> {
    let digits = input.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let number: u64 = input[..digits].parse().ok()?;
    let suffix = &input[digits..];
    if suffix == "%" {
        return (number <= 100).then_some(Size::Percent(number as u8));
    }
    let multiplier: u64 = match suffix {
        "" => 1,
        "Ki" => 1 << 10,
        "Mi" => 1 << 20,
        "Gi" => 1 << 30,
        _ => return None,
    };
    let bytes = number.checked_mul(multiplier)?;
    (bytes == 0 || bytes >= 1 << 20).then_some(Size::Bytes(bytes))
}

impl TryFrom<&str> for ZramSize {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        crate::reject_control_chars(input, "ZramSize")?;
        let value = parse(input).ok_or_else(|| error::Error::InvalidZramSize {
            input: input.to_owned(),
        })?;
        Ok(Self {
            inner: input.to_owned(),
            value,
        })
    }
}
string_impls_for!(ZramSize, "ZramSize");

#[cfg(test)]
mod test_zram_size {
    use super::*;

    #[test]
    fn absolute_sizes_and_round_trip() {
        for (input, bytes) in [
            ("0", 0),
            ("0Ki", 0),
            ("0Mi", 0),
            ("0Gi", 0),
            ("512Mi", 512 << 20),
            ("2Gi", 2 << 30),
            ("1024Ki", 1 << 20),
            ("1Mi", 1 << 20),
            ("1024Gi", 1 << 40),
            ("1048576", 1048576),
            ("18446744073709551615", u64::MAX),
        ] {
            let size = ZramSize::try_from(input).unwrap();
            assert_eq!(size.as_bytes(123), bytes, "{input}");
            let json = serde_json::to_string(&size).unwrap();
            assert_eq!(json, format!("\"{input}\""));
            assert_eq!(serde_json::from_str::<ZramSize>(&json).unwrap(), size);
        }
    }

    #[test]
    fn rejects_absolute_sizes_below_one_mib_and_unsupported_suffixes() {
        for bad in [
            "1", "655360", "1048575", "1Ki", "1023Ki", "1Ti", "1Pi", "1Ei", "2000k", "128M", "1G",
            "1T", "1P", "1E", "0Ti", "0M",
        ] {
            assert!(ZramSize::try_from(bad).is_err(), "{bad}");
        }
        // Percentages are validated against the running node's memory at boot.
        assert_eq!(ZramSize::try_from("1%").unwrap().as_bytes(100), 1);
    }

    #[test]
    fn percentages_and_overflow() {
        for (input, expected) in [("0%", 0), ("25%", 1024), ("100%", 4099)] {
            assert_eq!(ZramSize::try_from(input).unwrap().as_bytes(4099), expected);
        }
        assert_eq!(
            ZramSize::try_from("100%").unwrap().as_bytes(u64::MAX),
            u64::MAX
        );
    }

    #[test]
    fn rejects_ambiguous_invalid_and_overflowing_sizes() {
        for bad in [
            "",
            "-1",
            "+1Gi",
            "1.5Gi",
            "1e9",
            "100m",
            " 1Gi",
            "1Gi ",
            "1Gi\n",
            "101%",
            "1.5%",
            "%",
            "NaN",
            "1Gib",
            "1GB",
            "1mi",
            "18446744073709551616",
            "16Ei",
            "18446744073709551615Gi",
            "1Gi\"; reboot",
            "１Gi",
        ] {
            assert!(ZramSize::try_from(bad).is_err(), "{bad:?}");
        }
        for bad in ["1", "true", "{}", "[]"] {
            assert!(serde_json::from_str::<ZramSize>(bad).is_err(), "{bad}");
        }
    }
}
