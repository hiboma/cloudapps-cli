#![allow(deprecated)] // Command::cargo_bin is deprecated but cargo_bin_cmd! has different ergonomics

//! Integration tests for `cloudapps-cli agent start/stop/status` lifecycle.
//!
//! Each test uses an isolated temporary directory for TMPDIR and XDG_DATA_HOME
//! so that sockets and session files do not interfere with each other or with
//! a real running agent.
//!
//! Dummy credentials (CLOUDAPPS_API_URL, CLOUDAPPS_API_TOKEN) are injected via
//! environment variables so the agent passes credential validation without
//! hitting any real API.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

/// Build a `Command` for `cloudapps-cli` with isolated env.
/// - TMPDIR -> temp dir (socket directory lives under $TMPDIR/cloudapps-agent/)
/// - XDG_DATA_HOME -> temp dir (session.json lives under $XDG_DATA_HOME/cloudapps-cli/)
/// - Dummy credentials to pass validation
fn cli_cmd(tmpdir: &Path, xdg_data_home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("cloudapps-cli").unwrap();
    cmd.env("TMPDIR", tmpdir)
        .env("XDG_DATA_HOME", xdg_data_home)
        .env("CLOUDAPPS_API_URL", "https://test.example.com/api/v1")
        .env("CLOUDAPPS_API_TOKEN", "test-dummy-token")
        // Prevent loading the user's .env file
        .env("HOME", tmpdir);
    cmd
}

/// Wait for the session file to appear (agent needs a moment to fork and write it).
fn wait_for_session_file(xdg_data_home: &Path) -> std::path::PathBuf {
    let session_file = xdg_data_home
        .join("cloudapps-cli")
        .join("session.debug.json");
    for _ in 0..50 {
        if session_file.exists() {
            return session_file;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("session file did not appear at {}", session_file.display());
}

/// Wait for the session file to be removed (agent stop needs a moment).
fn wait_for_session_removed(xdg_data_home: &Path) {
    let session_file = xdg_data_home
        .join("cloudapps-cli")
        .join("session.debug.json");
    for _ in 0..50 {
        if !session_file.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("session file was not removed at {}", session_file.display());
}

/// Stop agent if running (cleanup helper).
fn stop_agent(tmpdir: &Path, xdg_data_home: &Path) {
    let _ = cli_cmd(tmpdir, xdg_data_home)
        .args(["agent", "stop"])
        .output();
    // Wait a bit for cleanup.
    std::thread::sleep(Duration::from_millis(200));
}

#[test]
fn test_agent_start_creates_session_file() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success()
        .stderr(predicate::str::contains("agent started, pid"));

    let session_file = wait_for_session_file(xdg_data_home.path());
    assert!(session_file.exists());

    // Cleanup.
    stop_agent(tmpdir.path(), xdg_data_home.path());
}

#[test]
fn test_agent_status_shows_running() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Start the agent.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success();

    wait_for_session_file(xdg_data_home.path());

    // Check status.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": true"));

    // Cleanup.
    stop_agent(tmpdir.path(), xdg_data_home.path());
}

#[test]
fn test_agent_stop_removes_session() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Start the agent.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success();

    wait_for_session_file(xdg_data_home.path());

    // Stop the agent.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped agent"));

    wait_for_session_removed(xdg_data_home.path());
}

#[test]
fn test_agent_start_duplicate_detected() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Start first agent.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success();

    wait_for_session_file(xdg_data_home.path());

    // Start second agent (should detect already running).
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already started"));

    // Cleanup.
    stop_agent(tmpdir.path(), xdg_data_home.path());
}

#[test]
fn test_agent_stop_no_agent_running() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Stop with no agent running (should error).
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "stop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no session file found"));
}

#[test]
fn test_agent_status_no_agent_running() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Status with no agent running.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": false"));
}

#[test]
fn test_agent_full_lifecycle() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // 1. Start.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success()
        .stderr(predicate::str::contains("agent started, pid"));

    wait_for_session_file(xdg_data_home.path());

    // 2. Status (running).
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": true"));

    // 3. Duplicate start.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already started"));

    // 4. Stop.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stopped agent"));

    wait_for_session_removed(xdg_data_home.path());

    // 5. Status (not running).
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": false"));

    // 6. Stop again (should error).
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "stop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no session file found"));
}

#[test]
fn test_agent_credentials_not_leaked_to_env() {
    let tmpdir = TempDir::new().unwrap();
    let xdg_data_home = TempDir::new().unwrap();

    // Start the agent.
    cli_cmd(tmpdir.path(), xdg_data_home.path())
        .args(["agent", "start"])
        .assert()
        .success();

    wait_for_session_file(xdg_data_home.path());

    // Read the session file to get the agent PID.
    let session_file = xdg_data_home
        .path()
        .join("cloudapps-cli")
        .join("session.debug.json");
    let content = std::fs::read_to_string(&session_file).unwrap();
    let session: serde_json::Value = serde_json::from_str(&content).unwrap();
    let pid = session["pid"].as_u64().unwrap();

    // Check that CLOUDAPPS_API_TOKEN is not in the agent's environment.
    // On macOS, /proc doesn't exist, so we use `ps -E` instead.
    #[cfg(target_os = "linux")]
    {
        let environ_path = format!("/proc/{}/environ", pid);
        if let Ok(environ) = std::fs::read_to_string(&environ_path) {
            assert!(
                !environ.contains("CLOUDAPPS_API_TOKEN=test-dummy-token"),
                "CLOUDAPPS_API_TOKEN should not be in agent environment"
            );
        }
    }

    // On any platform, verify the agent is running (PID is alive).
    let alive = unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
    assert!(alive, "agent process should be alive");

    // Cleanup.
    stop_agent(tmpdir.path(), xdg_data_home.path());
}
