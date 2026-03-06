use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Alert {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub timestamp: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub entities: Option<Vec<Value>>,
    #[serde(rename = "statusValue")]
    pub status_value: Option<i32>,
    #[serde(rename = "severityValue")]
    pub severity_value: Option<i32>,
    #[serde(rename = "resolutionStatusValue")]
    pub resolution_status_value: Option<i32>,
    pub stories: Option<Vec<i32>>,
    pub evidence: Option<Vec<Value>>,
    pub intent: Option<Vec<i32>>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Informational,
}

impl Severity {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Low),
            1 => Some(Self::Medium),
            2 => Some(Self::High),
            3 => Some(Self::Informational),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Informational => 3,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "informational" => Some(Self::Informational),
            _ => None,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Informational => write!(f, "INFORMATIONAL"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStatus {
    Open,
    Dismissed,
    Resolved,
    FalsePositive,
    Benign,
    TruePositive,
}

impl ResolutionStatus {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Open),
            1 => Some(Self::Dismissed),
            2 => Some(Self::Resolved),
            3 => Some(Self::FalsePositive),
            4 => Some(Self::Benign),
            5 => Some(Self::TruePositive),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Open => 0,
            Self::Dismissed => 1,
            Self::Resolved => 2,
            Self::FalsePositive => 3,
            Self::Benign => 4,
            Self::TruePositive => 5,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "open" => Some(Self::Open),
            "dismissed" => Some(Self::Dismissed),
            "resolved" => Some(Self::Resolved),
            "false-positive" | "false_positive" => Some(Self::FalsePositive),
            "benign" => Some(Self::Benign),
            "true-positive" | "true_positive" => Some(Self::TruePositive),
            _ => None,
        }
    }
}

impl fmt::Display for ResolutionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => write!(f, "OPEN"),
            Self::Dismissed => write!(f, "DISMISSED"),
            Self::Resolved => write!(f, "RESOLVED"),
            Self::FalsePositive => write!(f, "FALSE_POSITIVE"),
            Self::Benign => write!(f, "BENIGN"),
            Self::TruePositive => write!(f, "TRUE_POSITIVE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseType {
    Benign,
    FalsePositive,
    TruePositive,
}

impl CloseType {
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "benign" => Some(Self::Benign),
            "false-positive" | "false_positive" => Some(Self::FalsePositive),
            "true-positive" | "true_positive" => Some(Self::TruePositive),
            _ => None,
        }
    }

    pub fn api_path_segment(self) -> &'static str {
        match self {
            Self::Benign => "close_benign",
            Self::FalsePositive => "close_false_positive",
            Self::TruePositive => "close_true_positive",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_roundtrip() {
        for val in 0..=3 {
            let sev = Severity::from_api(val).unwrap();
            assert_eq!(sev.to_api(), val);
        }
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str_loose("high"), Some(Severity::High));
        assert_eq!(Severity::from_str_loose("HIGH"), Some(Severity::High));
        assert_eq!(Severity::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_resolution_status_roundtrip() {
        for val in 0..=5 {
            let status = ResolutionStatus::from_api(val).unwrap();
            assert_eq!(status.to_api(), val);
        }
    }

    #[test]
    fn test_close_type_path() {
        assert_eq!(CloseType::Benign.api_path_segment(), "close_benign");
        assert_eq!(
            CloseType::FalsePositive.api_path_segment(),
            "close_false_positive"
        );
        assert_eq!(
            CloseType::TruePositive.api_path_segment(),
            "close_true_positive"
        );
    }

    #[test]
    fn test_close_type_from_str() {
        assert_eq!(
            CloseType::from_str_loose("false-positive"),
            Some(CloseType::FalsePositive)
        );
        assert_eq!(
            CloseType::from_str_loose("true_positive"),
            Some(CloseType::TruePositive)
        );
    }
}
