use crate::error;
use bottlerocket_scalar_derive::Scalar;
use bottlerocket_string_impls_for::string_impls_for;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use snafu::{ensure, OptionExt};
use std::collections::HashMap;

/// HugepagesSettings contains static hugepages and transparent hugepages related settings.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HugepagesSettings {
    #[serde(skip_serializing_if = "Option::is_none", rename = "transparent")]
    pub transparent_hugepages: Option<HugepagesTransparent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "static")]
    pub static_hugepages: Option<HugepagesStatic>,
}

/// Supported static hugepages knobs in Bottlerocket.
/// essential takes boolean variable. We set it to true if we want the boot to fail if
/// the user doesn't get the requested number of hugepages allocation. hugepages_config
/// is a hashmap where key is the hugepage size and value is either total count or
/// NUMA node-wise distribution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HugepagesStatic {
    #[serde(default)]
    pub essential: bool,
    #[serde(flatten, deserialize_with = "deserialize_hugepages_config")]
    pub hugepages_config: HashMap<HugepageSize, HugepageConfig>,
}

/// Deserializes the per-size huge page pools, rejecting sizes that are written
/// differently but name the same page size.
///
/// `HugepageSize` keeps the string it was given, so `2Mi` and `2048Ki` are
/// distinct keys in the map even though both resolve to 2048 kiB. Consumers
/// address a pool by its size in kibibytes -- corndog writes to
/// `/sys/kernel/mm/hugepages/hugepages-<size>kB/nr_hugepages` -- so a map
/// holding both would have two counts for one pool, and whichever the consumer
/// applied last would win. Rejecting the input is better than silently keeping
/// one of the two counts.
fn deserialize_hugepages_config<'de, D>(
    deserializer: D,
) -> Result<HashMap<HugepageSize, HugepageConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let hugepages_config = HashMap::<HugepageSize, HugepageConfig>::deserialize(deserializer)?;

    // Keyed by size in kibibytes, so equivalent spellings collide here.
    let mut seen: HashMap<u64, &HugepageSize> = HashMap::with_capacity(hugepages_config.len());
    for size in hugepages_config.keys() {
        // `HugepageSize` only exists if it parsed, so this cannot fail in practice.
        let size_kib = size.as_kib().ok_or_else(|| {
            D::Error::custom(format!(
                "unable to determine the size of '{size}' in kibibytes"
            ))
        })?;

        if let Some(existing) = seen.insert(size_kib, size) {
            // The map iterates in an arbitrary order; sort so the message does
            // not depend on which of the two was reached first.
            let mut names = [existing.to_string(), size.to_string()];
            names.sort();
            return Err(D::Error::custom(format!(
                "huge page sizes '{}' and '{}' both mean {} kiB; specify only one of them",
                names[0], names[1], size_kib,
            )));
        }
    }

    Ok(hugepages_config)
}

/// HugepageConfig contains all the configurations related to 1 type of static hugepage.
/// count contains the requested number of hugepages for the instance or NUMA node-wise distribution.
/// NUMA node-wise distribution has to follow a particular convention mentioned in the regex below.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HugepageConfig {
    pub count: HugepageAllocation,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct HugepageAllocation {
    inner: String,
}

lazy_static! {
    // This parameter accepts either a single non-negative integer (the number
    // of pages), or a comma-separated list of `<node>:<pages>` pairs where
    // both `<node>` and `<pages>` are non-negative integers.
    static ref HUGEPAGE_ALLOCATION_RE: Regex =
        Regex::new(r"^(?:[0-9]+|[0-9]+:[0-9]+(?:,[0-9]+:[0-9]+)*)$").unwrap();
}

/// Validate format.
impl TryFrom<&str> for HugepageAllocation {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            HUGEPAGE_ALLOCATION_RE.is_match(input),
            error::InvalidHugepageAllocationSnafu { input }
        );

        Ok(HugepageAllocation {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(HugepageAllocation, "HugepageAllocation");

lazy_static! {
    // A positive integer followed by an IEC binary unit (Ki, Mi, Gi, Ti).
    static ref HUGEPAGE_SIZE_RE: Regex =
        Regex::new(r"^(?P<value>[1-9][0-9]*)(?P<unit>Ki|Mi|Gi|Ti)$").unwrap();
}

/// HugepageSize represents an explicit HugeTLB page size in IEC binary unit, e.g. `2Mi` or `1Gi`.
/// We chose validated string instead of enum to accomodate arch specific allowed values.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HugepageSize {
    inner: String,
}

impl HugepageSize {
    /// Size in kibibytes, as used by the kernel's sysfs `hugepages-<N>kB` directories.
    pub fn as_kib(&self) -> Option<u64> {
        if let Some(nbytes) = Self::parse(&self.inner) {
            return Some(nbytes / 1024);
        }

        None
    }

    /// Returns number of bytes if `input` is well-formed and does not overflow `u64`.
    fn parse(input: &str) -> Option<u64> {
        if let Some(captured) = HUGEPAGE_SIZE_RE.captures(input) {
            let Ok(value) = captured["value"].parse::<u64>() else {
                return None;
            };

            let multiplier: u64 = match &captured["unit"] {
                "Ki" => 1 << 10,
                "Mi" => 1 << 20,
                "Gi" => 1 << 30,
                "Ti" => 1 << 40,
                _ => return None,
            };

            return value.checked_mul(multiplier);
        }

        None
    }
}

/// Validate format and the power-of-two invariant before we accept the input.
impl TryFrom<&str> for HugepageSize {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let bytes = HugepageSize::parse(input).context(error::InvalidHugepageSizeSnafu {
            input,
            msg: "must be a positive integer with an IEC binary unit (Ki, Mi, Gi, Ti) \
                in u64, e.g. \"2Mi\" or \"1Gi\"",
        })?;

        ensure!(
            bytes.is_power_of_two(),
            error::InvalidHugepageSizeSnafu {
                input,
                msg: "huge page size must be a power of two",
            }
        );

        Ok(HugepageSize {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(HugepageSize, "HugepageSize");

/// Supported transparent hugepages configurations in Bottlerocket
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct HugepagesTransparent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<TransparentHugepagePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defrag: Option<TransparentHugepageDefragPolicy>,
}

/// Acceptable transparent hugepages policies
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Scalar, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransparentHugepagePolicy {
    Never,
    #[default]
    Madvise,
    Always,
}

#[cfg(test)]
mod test_thp_policy {
    use super::TransparentHugepagePolicy;
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test_case("never")]
    #[test_case("madvise")]
    #[test_case("always")]
    fn good_policy(input: &str) {
        assert!(TransparentHugepagePolicy::try_from(input).is_ok());
    }

    #[test_case("disable")]
    #[test_case("enable")]
    #[test_case("")]
    #[test_case("sometimes")]
    #[test_case(&"a".repeat(64))]
    fn bad_policy(input: &str) {
        assert!(TransparentHugepagePolicy::try_from(input).is_err());
    }
}

/// Acceptable transparent hugepages defragmentation policies
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Scalar, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransparentHugepageDefragPolicy {
    Never,
    #[default]
    Madvise,
    Defer,
    #[serde(rename = "defer+madvise")]
    DeferMadvise,
    Always,
}

#[cfg(test)]
mod test_thp_defrag_policy {
    use super::TransparentHugepageDefragPolicy;
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test_case("never")]
    #[test_case("madvise")]
    #[test_case("defer")]
    #[test_case("defer+madvise")]
    #[test_case("always")]
    fn good_policy(input: &str) {
        assert!(TransparentHugepageDefragPolicy::try_from(input).is_ok());
    }

    #[test_case("disable")]
    #[test_case("enable")]
    #[test_case("")]
    #[test_case("sometimes")]
    #[test_case(&"a".repeat(64))]
    fn bad_policy(input: &str) {
        assert!(TransparentHugepageDefragPolicy::try_from(input).is_err());
    }
}

#[cfg(test)]
mod test_hugepage_size {
    use super::HugepageSize;
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test_case("64Ki", 64 ; "64 KB hugepages")]
    #[test_case("2Mi", 2048; "2 MB hugepages")]
    #[test_case("512Mi", 524_288 ; "512 MB")]
    #[test_case("1Gi", 1_048_576 ; "1 GB hugepages")]
    #[test_case("16Gi", 16 * 1_048_576 ; "16 GB")]
    fn valid(input: &str, kib: u64) {
        let h = HugepageSize::try_from(input);
        assert!(h.is_ok());
        assert_eq!(h.unwrap().as_kib(), Some(kib), "{input} kib");
    }

    #[test_case(""; "empty")]
    #[test_case("2"; "no unit")]
    #[test_case("2M"; "SI-style suffix not accepted")]
    #[test_case("2mi"; "wrong case")]
    #[test_case("Mi"; "no mantissa")]
    #[test_case("0Mi"; "zero")]
    #[test_case("3Mi"; "not a power of two")]
    #[test_case("1.5Gi"; "not an integer")]
    #[test_case("-2Mi"; "negative")]
    #[test_case("2 Mi"; "whitespace")]
    #[test_case("2Pi"; "unsupported unit")]
    fn invalid(input: &str) {
        assert!(
            HugepageSize::try_from(input).is_err(),
            "expected '{input}' to be rejected"
        );
    }
}

#[cfg(test)]
mod test_hugepage_allocation {
    use super::HugepageAllocation;
    use std::convert::TryFrom;
    use test_case::test_case;

    #[test_case("0"; "zero")]
    #[test_case("128"; "single non-negative integer")]
    #[test_case("0:128"; "single node pair")]
    #[test_case("0:128,1:256"; "multiple node pairs")]
    #[test_case("0:0,1:0,2:512"; "multiple node pairs with zeros")]
    fn valid(input: &str) {
        assert!(
            HugepageAllocation::try_from(input).is_ok(),
            "expected '{input}' to be accepted"
        );
    }

    #[test_case(""; "empty")]
    #[test_case("-1"; "negative")]
    #[test_case("12.5"; "not an integer")]
    #[test_case("abc"; "non-numeric")]
    #[test_case("0:"; "missing pages")]
    #[test_case(":128"; "missing node")]
    #[test_case("0:128,"; "trailing comma")]
    #[test_case(",0:128"; "leading comma")]
    #[test_case("0:128,1"; "incomplete pair")]
    #[test_case("0:128 ,1:256"; "whitespace")]
    #[test_case("0:1:2"; "too many parts")]
    fn invalid(input: &str) {
        assert!(
            HugepageAllocation::try_from(input).is_err(),
            "expected '{input}' to be rejected"
        );
    }
}

#[cfg(test)]
mod test_hugepages_settings {
    use super::{
        HugepageSize, HugepagesSettings, TransparentHugepageDefragPolicy, TransparentHugepagePolicy,
    };
    use std::convert::TryFrom;

    #[test]
    fn pool_count_table_deserializes_and_round_trips() {
        let settings: HugepagesSettings = serde_json::from_str(
            r#"
            {
                "static": {
                    "essential": true,
                    "1Gi": {
                        "count": "2"
                    }
                },
                "transparent": {
                    "enabled": "always",
                    "defrag": "madvise"
                }
            }"#,
        )
        .unwrap();
        let static_hugepages = settings
            .static_hugepages
            .as_ref()
            .expect("static hugepages");
        let transparent_hugepages = settings
            .transparent_hugepages
            .as_ref()
            .expect("transparent hugepages");
        let key = HugepageSize::try_from("1Gi").unwrap();
        let pool = static_hugepages
            .hugepages_config
            .get(&key)
            .expect("1Gi pool");
        assert_eq!(pool.count, "2");
        assert_eq!(static_hugepages.essential, true);
        assert_eq!(
            transparent_hugepages.enabled,
            Some(TransparentHugepagePolicy::Always)
        );
        assert_eq!(
            transparent_hugepages.defrag,
            Some(TransparentHugepageDefragPolicy::Madvise)
        );

        // Round-trips back to the same nested shape.
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(
            json,
            serde_json::json!(
                {
                    "static": {
                        "1Gi": {
                            "count": "2"
                        },
                        "essential": true
                    },
                    "transparent": {
                        "enabled": "always",
                        "defrag": "madvise"
                    }
                }
            )
        );
    }

    #[test]
    fn equivalent_sizes_are_rejected() {
        // Both keys mean 2048 kiB, so the two counts would fight over one pool.
        let err = serde_json::from_str::<HugepagesSettings>(
            r#"{"static": {"2Mi": {"count": "512"}, "2048Ki": {"count": "4"}}}"#,
        )
        .expect_err("expected equivalent huge page sizes to be rejected");

        let message = err.to_string();
        assert!(
            message.contains("2048Ki") && message.contains("2Mi") && message.contains("2048 kiB"),
            "error should name both sizes and the size they share, got: {message}"
        );
    }

    #[test]
    fn distinct_sizes_are_accepted() {
        let settings: HugepagesSettings =
            serde_json::from_str(r#"{"static": {"2Mi": {"count": "512"}, "1Gi": {"count": "4"}}}"#)
                .unwrap();

        let static_hugepages = settings
            .static_hugepages
            .as_ref()
            .expect("static hugepages");
        assert_eq!(static_hugepages.hugepages_config.len(), 2);
        for (size, count) in [("2Mi", "512"), ("1Gi", "4")] {
            let key = HugepageSize::try_from(size).unwrap();
            let pool = static_hugepages
                .hugepages_config
                .get(&key)
                .unwrap_or_else(|| panic!("{size} pool"));
            assert_eq!(pool.count, count);
        }
    }

    #[test]
    fn pool_count_table_deserializes_default() {
        let settings: HugepagesSettings =
            serde_json::from_str(r#"{"static": {"1Gi": {"count": "2"}}}"#).unwrap();
        let key = HugepageSize::try_from("1Gi").unwrap();
        let static_hugepages = settings
            .static_hugepages
            .as_ref()
            .expect("static hugepages");
        let pool = static_hugepages
            .hugepages_config
            .get(&key)
            .expect("1Gi pool");
        assert_eq!(pool.count, "2");
        assert_eq!(static_hugepages.essential, false);
    }
}
