/// Resolve a value from CLI option or environment variable (in priority order).
pub fn resolve_value(cli_value: Option<&str>, env_var: &str) -> Option<String> {
    cli_value
        .map(String::from)
        .or_else(|| std::env::var(env_var).ok())
}

/// Resolved CloudApps credentials collected from CLI args and environment variables.
/// Once constructed, the process should unset the CLOUDAPPS_* environment variables so that
/// forked child processes (agent) do not inherit credentials via the environment.
#[derive(Debug, Clone, Default)]
pub struct CloudAppsCredentials {
    pub api_url: Option<String>,
    pub api_token: Option<String>,
}

impl CloudAppsCredentials {
    /// Resolve credentials from CLI args and environment variables.
    /// Priority: CLI args > environment variables.
    pub fn resolve(cli_api_url: Option<&str>) -> Self {
        let api_url = cli_api_url
            .map(String::from)
            .or_else(|| std::env::var("CLOUDAPPS_API_URL").ok());

        let api_token = std::env::var("CLOUDAPPS_API_TOKEN").ok();

        Self { api_url, api_token }
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
                "missing required credentials: {}. Set via environment variables or CLI options.",
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

    /// Helper to ensure CLOUDAPPS_* env vars are cleared before tests that call resolve().
    unsafe fn clear_cloudapps_env() {
        unsafe {
            std::env::remove_var("CLOUDAPPS_API_URL");
            std::env::remove_var("CLOUDAPPS_API_TOKEN");
        }
    }

    #[test]
    fn test_credentials_resolve_cli_overrides_env() {
        unsafe { clear_cloudapps_env() };
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://env.example.com");
        }
        let creds = CloudAppsCredentials::resolve(Some("https://cli.example.com"));
        assert_eq!(creds.api_url.as_deref(), Some("https://cli.example.com"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_credentials_resolve_env_fallback() {
        unsafe { clear_cloudapps_env() };
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://env.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "env-token");
        }
        let creds = CloudAppsCredentials::resolve(None);
        assert_eq!(creds.api_url.as_deref(), Some("https://env.example.com"));
        assert_eq!(creds.api_token.as_deref(), Some("env-token"));
        unsafe { clear_cloudapps_env() };
    }

    #[test]
    fn test_credentials_resolve_empty() {
        unsafe { clear_cloudapps_env() };
        let creds = CloudAppsCredentials::resolve(None);
        assert!(creds.api_url.is_none());
        assert!(creds.api_token.is_none());
    }

    #[test]
    fn test_credentials_clear_env() {
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
    fn test_credentials_resolve_then_clear_env() {
        unsafe {
            std::env::set_var("CLOUDAPPS_API_URL", "https://test.example.com");
            std::env::set_var("CLOUDAPPS_API_TOKEN", "test-token");
        }

        let creds = CloudAppsCredentials::resolve(None);
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
}
