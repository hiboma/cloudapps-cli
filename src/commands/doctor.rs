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

use std::time::Instant;

use crate::config::credential_store::{
    CredentialStore, KEY_API_TOKEN, KEYCHAIN_SERVICE, default_store,
};
use crate::config::{
    CredentialSource, api_token_source, api_url_source, credentials_search_paths,
    load_credentials_from_paths,
};
use crate::error::AppError;

/// Human-readable label for a resolved credential source.
fn source_label(source: CredentialSource) -> &'static str {
    match source {
        CredentialSource::CliFlag => "cli --api-url",
        CredentialSource::Env => "env",
        CredentialSource::Keychain => "keychain",
        CredentialSource::File => "credentials.toml",
    }
}

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
    // Reuse the resolver's own search paths so CONFIG can never disagree
    // with where credentials actually load from.
    let paths = credentials_search_paths();
    match paths.iter().find(|p| p.exists()) {
        Some(path) => {
            println!("  credentials.toml:  {}", path.display());
            println!("  status:            present");
        }
        None => {
            // Print every candidate path rather than guessing a single
            // "primary" one: the last entry is the user-level config only
            // when HOME / XDG_CONFIG_HOME is set; otherwise the only path is
            // the cwd-relative `.cloudapps-credentials.toml`, and labeling
            // that "user-level" would mislead.
            match paths.as_slice() {
                [] => println!("  credentials.toml:  (no search path)"),
                [single] => println!("  credentials.toml:  {} (not found)", single.display()),
                many => {
                    println!("  credentials.toml:  (not found; searched)");
                    for p in many {
                        println!("                     {}", p.display());
                    }
                }
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

    // Reuse the resolver's loader (it emits the same `warning: failed to
    // parse ...` a real request would) and its source-attribution helpers,
    // so doctor reports exactly the source the request path would use.
    let file = load_credentials_from_paths(&credentials_search_paths());

    // -- api-url: CLI > env > credentials.toml --
    let resolved_url = api_url_source(cli_api_url, &file);
    match &resolved_url {
        Some((v, CredentialSource::Env)) => {
            println!("  api-url:    {}  (source: env CLOUDAPPS_API_URL)", v)
        }
        Some((v, source)) => println!("  api-url:    {}  (source: {})", v, source_label(*source)),
        None => println!("  api-url:    (unset)"),
    }

    // -- api-token: env > Keychain > credentials.toml. Presence + source
    //    only; the value is never printed. The store is read exactly once
    //    (a single Keychain authorization) and the result is reused for both
    //    the source classification and the raw status line below. --
    let store_result = store.get(KEY_API_TOKEN);
    match api_token_source(&store_result, &file) {
        Ok(Some(CredentialSource::Env)) => {
            println!("  api-token:  set  (source: env CLOUDAPPS_API_TOKEN)")
        }
        Ok(Some(CredentialSource::Keychain)) => println!(
            "  api-token:  stored  (source: keychain {}/{})",
            KEYCHAIN_SERVICE, KEY_API_TOKEN
        ),
        Ok(Some(CredentialSource::File)) => {
            println!("  api-token:  set  (source: credentials.toml)")
        }
        Ok(Some(CredentialSource::CliFlag)) => {
            // No CLI flag carries the token; unreachable, but keep the match
            // exhaustive without an unlabeled catch-all.
            println!("  api-token:  set")
        }
        Ok(None) => println!("  api-token:  (unset)"),
        Err(()) => {
            // A Backend store error refuses the toml fallback (see
            // config::lookup_store_token). Surface it so the user understands
            // why a token in credentials.toml is not being picked up; the raw
            // keychain line below carries the underlying error message.
            println!(
                "  api-token:  UNRESOLVABLE  (keychain {}/{} access failed)",
                KEYCHAIN_SERVICE, KEY_API_TOKEN
            );
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

    resolved_url.map(|(v, _)| v)
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
    // Bound the probe so an unreachable host fails fast with a `timeout`
    // classification instead of stalling on the OS TCP timeout (tens of
    // seconds). A diagnostic command must return quickly to be useful.
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn arg_value_equals_empty_is_some_empty() {
        // `--api-url=` yields Some("") from argv. `api_url_source` then runs
        // it through `non_empty`, so the CREDENTIALS section falls through to
        // env / toml rather than attributing an empty CLI flag — matching the
        // resolver, which now also filters the CLI value through `non_empty`.
        let args = argv(&["doctor", "--api-url="]);
        assert_eq!(
            arg_value(args.into_iter(), "--api-url").as_deref(),
            Some("")
        );
    }
}
