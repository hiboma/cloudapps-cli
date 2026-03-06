use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ApiConfig {
    pub url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, AppError> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(|e| {
                AppError::Config(format!(
                    "failed to read config file {}: {}",
                    path.display(),
                    e
                ))
            })?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| AppError::Config(format!("failed to parse config file: {}", e)))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    fn config_path() -> PathBuf {
        dirs_config_path().join("config.toml")
    }
}

fn dirs_config_path() -> PathBuf {
    if let Some(config_dir) = dirs_home().map(|h| h.join(".config").join("cloudapps")) {
        config_dir
    } else {
        PathBuf::from(".config/cloudapps")
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Resolve a value from CLI option, environment variable, or config file (in priority order).
pub fn resolve_value(
    cli_value: Option<&str>,
    env_var: &str,
    config_value: Option<&str>,
) -> Option<String> {
    cli_value
        .map(String::from)
        .or_else(|| std::env::var(env_var).ok())
        .or_else(|| config_value.map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auth.token.is_none());
        assert!(config.api.url.is_none());
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[auth]
token = "test-token"

[api]
url = "https://example.com/api"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auth.token.as_deref(), Some("test-token"));
        assert_eq!(config.api.url.as_deref(), Some("https://example.com/api"));
    }

    #[test]
    fn test_resolve_value_priority() {
        // CLI option takes highest priority
        let result = resolve_value(Some("cli"), "NONEXISTENT_VAR", Some("config"));
        assert_eq!(result.as_deref(), Some("cli"));

        // Config is lowest priority
        let result = resolve_value(None, "NONEXISTENT_VAR_12345", Some("config"));
        assert_eq!(result.as_deref(), Some("config"));

        // None if nothing is set
        let result = resolve_value(None, "NONEXISTENT_VAR_12345", None);
        assert!(result.is_none());
    }
}
