use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
pub struct Activity {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub timestamp: Option<i64>,
    pub description: Option<String>,
    #[serde(rename = "actionType")]
    pub action_type: Option<String>,
    #[serde(rename = "appId")]
    pub app_id: Option<i64>,
    #[serde(rename = "appName")]
    pub app_name: Option<String>,
    pub user: Option<Value>,
    pub device: Option<Value>,
    pub location: Option<Value>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}
