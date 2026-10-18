use assert_cmd::cargo::cargo_bin_cmd;
use mockito::{Matcher, Server};
use serde_json::{Value, json};
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn activities_list_uses_environment_credentials_and_prints_json() {
    let mut server = Server::new();
    let response: Value =
        serde_json::from_str(include_str!("../testdata/activities/list_response.json")).unwrap();
    let mock = server
        .mock("POST", "/api/v1/activities/")
        .match_header("authorization", "Token integration-test-token")
        .match_body(Matcher::Json(json!({ "limit": 2 })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response.to_string())
        .expect(1)
        .create();
    let home = TempDir::new().unwrap();

    // Supplying a non-empty environment token skips credential-store reads.
    // Isolate both file search locations and bypass inherited proxy settings.
    let assertion = cargo_bin_cmd!("cloudapps-cli")
        .current_dir(home.path())
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .env("CLOUDAPPS_API_URL", server.url())
        .env("CLOUDAPPS_API_TOKEN", "integration-test-token")
        .env("CLOUDAPPS_OUTPUT_FORMAT", "json")
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("no_proxy", "127.0.0.1,localhost")
        .args(["activities", "list", "--limit", "2"])
        .timeout(Duration::from_secs(10))
        .assert()
        .success()
        .stderr("");

    let output: Value = serde_json::from_slice(&assertion.get_output().stdout).unwrap();
    assert_eq!(output, response["data"]);
    mock.assert();
}
