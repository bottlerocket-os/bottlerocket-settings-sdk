//! Settings for driver branch selection.
//!
//! Provides the `DriversSettings` model which groups per-vendor driver
//! settings. Today it exposes `nvidia.branch`, allowing users to pin the NVIDIA
//! driver branch (`lts` or `pb`) used on multi-driver images. When unset, branch
//! selection falls back to instance-type detection.
use serde::{Deserialize, Serialize};
use std::fmt;

/// DriversSettings groups settings for the drivers shipped with the image.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DriversSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvidia: Option<NvidiaDriverSettings>,
}

/// NvidiaDriverSettings lets users change settings for the shipped NVIDIA driver.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NvidiaDriverSettings {
    pub branch: NvidiaDriverBranch,
}

/// The NVIDIA driver branch to activate on multi-driver images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NvidiaDriverBranch {
    /// Long-term support branch.
    #[default]
    Lts,
    /// Product branch.
    Pb,
}

impl fmt::Display for NvidiaDriverBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lts => write!(f, "lts"),
            Self::Pb => write!(f, "pb"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_nvidia_branch_lts() {
        let test_json = r#"{"nvidia":{"branch":"lts"}}"#;

        let settings: DriversSettings = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            settings,
            DriversSettings {
                nvidia: Some(NvidiaDriverSettings {
                    branch: NvidiaDriverBranch::Lts,
                }),
            }
        );

        let results = serde_json::to_string(&settings).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_serde_nvidia_branch_pb() {
        let test_json = r#"{"nvidia":{"branch":"pb"}}"#;

        let settings: DriversSettings = serde_json::from_str(test_json).unwrap();
        assert_eq!(
            settings,
            DriversSettings {
                nvidia: Some(NvidiaDriverSettings {
                    branch: NvidiaDriverBranch::Pb,
                }),
            }
        );

        let results = serde_json::to_string(&settings).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_serde_drivers_empty() {
        let test_json = r#"{}"#;

        let settings: DriversSettings = serde_json::from_str(test_json).unwrap();
        assert_eq!(settings, DriversSettings { nvidia: None });

        let results = serde_json::to_string(&settings).unwrap();
        assert_eq!(results, test_json);
    }

    #[test]
    fn test_serde_nvidia_branch_invalid() {
        let test_json = r#"{"nvidia":{"branch":"invalid"}}"#;
        assert!(serde_json::from_str::<DriversSettings>(test_json).is_err());
    }

    #[test]
    fn test_nvidia_driver_branch_display() {
        assert_eq!(NvidiaDriverBranch::Lts.to_string(), "lts");
        assert_eq!(NvidiaDriverBranch::Pb.to_string(), "pb");
    }
}
