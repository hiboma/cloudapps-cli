use std::io::{self, Write};

use crate::error::AppError;

/// Write a line to stdout, silently ignoring BrokenPipe errors.
fn writeln_stdout(s: &str) -> Result<(), AppError> {
    match writeln!(io::stdout(), "{}", s) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(AppError::Network(e.to_string())),
    }
}

/// Print a value as formatted JSON to stdout.
pub fn print_json<T: serde::Serialize>(value: &T, minify: bool) -> Result<(), AppError> {
    let output = if minify {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };
    writeln_stdout(&output)
}

/// Print a raw JSON string to stdout.
pub fn print_json_raw(value: &serde_json::Value, minify: bool) -> Result<(), AppError> {
    let output = if minify {
        serde_json::to_string(value)?
    } else {
        serde_json::to_string_pretty(value)?
    };
    writeln_stdout(&output)
}

/// Print JSON output, extracting the "data" field unless raw mode is enabled.
pub fn print_json_data(value: &serde_json::Value, raw: bool, minify: bool) -> Result<(), AppError> {
    if raw {
        print_json_raw(value, minify)
    } else {
        let data = value.get("data").unwrap_or(value);
        print_json_raw(data, minify)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_print_json_raw() {
        let value = json!({"key": "value"});
        let result = print_json_raw(&value, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_with_struct() {
        #[derive(serde::Serialize)]
        struct Sample {
            name: String,
            count: i32,
        }
        let s = Sample {
            name: "test".to_string(),
            count: 42,
        };
        let result = print_json(&s, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_raw_array() {
        let value = json!([1, 2, 3]);
        let result = print_json_raw(&value, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_raw_nested() {
        let value = json!({"a": {"b": {"c": 1}}});
        let result = print_json_raw(&value, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_json_raw_null() {
        let value = json!(null);
        let result = print_json_raw(&value, false);
        assert!(result.is_ok());
    }
}
