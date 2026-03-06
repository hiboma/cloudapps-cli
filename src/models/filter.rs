use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A collection of filters to send in API requests.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FilterRequest {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub filters: HashMap<String, FilterCondition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// A single filter condition with operator and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition(HashMap<String, Value>);

impl FilterCondition {
    pub fn eq<V: Into<Value>>(values: Vec<V>) -> Self {
        let mut map = HashMap::new();
        let arr: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        map.insert("eq".to_string(), Value::Array(arr));
        Self(map)
    }

    pub fn neq<V: Into<Value>>(values: Vec<V>) -> Self {
        let mut map = HashMap::new();
        let arr: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        map.insert("neq".to_string(), Value::Array(arr));
        Self(map)
    }

    pub fn gte<V: Into<Value>>(value: V) -> Self {
        let mut map = HashMap::new();
        map.insert("gte".to_string(), value.into());
        Self(map)
    }

    pub fn lte<V: Into<Value>>(value: V) -> Self {
        let mut map = HashMap::new();
        map.insert("lte".to_string(), value.into());
        Self(map)
    }

    pub fn gte_ndays(days: u64) -> Self {
        let mut map = HashMap::new();
        map.insert("gte_ndays".to_string(), Value::Number(days.into()));
        Self(map)
    }

    pub fn text(query: &str) -> Self {
        let mut map = HashMap::new();
        map.insert("text".to_string(), Value::String(query.to_string()));
        Self(map)
    }

    pub fn eq_bool(value: bool) -> Self {
        let mut map = HashMap::new();
        map.insert("eq".to_string(), Value::Bool(value));
        Self(map)
    }

    pub fn contains(values: Vec<String>) -> Self {
        let mut map = HashMap::new();
        let arr: Vec<Value> = values.into_iter().map(Value::String).collect();
        map.insert("contains".to_string(), Value::Array(arr));
        Self(map)
    }

    pub fn startswith(values: Vec<String>) -> Self {
        let mut map = HashMap::new();
        let arr: Vec<Value> = values.into_iter().map(Value::String).collect();
        map.insert("startswith".to_string(), Value::Array(arr));
        Self(map)
    }

    /// Build from a raw operator-value pair.
    pub fn raw(operator: String, value: Value) -> Self {
        let mut map = HashMap::new();
        map.insert(operator, value);
        Self(map)
    }
}

impl FilterRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_filter(&mut self, field: &str, condition: FilterCondition) {
        self.filters.insert(field.to_string(), condition);
    }

    pub fn with_skip(mut self, skip: u64) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty() && self.skip.is_none() && self.limit.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_eq() {
        let condition = FilterCondition::eq(vec!["value1", "value2"]);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json, serde_json::json!({"eq": ["value1", "value2"]}));
    }

    #[test]
    fn test_filter_gte() {
        let condition = FilterCondition::gte(1700000000000_i64);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json, serde_json::json!({"gte": 1700000000000_i64}));
    }

    #[test]
    fn test_filter_gte_ndays() {
        let condition = FilterCondition::gte_ndays(7);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json, serde_json::json!({"gte_ndays": 7}));
    }

    #[test]
    fn test_filter_text() {
        let condition = FilterCondition::text("failed login");
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json, serde_json::json!({"text": "failed login"}));
    }

    #[test]
    fn test_filter_request_serialization() {
        let mut req = FilterRequest::new().with_skip(5).with_limit(10);
        req.add_filter("severity", FilterCondition::eq(vec![2]));
        req.add_filter("alertOpen", FilterCondition::eq_bool(true));

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["skip"], 5);
        assert_eq!(json["limit"], 10);
        assert_eq!(json["filters"]["severity"]["eq"], serde_json::json!([2]));
        assert_eq!(json["filters"]["alertOpen"]["eq"], true);
    }

    #[test]
    fn test_empty_filter_request() {
        let req = FilterRequest::new();
        assert!(req.is_empty());
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, "{}");
    }
}
