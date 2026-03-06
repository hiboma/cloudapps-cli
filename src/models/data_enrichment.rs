use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct DataEnrichment {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub name: Option<String>,
    pub subnets: Option<Vec<Value>>,
    pub location: Option<Value>,
    pub organization: Option<String>,
    pub tags: Option<Vec<Value>>,
    pub category: Option<i32>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<i64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDataEnrichment {
    pub name: String,
    pub subnets: Vec<String>,
    pub category: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubnetCategory {
    Corporate,
    Administrative,
    Risky,
    Vpn,
    CloudProvider,
    Other,
}

impl SubnetCategory {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Corporate),
            2 => Some(Self::Administrative),
            3 => Some(Self::Risky),
            4 => Some(Self::Vpn),
            5 => Some(Self::CloudProvider),
            6 => Some(Self::Other),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Corporate => 1,
            Self::Administrative => 2,
            Self::Risky => 3,
            Self::Vpn => 4,
            Self::CloudProvider => 5,
            Self::Other => 6,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "corporate" => Some(Self::Corporate),
            "administrative" | "admin" => Some(Self::Administrative),
            "risky" => Some(Self::Risky),
            "vpn" => Some(Self::Vpn),
            "cloud-provider" | "cloud_provider" => Some(Self::CloudProvider),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

impl fmt::Display for SubnetCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corporate => write!(f, "CORPORATE"),
            Self::Administrative => write!(f, "ADMINISTRATIVE"),
            Self::Risky => write!(f, "RISKY"),
            Self::Vpn => write!(f, "VPN"),
            Self::CloudProvider => write!(f, "CLOUD_PROVIDER"),
            Self::Other => write!(f, "OTHER"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subnet_category_roundtrip() {
        for val in 1..=6 {
            let cat = SubnetCategory::from_api(val).unwrap();
            assert_eq!(cat.to_api(), val);
        }
    }

    #[test]
    fn test_subnet_category_from_str() {
        assert_eq!(
            SubnetCategory::from_str_loose("vpn"),
            Some(SubnetCategory::Vpn)
        );
        assert_eq!(
            SubnetCategory::from_str_loose("cloud-provider"),
            Some(SubnetCategory::CloudProvider)
        );
        assert_eq!(
            SubnetCategory::from_str_loose("admin"),
            Some(SubnetCategory::Administrative)
        );
    }
}
