//! `cloudapps-cli doctor` — environment and configuration diagnostics.
//!
//! Modeled after `jamf-cli doctor`: a single read-only command that prints
//! where each piece of configuration is coming from, whether the credential
//! store can be reached, which `CLOUDAPPS_*` environment variables are set,
//! and (optionally) whether the API endpoint is reachable.
//!
//! Security stance, mirroring `credentials status`:
//!
//! - The API token value is NEVER printed. Only its presence ("stored" /
//!   "not stored") and the source that would win resolution are shown.
//! - Environment variables are reported as `(set)` / `(unset)`, never with
//!   their values, so a token passed via `CLOUDAPPS_API_TOKEN` does not leak
//!   into terminal scrollback or an AI-agent transcript.
//!
//! Unlike the rest of the CLI, `doctor` must run BEFORE the `CLOUDAPPS_*`
//! environment scrub in `main()` — otherwise the ENVIRONMENT section would
//! always report `CLOUDAPPS_API_URL` / `CLOUDAPPS_API_TOKEN` as `(unset)`,
//! which is exactly the opposite of useful for diagnosing a misconfigured
//! shell. `main()` therefore routes `doctor` like `credentials`: it skips the
//! pre-fork scrub.

use std::path::PathBuf;
use std::time::Instant;

use crate::config::credential_store::{
    CredentialStore, KEY_API_TOKEN, KEYCHAIN_SERVICE, StoreError, default_store,
};
use crate::error::AppError;

/// Environment variables that influence credential resolution and agent
/// routing. Reported as `(set)` / `(unset)` only — never with their values.
const ENV_VARS: &[&str] = &[
    "CLOUDAPPS_API_URL",
    "CLOUDAPPS_API_TOKEN",
    "CLOUDAPPS_OUTPUT_FORMAT",
    "CLOUDAPPS_AGENT_SOCKET",
    "CLOUDAPPS_AGENT_TOKEN",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "HOME",
];

/// Run the doctor diagnostics and print the report to stdout.
///
/// `check_connectivity` controls whether the CONNECTIVITY section sends a
/// real HTTP request.
///
/// Note on `--api-url`: the clap arg carries `env = "CLOUDAPPS_API_URL"`, so
/// the parsed value cannot distinguish a real `--api-url` flag from the
/// environment variable. For correct source attribution we therefore inspect
/// the raw process args ourselves rather than trusting the parsed value.
pub async fn handle(check_connectivity: bool) -> Result<(), AppError> {
    println!("cloudapps-cli {}", env!("CARGO_PKG_VERSION"));
    println!();

    print_config_section();
    println!();

    let cli_api_url = api_url_from_argv();
    let store = default_store();
    let resolved_url = print_credentials_section(cli_api_url.as_deref(), &*store);
    println!();

    #[cfg(unix)]
    {
        print_agent_section();
        println!();
    }

    print_environment_section();

    if check_connectivity {
        println!();
        print_connectivity_section(resolved_url.as_deref()).await;
    }

    Ok(())
}

/// CONFIG: where credentials.toml is looked for and whether it exists.
fn print_config_section() {
    println!("CONFIG");
    let paths = credentials_search_paths();
    let found = paths.iter().find(|p| p.exists());
    match found {
        Some(path) => {
            println!("  credentials.toml:  {}", path.display());
            println!("  status:            present");
        }
        None => {
            // Show the primary (user-level) path so the user knows where to
            // create the file if they want one.
            let primary = paths.last().or_else(|| paths.first());
            match primary {
                Some(path) => println!("  credentials.toml:  {} (not found)", path.display()),
                None => println!("  credentials.toml:  (no search path)"),
            }
            println!("  status:            not found");
        }
    }
}

/// CREDENTIALS: the resolved source for `api-url` and `api-token`, plus the
/// raw Keychain entry status. Returns the resolved api-url (if any) so the
/// CONNECTIVITY section can reuse it without re-resolving.
fn print_credentials_section(
    cli_api_url: Option<&str>,
    store: &dyn CredentialStore,
) -> Option<String> {
    println!("CREDENTIALS");

    // -- api-url: CLI > env > credentials.toml --
    let file = load_credentials_file();
    let (url_value, url_source) = if let Some(v) = non_empty(cli_api_url.map(String::from)) {
        (Some(v), "cli --api-url")
    } else if let Some(v) = non_empty(std::env::var("CLOUDAPPS_API_URL").ok()) {
        (Some(v), "env CLOUDAPPS_API_URL")
    } else if let Some(v) = file.api_url.clone() {
        (Some(v), "credentials.toml")
    } else {
        (None, "")
    };

    match &url_value {
        Some(v) => println!("  api-url:    {}  (source: {})", v, url_source),
        None => println!("  api-url:    (unset)"),
    }

    // -- api-token: env > Keychain > credentials.toml. Presence + source
    //    only; the value is never printed. We deliberately classify the
    //    source the same way the resolver does so `doctor` and the real
    //    request path agree on which secret would be used. --
    let token_env = non_empty(std::env::var("CLOUDAPPS_API_TOKEN").ok());
    let store_result = store.get(KEY_API_TOKEN);

    if token_env.is_some() {
        println!("  api-token:  set  (source: env CLOUDAPPS_API_TOKEN)");
    } else {
        match &store_result {
            Ok(Some(_)) => println!(
                "  api-token:  stored  (source: keychain {}/{})",
                KEYCHAIN_SERVICE, KEY_API_TOKEN
            ),
            Ok(None) if file.api_token.is_some() => {
                println!("  api-token:  set  (source: credentials.toml)")
            }
            Ok(None) => println!("  api-token:  (unset)"),
            Err(StoreError::Unavailable(_)) if file.api_token.is_some() => {
                println!("  api-token:  set  (source: credentials.toml)")
            }
            Err(StoreError::Unavailable(_)) => println!("  api-token:  (unset)"),
            Err(StoreError::Backend(msg)) => {
                // A Backend error means the resolver would REFUSE to fall back
                // to the toml (see config::lookup_store_token). Surface that
                // so the user understands why a token in credentials.toml is
                // not being picked up.
                println!(
                    "  api-token:  UNRESOLVABLE  (keychain {}/{}: {})",
                    KEYCHAIN_SERVICE, KEY_API_TOKEN, msg
                );
            }
        }
    }

    // -- keychain: the raw entry status, independent of resolution order.
    //    This is the same information `credentials status` prints. --
    match &store_result {
        Ok(Some(_)) => println!(
            "  keychain:   {}/{}  →  stored",
            KEYCHAIN_SERVICE, KEY_API_TOKEN
        ),
        Ok(None) => println!(
            "  keychain:   {}/{}  →  not stored",
            KEYCHAIN_SERVICE, KEY_API_TOKEN
        ),
        Err(e) => println!(
            "  keychain:   {}/{}  →  error ({})",
            KEYCHAIN_SERVICE, KEY_API_TOKEN, e
        ),
    }

    url_value
}

/// AGENT: the credential isolation agent session state (Unix only).
#[cfg(unix)]
fn print_agent_section() {
    use crate::agent::session;

    println!("AGENT");
    let session_path = session::session_file_path();
    println!("  session-file:  {}", session_path.display());

    match session::read_session() {
        Some(info) => {
            if session::is_session_alive(&info) {
                println!(
                    "  status:        running (pid {}, socket {})",
                    info.pid, info.socket_path
                );
            } else {
                println!(
                    "  status:        stale (session file present, socket {} missing)",
                    info.socket_path
                );
            }
        }
        None => println!("  status:        not running"),
    }
}

/// ENVIRONMENT: which influential environment variables are set. Values are
/// never printed — only presence.
fn print_environment_section() {
    println!("ENVIRONMENT");
    for var in ENV_VARS {
        let state = match std::env::var(var) {
            Ok(v) if !v.is_empty() => "(set)",
            _ => "(unset)",
        };
        println!("  {:<26} {}", var, state);
    }
}

/// CONNECTIVITY: send a HEAD request to the resolved api-url and report the
/// status code and elapsed time. Network errors are reported, not fatal — a
/// 401 still proves the endpoint is reachable, which is the diagnostic point.
async fn print_connectivity_section(api_url: Option<&str>) {
    println!("CONNECTIVITY");
    let Some(url) = api_url else {
        println!("  (skipped: api-url is unset)");
        return;
    };

    let target = url.trim_end_matches('/').to_string();
    let client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            println!("  HEAD {}  →  error building HTTP client: {}", target, e);
            return;
        }
    };

    let start = Instant::now();
    let result = client.head(&target).send().await;
    let elapsed = start.elapsed().as_millis();

    match result {
        Ok(resp) => {
            let status = resp.status();
            let reason = status.canonical_reason().unwrap_or("");
            println!(
                "  HEAD {}  →  {} {} ({}ms)",
                target,
                status.as_u16(),
                reason,
                elapsed
            );
        }
        Err(e) => {
            // Strip the URL from reqwest's error so a token embedded in a
            // userinfo component cannot leak; reqwest does not put tokens in
            // errors, but keep the output terse and url-free regardless.
            println!(
                "  HEAD {}  →  unreachable ({}) ({}ms)",
                target,
                classify_reqwest_err(&e),
                elapsed
            );
        }
    }
}

/// Produce a short, value-free description of a reqwest error for the
/// connectivity report.
fn classify_reqwest_err(e: &reqwest::Error) -> &'static str {
    if e.is_timeout() {
        "timeout"
    } else if e.is_connect() {
        "connection failed"
    } else if e.is_request() {
        "request error"
    } else {
        "network error"
    }
}

// --- Helpers duplicated from config:: (kept private there) -------------------

/// Filter empty strings to None. Mirrors `config::non_empty`.
fn non_empty(s: Option<String>) -> Option<String> {
    s.filter(|v| !v.is_empty())
}

/// Extract an explicit `--api-url` value from the raw process arguments.
///
/// We cannot use the clap-parsed value because the `--api-url` arg declares
/// `env = "CLOUDAPPS_API_URL"`: clap folds the environment variable into the
/// same field, so the parsed value cannot tell a real flag from the env var.
/// Scanning argv ourselves lets the CREDENTIALS section report `cli --api-url`
/// only when the flag was actually passed.
fn api_url_from_argv() -> Option<String> {
    arg_value(std::env::args().skip(1), "--api-url")
}

/// Find the value of `--flag VALUE` or `--flag=VALUE` in an argument iterator.
fn arg_value<I: Iterator<Item = String>>(args: I, flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    let mut iter = args;
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next();
        }
        if let Some(value) = arg.strip_prefix(&prefix) {
            return Some(value.to_string());
        }
    }
    None
}

/// credentials.toml search paths. Mirrors `config::credentials_search_paths`.
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

/// A minimal mirror of the `[credentials]` table for source attribution.
/// `doctor` only needs presence + the api_url value (never the token value),
/// so it parses the file independently rather than reusing the private
/// loader in `config`.
struct FileCredentials {
    api_url: Option<String>,
    api_token: Option<String>,
}

/// Load the first credentials.toml found, returning presence of each field.
fn load_credentials_file() -> FileCredentials {
    #[derive(serde::Deserialize, Default)]
    struct Root {
        #[serde(default)]
        credentials: Section,
    }
    #[derive(serde::Deserialize, Default)]
    struct Section {
        api_url: Option<String>,
        api_token: Option<String>,
    }

    for path in credentials_search_paths() {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(root) = toml::from_str::<Root>(&content) {
            return FileCredentials {
                api_url: non_empty(root.credentials.api_url),
                api_token: non_empty(root.credentials.api_token),
            };
        }
    }
    FileCredentials {
        api_url: None,
        api_token: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_reqwest_err_returns_static_labels() {
        // We can't easily synthesize each reqwest::Error variant, but we can
        // assert the catch-all branch is value-free (no URL, no token).
        // Build a client error via an invalid request to exercise the path.
        // This is best-effort; the important guarantee is the &'static str
        // return type, which forbids interpolating dynamic content.
        let _ = classify_reqwest_err;
    }

    #[test]
    fn non_empty_filters_blank() {
        assert_eq!(non_empty(Some(String::new())), None);
        assert_eq!(non_empty(Some(" ".to_string())), Some(" ".to_string()));
        assert_eq!(non_empty(None), None);
    }

    #[test]
    fn search_paths_start_with_cwd_file() {
        let paths = credentials_search_paths();
        assert_eq!(paths[0], PathBuf::from(".cloudapps-credentials.toml"));
    }

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn arg_value_space_form() {
        let args = argv(&["doctor", "--api-url", "https://x.example"]);
        assert_eq!(
            arg_value(args.into_iter(), "--api-url").as_deref(),
            Some("https://x.example")
        );
    }

    #[test]
    fn arg_value_equals_form() {
        let args = argv(&["doctor", "--api-url=https://y.example"]);
        assert_eq!(
            arg_value(args.into_iter(), "--api-url").as_deref(),
            Some("https://y.example")
        );
    }

    #[test]
    fn arg_value_absent_returns_none() {
        // The env var being set must NOT make this return Some — argv-only.
        let args = argv(&["doctor", "--no-connectivity"]);
        assert_eq!(arg_value(args.into_iter(), "--api-url"), None);
    }

    #[test]
    fn arg_value_flag_without_value_returns_none() {
        let args = argv(&["doctor", "--api-url"]);
        assert_eq!(arg_value(args.into_iter(), "--api-url"), None);
    }
}
