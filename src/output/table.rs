use chrono::{DateTime, Utc};

/// Convert epoch milliseconds to ISO 8601 string.
pub fn format_timestamp(epoch_ms: Option<i64>) -> String {
    match epoch_ms {
        Some(ms) => {
            let secs = ms / 1000;
            let nsecs = ((ms % 1000) * 1_000_000) as u32;
            match DateTime::<Utc>::from_timestamp(secs, nsecs) {
                Some(dt) => dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                None => "INVALID".to_string(),
            }
        }
        None => "-".to_string(),
    }
}

/// Truncate a string to max_len, appending "..." if truncated.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp() {
        // 2023-11-14T22:13:20Z = 1700000000000 ms
        assert_eq!(
            format_timestamp(Some(1700000000000)),
            "2023-11-14T22:13:20Z"
        );
    }

    #[test]
    fn test_format_timestamp_none() {
        assert_eq!(format_timestamp(None), "-");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("ab", 2), "ab");
    }
}
