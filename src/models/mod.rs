pub mod activity;
pub mod alert;
pub mod data_enrichment;
pub mod entity;
pub mod file;
pub mod filter;

use serde::Deserialize;

/// Generic list response from the API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T> {
    pub total: Option<i64>,
    pub has_next: bool,
    pub data: Vec<T>,
}
