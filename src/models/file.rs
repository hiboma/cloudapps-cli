use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "fileType")]
    pub file_type: Option<i32>,
    pub sharing: Option<i32>,
    #[serde(rename = "ownerName")]
    pub owner_name: Option<String>,
    #[serde(rename = "modifiedDate")]
    pub modified_date: Option<i64>,
    #[serde(rename = "createdDate")]
    pub created_date: Option<i64>,
    pub extension: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Other,
    Document,
    Spreadsheet,
    Presentation,
    Text,
    Image,
    Folder,
}

impl FileType {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Other),
            1 => Some(Self::Document),
            2 => Some(Self::Spreadsheet),
            3 => Some(Self::Presentation),
            4 => Some(Self::Text),
            5 => Some(Self::Image),
            6 => Some(Self::Folder),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Other => 0,
            Self::Document => 1,
            Self::Spreadsheet => 2,
            Self::Presentation => 3,
            Self::Text => 4,
            Self::Image => 5,
            Self::Folder => 6,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "other" => Some(Self::Other),
            "document" | "doc" => Some(Self::Document),
            "spreadsheet" => Some(Self::Spreadsheet),
            "presentation" => Some(Self::Presentation),
            "text" => Some(Self::Text),
            "image" => Some(Self::Image),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other => write!(f, "OTHER"),
            Self::Document => write!(f, "DOCUMENT"),
            Self::Spreadsheet => write!(f, "SPREADSHEET"),
            Self::Presentation => write!(f, "PRESENTATION"),
            Self::Text => write!(f, "TEXT"),
            Self::Image => write!(f, "IMAGE"),
            Self::Folder => write!(f, "FOLDER"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharingLevel {
    Private,
    Internal,
    External,
    Public,
    Internet,
}

impl SharingLevel {
    pub fn from_api(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Private),
            1 => Some(Self::Internal),
            2 => Some(Self::External),
            3 => Some(Self::Public),
            4 => Some(Self::Internet),
            _ => None,
        }
    }

    pub fn to_api(self) -> i32 {
        match self {
            Self::Private => 0,
            Self::Internal => 1,
            Self::External => 2,
            Self::Public => 3,
            Self::Internet => 4,
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "private" => Some(Self::Private),
            "internal" => Some(Self::Internal),
            "external" => Some(Self::External),
            "public" => Some(Self::Public),
            "internet" => Some(Self::Internet),
            _ => None,
        }
    }
}

impl fmt::Display for SharingLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Private => write!(f, "PRIVATE"),
            Self::Internal => write!(f, "INTERNAL"),
            Self::External => write!(f, "EXTERNAL"),
            Self::Public => write!(f, "PUBLIC"),
            Self::Internet => write!(f, "INTERNET"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_roundtrip() {
        for val in 0..=6 {
            let ft = FileType::from_api(val).unwrap();
            assert_eq!(ft.to_api(), val);
        }
    }

    #[test]
    fn test_sharing_level_roundtrip() {
        for val in 0..=4 {
            let sl = SharingLevel::from_api(val).unwrap();
            assert_eq!(sl.to_api(), val);
        }
    }

    #[test]
    fn test_file_type_from_str() {
        assert_eq!(
            FileType::from_str_loose("document"),
            Some(FileType::Document)
        );
        assert_eq!(FileType::from_str_loose("doc"), Some(FileType::Document));
    }

    #[test]
    fn test_sharing_level_from_str() {
        assert_eq!(
            SharingLevel::from_str_loose("internet"),
            Some(SharingLevel::Internet)
        );
    }
}
