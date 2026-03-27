pub mod client;
pub mod handler;
pub mod peer_verify;
pub mod protocol;
pub mod security;
pub mod server;
pub mod session;

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Resolve the socket directory.
///
/// Priority:
/// 1. `dirs::runtime_dir()/cloudapps-agent` (Linux systemd: `/run/user/<uid>/cloudapps-agent`)
/// 2. `std::env::temp_dir()/cloudapps-agent` (macOS: `/var/folders/.../T/cloudapps-agent`, Linux: `/tmp/cloudapps-agent`)
///
/// On macOS, `std::env::temp_dir()` returns a user-specific directory under
/// `/var/folders/` with 0700 permissions, which is more secure than `/tmp`.
fn resolve_socket_dir() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("cloudapps-agent")
}

/// Ensure the socket directory exists with mode 0700.
pub fn ensure_socket_dir() -> std::io::Result<PathBuf> {
    let dir = resolve_socket_dir();
    fs::create_dir_all(&dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
    }

    Ok(dir)
}

/// Resolve the socket path with 3-stage fallback:
/// 1. CLOUDAPPS_AGENT_SOCKET environment variable
/// 2. Auto-discover: if exactly one socket exists in the directory, use it
/// 3. Fallback to default name
pub fn resolve_socket_path() -> PathBuf {
    if let Ok(path) = std::env::var("CLOUDAPPS_AGENT_SOCKET") {
        return PathBuf::from(path);
    }

    let sockets = list_agent_sockets();
    if sockets.len() == 1 {
        return sockets.into_iter().next().unwrap();
    }

    // Multiple or no sockets: fall back to default name.
    resolve_socket_dir().join("cloudapps.sock")
}

/// Generate a PID-based socket path for a new agent instance.
pub fn pid_socket_path(pid: u32) -> PathBuf {
    resolve_socket_dir().join(format!("cloudapps-{}.sock", pid))
}

/// List all agent socket files in the socket directory.
pub fn list_agent_sockets() -> Vec<PathBuf> {
    let dir = resolve_socket_dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        return vec![];
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("cloudapps-") && name.ends_with(".sock"))
        })
        .map(|e| e.path())
        .collect()
}

/// Generate a session token (two UUIDv4 concatenated).
pub fn generate_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// PID file path for a given socket path.
pub fn pid_file_path(socket_path: &std::path::Path) -> PathBuf {
    socket_path.with_extension("pid")
}

/// Write a PID file.
pub fn write_pid_file(path: &std::path::Path, pid: u32) -> std::io::Result<()> {
    fs::write(path, pid.to_string())
}

/// Read a PID from a PID file.
pub fn read_pid_file(path: &std::path::Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Clean up socket and PID files.
pub fn cleanup_files(socket_path: &std::path::Path) {
    let _ = fs::remove_file(socket_path);
    let _ = fs::remove_file(pid_file_path(socket_path));
}

/// Environment variable names that the agent process needs to retain.
/// All other environment variables are removed after fork to prevent
/// leaking secrets (e.g., tokens loaded by `op run --env-file=.env`).
const ENV_WHITELIST: &[&str] = &[
    // Path resolution
    "HOME",
    "PATH",
    "USER",
    "TMPDIR",
    // XDG directories (session file, config, socket)
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_RUNTIME_DIR",
    // HTTP proxy (reqwest)
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
    // TLS certificates
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    // Locale
    "LANG",
    // Debug
    "RUST_LOG",
    "RUST_BACKTRACE",
];

/// Prefix patterns for whitelisted environment variables.
/// Variables whose name starts with any of these prefixes are retained.
const ENV_WHITELIST_PREFIXES: &[&str] = &[
    "LC_", // locale categories (LC_ALL, LC_CTYPE, etc.)
];

/// Remove all environment variables except those in the whitelist.
/// Called after fork() (or in foreground mode after Config is built)
/// to prevent leaking secrets from the parent process environment.
///
/// See ADR-0004 for the design rationale.
pub fn sanitize_env() {
    let vars_to_remove: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| !is_env_whitelisted(k))
        .collect();

    let count = vars_to_remove.len();
    for key in &vars_to_remove {
        // SAFETY: remove_var modifies the libc environ pointer.
        // This is safe because we are single-threaded at this point
        // (called before tokio runtime creation).
        unsafe {
            std::env::remove_var(key);
        }
    }

    if count > 0 && is_debug() {
        eprintln!("agent: sanitized environment ({} variables removed)", count);
    }
}

/// Check if an environment variable name is in the whitelist.
fn is_env_whitelisted(name: &str) -> bool {
    if ENV_WHITELIST.contains(&name) {
        return true;
    }
    for prefix in ENV_WHITELIST_PREFIXES {
        if name.starts_with(prefix) {
            return true;
        }
    }
    false
}

/// Apply OS-level process hardening to prevent credential leakage.
///
/// - Linux: `prctl(PR_SET_DUMPABLE, 0)` restricts `/proc/[pid]/environ` access.
/// - macOS: `ptrace(PT_DENY_ATTACH)` prevents debugger attachment.
/// - Both: `setrlimit(RLIMIT_CORE, 0)` disables core dumps.
///
/// Errors are logged but not fatal (e.g., restricted container environments).
pub fn harden_process() {
    harden_process_os();
    disable_core_dump();
}

#[cfg(target_os = "linux")]
fn harden_process_os() {
    // SAFETY: prctl(PR_SET_DUMPABLE, 0) is safe; it only affects the calling process.
    let ret = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) };
    if ret != 0 {
        eprintln!(
            "agent: prctl(PR_SET_DUMPABLE, 0) failed: {}",
            std::io::Error::last_os_error()
        );
    } else if is_debug() {
        eprintln!("agent: process hardened (PR_SET_DUMPABLE=0)");
    }
}

#[cfg(target_os = "macos")]
fn harden_process_os() {
    // SAFETY: ptrace(PT_DENY_ATTACH) is safe; it only affects the calling process.
    let ret = unsafe { libc::ptrace(libc::PT_DENY_ATTACH, 0, std::ptr::null_mut(), 0) };
    if ret != 0 {
        eprintln!(
            "agent: ptrace(PT_DENY_ATTACH) failed: {}",
            std::io::Error::last_os_error()
        );
    } else if is_debug() {
        eprintln!("agent: process hardened (PT_DENY_ATTACH)");
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn harden_process_os() {
    // No OS-specific hardening available.
}

fn disable_core_dump() {
    // SAFETY: setrlimit is safe; it only affects the calling process.
    let rl = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rl) };
    if ret != 0 {
        eprintln!(
            "agent: setrlimit(RLIMIT_CORE, 0) failed: {}",
            std::io::Error::last_os_error()
        );
    } else if is_debug() {
        eprintln!("agent: core dumps disabled");
    }
}

/// Validate that required credentials are available before starting the agent.
/// Delegates to `CloudAppsCredentials::validate()`.
pub fn validate_credentials(
    credentials: &crate::config::CloudAppsCredentials,
) -> Result<(), String> {
    credentials.validate()
}

/// Check if debug logging is enabled via RUST_LOG environment variable.
/// This must be called before sanitize_env() clears the environment,
/// or use the cached result.
fn is_debug() -> bool {
    // RUST_LOG is in the whitelist, so it survives sanitize_env().
    std::env::var("RUST_LOG").is_ok()
}

#[cfg(test)]
mod env_tests {
    use super::*;

    #[test]
    fn test_is_env_whitelisted_exact_match() {
        assert!(is_env_whitelisted("HOME"));
        assert!(is_env_whitelisted("PATH"));
        assert!(is_env_whitelisted("RUST_LOG"));
        assert!(is_env_whitelisted("SSL_CERT_FILE"));
        assert!(is_env_whitelisted("http_proxy"));
    }

    #[test]
    fn test_is_env_whitelisted_prefix_match() {
        assert!(is_env_whitelisted("LC_ALL"));
        assert!(is_env_whitelisted("LC_CTYPE"));
        assert!(is_env_whitelisted("LC_MESSAGES"));
    }

    #[test]
    fn test_is_env_whitelisted_rejects_secrets() {
        assert!(!is_env_whitelisted("CLOUDAPPS_API_TOKEN"));
        assert!(!is_env_whitelisted("CLOUDAPPS_API_URL"));
        assert!(!is_env_whitelisted("GITHUB_TOKEN"));
        assert!(!is_env_whitelisted("SLACK_BOT_TOKEN"));
        assert!(!is_env_whitelisted("AWS_SECRET_ACCESS_KEY"));
        assert!(!is_env_whitelisted("DATABASE_URL"));
    }

    #[test]
    fn test_sanitize_env_removes_non_whitelisted() {
        // Set a test variable that is not in the whitelist.
        let key = "CLOUDAPPS_TEST_SANITIZE_SECRET_12345";
        unsafe {
            std::env::set_var(key, "should-be-removed");
        }
        assert!(std::env::var(key).is_ok());

        sanitize_env();

        assert!(
            std::env::var(key).is_err(),
            "non-whitelisted variable should be removed"
        );
    }

    #[test]
    fn test_sanitize_env_keeps_whitelisted() {
        // HOME should survive sanitization.
        if std::env::var("HOME").is_ok() {
            sanitize_env();
            assert!(
                std::env::var("HOME").is_ok(),
                "HOME should be retained after sanitize_env"
            );
        }
    }

    #[test]
    fn test_sanitize_env_removes_cloudapps_credentials() {
        let key = "CLOUDAPPS_API_TOKEN";
        unsafe {
            std::env::set_var(key, "should-be-removed");
        }
        assert!(std::env::var(key).is_ok());

        sanitize_env();

        assert!(
            std::env::var(key).is_err(),
            "CLOUDAPPS_API_TOKEN should be removed by sanitize_env"
        );
    }

    #[test]
    fn test_validate_credentials_delegates_to_cloudapps_credentials() {
        let creds = crate::config::CloudAppsCredentials {
            api_url: Some("https://example.com".to_string()),
            api_token: Some("token".to_string()),
        };
        assert!(validate_credentials(&creds).is_ok());

        let empty = crate::config::CloudAppsCredentials::default();
        assert!(validate_credentials(&empty).is_err());
    }
}
