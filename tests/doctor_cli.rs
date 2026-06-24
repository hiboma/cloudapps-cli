#![allow(deprecated)] // Command::cargo_bin is deprecated but cargo_bin_cmd! has different ergonomics

//! End-to-end integration tests for `cloudapps-cli doctor`.
//!
//! `doctor` is a read-only diagnostic command. These tests exercise the
//! observable CLI surface without touching the network: every invocation
//! passes `--no-connectivity` so no HTTP request is sent.
//!
//! Isolation strategy mirrors `credentials_cli.rs`:
//! - `HOME` / `XDG_CONFIG_HOME` are pinned to a tempdir so the credentials
//!   search path cannot fall back to the real user config.
//! - `CLOUDAPPS_*` env vars are cleared unless a test sets them explicitly,
//!   so the ENVIRONMENT and CREDENTIALS sections reflect only the test's
//!   intent.
//!
//! The security invariant under test: the token VALUE never appears in the
//! output, only its presence and source.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;

/// Build a `cloudapps-cli` command with HOME / XDG_CONFIG_HOME pinned to a
/// tempdir and all `CLOUDAPPS_*` vars cleared.
fn cli_cmd(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("cloudapps-cli").unwrap();
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env_remove("CLOUDAPPS_API_URL")
        .env_remove("CLOUDAPPS_API_TOKEN")
        .env_remove("CLOUDAPPS_OUTPUT_FORMAT")
        .env_remove("CLOUDAPPS_AGENT_TOKEN")
        .env_remove("CLOUDAPPS_AGENT_SOCKET");
    cmd
}

#[test]
fn doctor_prints_all_sections() {
    let home = TempDir::new().unwrap();
    cli_cmd(home.path())
        .args(["doctor", "--no-connectivity"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CONFIG"))
        .stdout(predicate::str::contains("CREDENTIALS"))
        .stdout(predicate::str::contains("ENVIRONMENT"))
        .stdout(predicate::str::contains("cloudapps-cli "));
}

#[test]
fn doctor_reports_env_vars_as_set_unset() {
    let home = TempDir::new().unwrap();
    cli_cmd(home.path())
        .args(["doctor", "--no-connectivity"])
        .assert()
        .success()
        // With CLOUDAPPS_API_URL cleared, the env line must read (unset).
        .stdout(predicate::str::contains(
            "CLOUDAPPS_API_URL          (unset)",
        ));
}

#[test]
fn doctor_never_prints_token_value() {
    let home = TempDir::new().unwrap();
    let secret = "super-secret-token-do-not-print";
    cli_cmd(home.path())
        .env("CLOUDAPPS_API_TOKEN", secret)
        .env("CLOUDAPPS_API_URL", "https://example.invalid")
        .args(["doctor", "--no-connectivity"])
        .assert()
        .success()
        // Presence + source are shown, but never the value itself.
        .stdout(predicate::str::contains(
            "api-token:  set  (source: env CLOUDAPPS_API_TOKEN)",
        ))
        .stdout(predicate::str::contains(secret).not())
        // The env var must read (set), again without echoing the value.
        .stdout(predicate::str::contains("CLOUDAPPS_API_TOKEN        (set)"));
}

#[test]
fn doctor_attributes_cli_flag_over_env() {
    let home = TempDir::new().unwrap();
    cli_cmd(home.path())
        .env("CLOUDAPPS_API_URL", "https://env.example")
        .args([
            "doctor",
            "--no-connectivity",
            "--api-url",
            "https://cli.example",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "api-url:    https://cli.example  (source: cli --api-url)",
        ));
}

#[test]
fn doctor_attributes_env_when_no_flag() {
    let home = TempDir::new().unwrap();
    cli_cmd(home.path())
        .env("CLOUDAPPS_API_URL", "https://env.example")
        .args(["doctor", "--no-connectivity"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "api-url:    https://env.example  (source: env CLOUDAPPS_API_URL)",
        ));
}

#[test]
fn doctor_help_mentions_connectivity_flag() {
    let home = TempDir::new().unwrap();
    cli_cmd(home.path())
        .args(["doctor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-connectivity"));
}
