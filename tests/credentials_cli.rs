#![allow(deprecated)] // Command::cargo_bin is deprecated but cargo_bin_cmd! has different ergonomics

//! End-to-end integration tests for `cloudapps-cli credentials` that do
//! not touch the real Keychain.
//!
//! The Keychain-mutating paths (`credentials set api-token`, successful
//! `credentials delete api-token`, non-dry-run `credentials migrate`)
//! are covered by unit tests in `src/commands/credentials.rs` with a
//! `MemoryStore` double. These integration tests focus on the CLI
//! surface that is observable without touching OS credential storage:
//!
//! 1. `credentials migrate --dry-run` (prints plan, leaves toml intact)
//! 2. `credentials migrate --dry-run` refuses unsupported quote forms
//! 3. `credentials set api-token --stdin` with empty input -> exit 5
//! 4. `credentials --help` mentions every subcommand
//!
//! Isolation strategy:
//! - `HOME` and `XDG_CONFIG_HOME` are redirected to a tempdir per test
//!   so the search path cannot fall back to the real user config.
//! - Tests that depend on cwd (migrate reads `./.cloudapps-credentials.toml`)
//!   acquire a process-wide mutex before swapping `std::env::set_current_dir`,
//!   because `current_dir` is per-process and `cargo test` runs tests
//!   on threads by default.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

/// Serialize tests that mutate `std::env::set_current_dir`. The cwd is
/// a per-process resource, so two threads racing on it would see each
/// other's tempdirs — or worse, find_credentials_toml() could pick up
/// a stale path from a neighboring test.
fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// Build a `Command` for `cloudapps-cli` with HOME / XDG_CONFIG_HOME
/// pinned to the given tempdir so the credentials search path cannot
/// escape into the real user's config.
///
/// We also clear `CLOUDAPPS_*` env vars so the resolver does not see
/// a value the test did not intend.
fn cli_cmd(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("cloudapps-cli").unwrap();
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("CLOUDAPPS_API_URL")
        .env_remove("CLOUDAPPS_API_TOKEN")
        .env_remove("CLOUDAPPS_AGENT_TOKEN")
        .env_remove("CLOUDAPPS_AGENT_SOCKET");
    cmd
}

#[test]
fn credentials_migrate_dry_run_leaves_toml_intact() {
    let _guard = cwd_lock();

    let home = TempDir::new().unwrap();
    let toml_path = home.path().join(".cloudapps-credentials.toml");
    let original = "[credentials]\napi_token = \"abc123\"\napi_url = \"https://example.com\"\n";
    fs::write(&toml_path, original).unwrap();

    // `migrate` finds `./.cloudapps-credentials.toml` via cwd. We chdir
    // into the tempdir for the duration of the child process. Because
    // `assert_cmd::Command` forwards the *current* process cwd to the
    // child by default, we do this under the cwd_lock guard.
    let saved_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(home.path()).unwrap();

    let result = cli_cmd(home.path())
        .args(["credentials", "migrate", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Would write"));

    // Always restore cwd before asserting on the filesystem, so a
    // panic from the assertion does not leave other tests stranded.
    std::env::set_current_dir(&saved_cwd).unwrap();

    // Explicitly drop the assert result so any late panic runs after
    // we've restored cwd.
    drop(result);

    let after = fs::read_to_string(&toml_path).unwrap();
    assert_eq!(
        after, original,
        "--dry-run must not modify the credentials toml"
    );
}

#[test]
fn credentials_migrate_refuses_unsupported_quote_form() {
    let _guard = cwd_lock();

    let home = TempDir::new().unwrap();
    let toml_path = home.path().join(".cloudapps-credentials.toml");
    // Literal (single-quoted) string — extract_api_token refuses this
    // because we do not attempt to re-parse TOML's literal-string rules.
    let original = "[credentials]\napi_token = 'raw'\n";
    fs::write(&toml_path, original).unwrap();

    let saved_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(home.path()).unwrap();

    let result = cli_cmd(home.path())
        .args(["credentials", "migrate", "--dry-run"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("unsupported").or(predicate::str::contains("Unsupported")),
        );

    std::env::set_current_dir(&saved_cwd).unwrap();
    drop(result);

    let after = fs::read_to_string(&toml_path).unwrap();
    assert_eq!(
        after, original,
        "a rejected migrate must not touch the toml"
    );
}

#[test]
fn credentials_set_rejects_empty_stdin() {
    // This test does not touch cwd, so it does not need the lock.
    // HOME is still pinned so the Keychain path never gets exercised —
    // the empty-input check fires before the store is invoked.
    let home = TempDir::new().unwrap();

    cli_cmd(home.path())
        .args(["credentials", "set", "api-token", "--stdin"])
        .write_stdin("") // feed EOF immediately
        .assert()
        // AppError::InvalidInput -> exit code 5 (see src/error.rs).
        .code(5)
        .stderr(predicate::str::contains("empty value"));
}

#[test]
fn credentials_help_mentions_subcommands() {
    // `--help` is clap's built-in. We assert all four subcommands are
    // discoverable from a single `credentials --help` dump so a future
    // refactor that drops one (e.g. `migrate`) is caught here rather
    // than only by a downstream user.
    let home = TempDir::new().unwrap();

    cli_cmd(home.path())
        .args(["credentials", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("migrate"));
}
