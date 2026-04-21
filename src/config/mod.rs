pub mod credential_store;

use serde::Deserialize;
use std::path::PathBuf;

use credential_store::{CredentialStore, KEY_API_TOKEN, StoreError, default_store};

/// Resolve a value from CLI option or environment variable (in priority order).
pub fn resolve_value(cli_value: Option<&str>, env_var: &str) -> Option<String> {
    cli_value
        .map(String::from)
        .or_else(|| std::env::var(env_var).ok())
}

/// TOML representation of `[credentials]` in credentials.toml.
#[derive(Debug, Deserialize, Default)]
struct CredentialsFileRoot {
    #[serde(default)]
    credentials: CredentialsFile,
}

#[derive(Debug, Deserialize, Default)]
struct CredentialsFile {
    api_url: Option<String>,
    api_token: Option<String>,
}

/// Resolved CloudApps credentials collected from CLI args, environment variables,
/// the OS credential store (macOS Keychain), and credentials.toml.
///
/// Once constructed, the process should unset the CLOUDAPPS_* environment
/// variables so that forked child processes (agent) do not inherit credentials
/// via the environment.
#[derive(Clone, Default)]
pub struct CloudAppsCredentials {
    pub api_url: Option<String>,
    pub api_token: Option<String>,
}

/// Hand-written `Debug` so an accidental `dbg!()` / `{:?}` on this struct
/// does not end up printing the API token into a log or an error message.
impl std::fmt::Debug for CloudAppsCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloudAppsCredentials")
            .field("api_url", &self.api_url)
            .field("api_token", &self.api_token.as_ref().map(|_| "***"))
            .finish()
    }
}

/// Search paths for credentials.toml (highest priority first).
fn credentials_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from(".cloudapps-credentials.toml")];
    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        paths.push(
            PathBuf::from(config_home)
                .join("cloudapps-cli")
                .join("credentials.toml"),
        );
    } else if let Ok(home) = std::env::var("HOME") {
        paths.push(
            PathBuf::from(home)
                .join(".config")
                .join("cloudapps-cli")
                .join("credentials.toml"),
        );
    }
    paths
}

/// Filter empty strings to None so that unfilled template values
/// (e.g. `api_token = ""`) do not bypass validation.
fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.is_empty())
}

/// Load credentials from the first credentials.toml found in the given search paths.
/// Only the first file found is used; subsequent paths are not merged.
fn load_credentials_from_paths(paths: &[PathBuf]) -> CredentialsFile {
    for path in paths {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        match toml::from_str::<CredentialsFileRoot>(&content) {
            Ok(root) => {
                let c = root.credentials;
                return CredentialsFile {
                    api_url: non_empty(c.api_url),
                    api_token: non_empty(c.api_token),
                };
            }
            Err(e) => {
                eprintln!("warning: failed to parse {}: {}", path.display(), e);
            }
        }
    }
    CredentialsFile::default()
}

/// Look up `api_token` in the credential store.
///
/// Returns:
/// - `Ok(Some(value))`: Keychain has an entry, use it.
/// - `Ok(None)`: No entry (or backend unavailable). The resolver may fall
///   through to `credentials.toml`.
/// - `Err(())`: Backend error (e.g. access denied). The resolver must NOT
///   fall through to plaintext; we've already emitted a warning to stderr.
fn lookup_store_token(store: &dyn CredentialStore) -> Result<Option<String>, ()> {
    match store.get(KEY_API_TOKEN) {
        Ok(v) => Ok(v),
        Err(StoreError::Unavailable(_)) => Ok(None),
        Err(StoreError::Backend(msg)) => {
            eprintln!(
                "warning: failed to read api_token from credential store: {}. \
                 Refusing to fall back to credentials.toml so a stale plaintext \
                 value cannot silently mask the intended Keychain secret.",
                msg
            );
            Err(())
        }
    }
}

impl CloudAppsCredentials {
    /// Resolve credentials from CLI args, environment variables, the OS
    /// credential store, and credentials.toml.
    ///
    /// Priority: CLI args > environment variables > OS credential store >
    /// credentials.toml > defaults.
    pub fn resolve(cli_api_url: Option<&str>) -> Self {
        let store = default_store();
        Self::resolve_with_store(cli_api_url, &*store, &credentials_search_paths())
    }

    /// Resolve credentials with an explicit credential store and explicit
    /// search paths for credentials.toml. Exposed so tests can substitute
    /// `MemoryStore`.
    pub fn resolve_with_store(
        cli_api_url: Option<&str>,
        store: &dyn CredentialStore,
        search_paths: &[PathBuf],
    ) -> Self {
        let file = load_credentials_from_paths(search_paths);

        let api_url = cli_api_url
            .map(String::from)
            .or_else(|| non_empty(std::env::var("CLOUDAPPS_API_URL").ok()))
            .or(file.api_url);

        // env > store > toml. A Backend error from the store refuses
        // the toml fallback so a stale plaintext value cannot silently
        // mask the intended Keychain secret.
        let env_token = non_empty(std::env::var("CLOUDAPPS_API_TOKEN").ok());
        let api_token = if let Some(t) = env_token {
            Some(t)
        } else {
            match lookup_store_token(store) {
                Ok(Some(t)) => Some(t),
                Ok(None) => file.api_token,
                Err(()) => None,
            }
        };

        Self { api_url, api_token }
    }

    /// Resolve credentials with explicit search paths for credentials.toml.
    /// Uses the platform-default credential store.
    #[cfg(test)]
    fn resolve_with_paths(cli_api_url: Option<&str>, search_paths: &[PathBuf]) -> Self {
        let store = default_store();
        Self::resolve_with_store(cli_api_url, &*store, search_paths)
    }

    /// Validate that required credentials are present for API access.
    pub fn validate(&self) -> Result<(), String> {
        let mut missing = Vec::new();
        if self.api_url.is_none() {
            missing.push("CLOUDAPPS_API_URL");
        }
        if self.api_token.is_none() {
            missing.push("CLOUDAPPS_API_TOKEN");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "missing required credentials: {}. Set via environment variables, CLI options, credentials.toml, or `cloudapps-cli credentials set api-token`.",
                missing.join(", ")
            ))
        }
    }

    /// Remove CloudApps credential environment variables from the current process.
    /// First overwrites the values in-place (via the C `environ` pointer) so that
    /// the kernel's process environment snapshot — visible through `ps -E` or
    /// `/proc/<pid>/environ` — no longer contains the real secrets.
    /// Then calls `remove_var` to fully unset each variable.
    ///
    /// # Safety
    /// Must be called in a single-threaded context (before tokio runtime creation).
    pub unsafe fn clear_env() {
        for key in &["CLOUDAPPS_API_URL", "CLOUDAPPS_API_TOKEN"] {
            // SAFETY: Caller guarantees single-threaded context.
            // Overwrite the value in the C environ array before removing,
            // so the kernel snapshot no longer contains the real value.
            unsafe {
                overwrite_environ_value(key);
                std::env::remove_var(key);
            }
        }
    }
}

/// Overwrite the value portion of an environment variable in-place with `*`.
/// This mutates the C `environ` array directly so that the kernel's snapshot
/// (read by `ps -E` / `/proc/<pid>/environ`) is scrubbed.
///
/// # Safety
/// Must be called in a single-threaded context. The `environ` pointer and its
/// strings must not be concurrently accessed.
unsafe fn overwrite_environ_value(name: &str) {
    unsafe extern "C" {
        static mut environ: *mut *mut libc::c_char;
    }

    unsafe {
        if environ.is_null() {
            return;
        }

        let name_bytes = name.as_bytes();
        let mut ep = environ;
        while !(*ep).is_null() {
            let entry = *ep;
            // Check if entry starts with "NAME="
            let mut matches = true;
            for (i, &b) in name_bytes.iter().enumerate() {
                if *entry.add(i) as u8 != b {
                    matches = false;
                    break;
                }
            }
            if matches && *entry.add(name_bytes.len()) == b'=' as libc::c_char {
                // Overwrite the value portion with '*'
                let val_start = entry.add(name_bytes.len() + 1);
                let mut p = val_start;
                while *p != 0 {
                    *p = b'*' as libc::c_char;
                    p = p.add(1);
                }
                return;
            }
            ep = ep.add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use credential_store::MemoryStore;
    use std::sync::Mutex;

    /// Mutex to serialize tests that manipulate CLOUDAPPS_* environment variables.
    /// Environment variables are process-global, so concurrent set_var/remove_var
    /// calls from different test threads cause data races.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_resolve_value_priority() {
        // CLI option takes highest priority
        let result = resolve_value(Some("cli"), "NONEXISTENT_VAR");
        assert_eq!(result.as_deref(), Some("cli"));

        // None if nothing is set
        let result = resolve_value(None, "NONEXISTENT_VAR_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_credentials_validate_both_present() {
        let creds = CloudAppsCredentials {
            api_url: Some("https://example.com".to_string()),
            api_token: Some("token".to_string()),
        };
        assert!(creds.validate().is_ok());
    }

    #[test]
    fn test_credentials_validate_missing_all() {
        let creds = CloudAppsCredentials::default();
        let err = creds.validate().unwrap_err();
        assert!(err.contains("CLOUDAPPS_API_URL"));
        assert!(err.contains("CLOUDAPPS_API_TOKEN"));
    }

    #[test]
    fn test_credentials_validate_partial_missing() {
        let creds = CloudAppsCredentials {
            api_url: Some("https://example.com".to_string()),
            api_token: None,
        };
        let err = creds.validate().unwrap_err();
        assert!(!err.contains("CLOUDAPPS_API_URL"));
        assert!(err.contains("CLOUDAPPS_API_TOKEN"));
    }

    #[test]
    fn test_debug_masks_api_token() {
        let creds = CloudAppsCredentials {
            api_url: Some("https://example.com".to_string()),
            api_token: Some("super-secret-token".to_string()),
        };
        let rendered = format!("{:?}", creds);
        assert!(!rendered.contains("super-secret-token"));
        assert!(rendered.contains("***"));
    }

    /// Helper to ensure CLOUDAPPS_* env vars are cleared before tests that call resolve().
    unsafe fn clear_cloudapps_env() {
        unsafe {
            std::env::remove_var("CLOUDAPPS_API_URL");
            std::env::remove_var("CLOUDAPPS_API_TOKEN");
        }
    }

    /// Use resolve_with_store with an empty MemoryStore and empty paths to
    /// isolate tests from real credentials.toml and the real Keychain.
    fn resolve_without_store_or_file(cli_api_url: Option<&str>) -> CloudAppsCredentials {
        let store = MemoryStore::new();
        CloudAppsCredentials::resolve_with_store(cli_api_url, &store, &[])
    }

    #[test]
    fn test_credentials_resolve_cli_overrides_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://env.example.com");
        }
        let creds = resolve_without_store_or_file(Some("https://cli.example.com"));
        assert_eq!(creds.api_url.as_deref(), Some("https://cli.example.com"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_credentials_resolve_env_fallback() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://env.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "env-token");
        }
        let creds = resolve_without_store_or_file(None);
        assert_eq!(creds.api_url.as_deref(), Some("https://env.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("env-token"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_credentials_resolve_empty() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let creds = resolve_without_store_or_file(None);
        assert!(creds.api_url.is_none());
        assert!(creds.api_token.is_none());
    }

    #[test]
    fn test_credentials_clear_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://test.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "test-token");
        }

        unsafe {
            CloudAppsCredentials::clear_env();
        }

        assert!(std::env::var("CLOUDAPPS_API_URL").is_err());
        assert!(std::env::var("CLOUDAPPS_API_TOKEN").is_err());
    }

    #[test]
    fn test_credentials_file_parse_full() {
        let toml_str = r#"
[credentials]
api_url = "https://toml.example.com"
api_token = "toml-token"
"#;
        let root: CredentialsFileRoot = toml::from_str(toml_str).unwrap();
        assert_eq!(
            root.credentials.api_url.as_deref(),
            Some("https://toml.example.com")
        );
        assert_eq!(root.credentials.api_token.as_deref(), Some("toml-token"));
    }

    #[test]
    fn test_credentials_file_parse_minimal() {
        let toml_str = r#"
[credentials]
api_token = "toml-token"
"#;
        let root: CredentialsFileRoot = toml::from_str(toml_str).unwrap();
        assert_eq!(root.credentials.api_token.as_deref(), Some("toml-token"));
        assert!(root.credentials.api_url.is_none());
    }

    #[test]
    fn test_credentials_file_parse_empty() {
        let toml_str = "";
        let root: CredentialsFileRoot = toml::from_str(toml_str).unwrap();
        assert!(root.credentials.api_url.is_none());
        assert!(root.credentials.api_token.is_none());
    }

    #[test]
    fn test_non_empty_filters_empty_strings() {
        assert_eq!(non_empty(Some("".to_string())), None);
        assert_eq!(
            non_empty(Some("value".to_string())),
            Some("value".to_string())
        );
        assert_eq!(non_empty(None), None);
    }

    #[test]
    fn test_resolve_with_toml_file() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(
            &toml_path,
            r#"
[credentials]
api_url = "https://toml.example.com"
api_token = "toml-token"
"#,
        )
        .unwrap();
        let creds = CloudAppsCredentials::resolve_with_paths(None, &[toml_path]);
        assert_eq!(creds.api_url.as_deref(), Some("https://toml.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("toml-token"));
    }

    #[test]
    fn test_env_overrides_toml_file() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(
            &toml_path,
            r#"
[credentials]
api_url = "https://toml.example.com"
api_token = "toml-token"
"#,
        )
        .unwrap();
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://env.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "env-token");
        }
        let creds = CloudAppsCredentials::resolve_with_paths(None, &[toml_path]);
        assert_eq!(creds.api_url.as_deref(), Some("https://env.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("env-token"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_cli_overrides_toml_file() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(
            &toml_path,
            r#"
[credentials]
api_url = "https://toml.example.com"
api_token = "toml-token"
"#,
        )
        .unwrap();
        let creds =
            CloudAppsCredentials::resolve_with_paths(Some("https://cli.example.com"), &[toml_path]);
        assert_eq!(creds.api_url.as_deref(), Some("https://cli.example.com"));
        // api_token has no CLI flag, so TOML value is used
        assert_eq!(creds.api_token.as_deref(), Some("toml-token"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_load_credentials_nonexistent_file() {
        let paths = vec![PathBuf::from("/nonexistent/credentials.toml")];
        let file = load_credentials_from_paths(&paths);
        assert!(file.api_url.is_none());
        assert!(file.api_token.is_none());
    }

    #[test]
    fn test_load_credentials_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(&toml_path, "this is not valid toml {{{{").unwrap();
        let file = load_credentials_from_paths(&[toml_path]);
        assert!(file.api_url.is_none());
        assert!(file.api_token.is_none());
    }

    #[test]
    fn test_first_file_takes_priority() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.toml");
        let second = dir.path().join("second.toml");
        std::fs::write(
            &first,
            r#"
[credentials]
api_url = "https://first.example.com"
api_token = "first-token"
"#,
        )
        .unwrap();
        std::fs::write(
            &second,
            r#"
[credentials]
api_url = "https://second.example.com"
api_token = "second-token"
"#,
        )
        .unwrap();
        let file = load_credentials_from_paths(&[first, second]);
        assert_eq!(file.api_url.as_deref(), Some("https://first.example.com"));
        assert_eq!(file.api_token.as_deref(), Some("first-token"));
    }

    #[test]
    fn test_credentials_resolve_then_clear_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://test.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "test-token");
        }

        let creds = resolve_without_store_or_file(None);
        assert_eq!(creds.api_url.as_deref(), Some("https://test.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("test-token"));

        unsafe {
            CloudAppsCredentials::clear_env();
        }

        // Credentials struct still holds the values
        assert_eq!(creds.api_url.as_deref(), Some("https://test.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("test-token"));

        // But env vars are gone
        assert!(std::env::var("CLOUDAPPS_API_URL").is_err());
        assert!(std::env::var("CLOUDAPPS_API_TOKEN").is_err());
    }

    #[test]
    fn test_store_token_preferred_over_toml() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(
            &toml_path,
            r#"
[credentials]
api_url = "https://toml.example.com"
api_token = "toml-token"
"#,
        )
        .unwrap();
        let store = MemoryStore::new();
        store.set(KEY_API_TOKEN, "store-token").unwrap();
        let creds = CloudAppsCredentials::resolve_with_store(None, &store, &[toml_path]);
        // Store beats toml when env is unset.
        assert_eq!(creds.api_token.as_deref(), Some("store-token"));
        // api_url still resolves from toml (stored credentials only cover api_token).
        assert_eq!(creds.api_url.as_deref(), Some("https://toml.example.com"));
    }

    #[test]
    fn test_env_token_beats_store_token() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        unsafe {
            std::env::set_var("CLOUDAPPS_API_TOKEN", "env-token");
        }
        let store = MemoryStore::new();
        store.set(KEY_API_TOKEN, "store-token").unwrap();
        let creds = CloudAppsCredentials::resolve_with_store(None, &store, &[]);
        assert_eq!(creds.api_token.as_deref(), Some("env-token"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_store_falls_through_to_toml_when_empty() {
        let _lock = ENV_MUTEX.lock().unwrap();
        unsafe { clear_cloudapps_env() };
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("credentials.toml");
        std::fs::write(
            &toml_path,
            r#"
[credentials]
api_token = "toml-token"
"#,
        )
        .unwrap();
        let store = MemoryStore::new();
        let creds = CloudAppsCredentials::resolve_with_store(None, &store, &[toml_path]);
        assert_eq!(creds.api_token.as_deref(), Some("toml-token"));
    }
}
