/// Resolve a value from CLI option or environment variable (in priority order).
pub fn resolve_value(cli_value: Option<&str>, env_var: &str) -> Option<String> {
    cli_value
        .map(String::from)
        .or_else(|| std::env::var(env_var).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_value_priority() {
        // CLI option takes highest priority
        let result = resolve_value(Some("cli"), "NONEXISTENT_VAR");
        assert_eq!(result.as_deref(), Some("cli"));

        // None if nothing is set
        let result = resolve_value(None, "NONEXISTENT_VAR_12345");
        assert!(result.is_none());
    }
}
