use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "entityType")]
    pub entity_type: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub domain: Option<String>,
    pub status: Option<i32>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityStatus {
    Na,
    Staged,
    Active,
    Suspended,
    Deleted,
}

impl EntityStatus {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Na),
            1 => Some(Self::Staged),
            2 => Some(Self::Active),
            3 => Some(Self::Suspended),
            4 => Some(Self::Deleted),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Na => 0,
            Self::Staged => 1,
            Self::Active => 2,
            Self::Suspended => 3,
            Self::Deleted => 4,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "na" | "n/a" => Some(Self::Na),
            "staged" => Some(Self::Staged),
            "active" => Some(Self::Active),
            "suspended" => Some(Self::Suspended),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}

impl fmt::Display for EntityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Na => write!(f, "N/A"),
            Self::Staged => write!(f, "STAGED"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Suspended => write!(f, "SUSPENDED"),
            Self::Deleted => write!(f, "DELETED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_status_roundtrip() {
        for val in 0..=4 {
            let status = EntityStatus::from_api(val).unwrap();
            assert_eq!(status.to_api(), val);
        }
    }

    #[test]
    fn test_entity_status_from_str() {
        assert_eq!(
            EntityStatus::from_str_loose("active"),
            Some(EntityStatus::Active)
        );
        assert_eq!(EntityStatus::from_str_loose("n/a"), Some(EntityStatus::Na));
    }
}
