mod common;

use cloudapps::cli::policies::{FetchArgs, ListArgs, PoliciesCommand, PolicyType};
use cloudapps::client::CloudAppsClient;
use cloudapps::commands::policies;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_policies_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/list_response.json");

    let mock = server
        .mock("GET", "/api/v1/policies/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/policies/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let arr = resp.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["_id"], "pol-xxxx-1");
    assert_eq!(arr[0]["policyType"], "AUDIT");
    assert_eq!(arr[1]["policyType"], "ANOMALY_DETECTION");
    assert_eq!(arr[2]["enabled"], false);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_policies_fetch_returns_single() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/fetch_response.json");

    let mock = server
        .mock("GET", "/api/v1/policy/activity/pol-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/policy/activity/pol-xxxx-1/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "pol-xxxx-1");
    assert_eq!(resp["policyType"], "AUDIT");
    assert_eq!(resp["enabled"], true);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_policies_handle_list() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/list_response.json");

    let _mock = server
        .mock("GET", "/api/v1/policies/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = PoliciesCommand::List(ListArgs { filter: None });

    let result = policies::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policies_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/list_response.json");

    let _mock = server
        .mock("GET", "/api/v1/policies/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = PoliciesCommand::List(ListArgs { filter: None });

    let result = policies::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policies_handle_fetch() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/policy/activity/pol-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = PoliciesCommand::Fetch(FetchArgs {
        r#type: PolicyType::Activity,
        id: "pol-xxxx-1".to_string(),
    });

    let result = policies::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policies_handle_fetch_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/policies/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/policy/activity/pol-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = PoliciesCommand::Fetch(FetchArgs {
        r#type: PolicyType::Activity,
        id: "pol-xxxx-1".to_string(),
    });

    let result = policies::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policies_auth_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/policies/")
        .with_status(401)
        .with_body(r#"{"error": "unauthorized"}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let result = client.get("/api/v1/policies/").await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_policy_type_api_path_segment() {
    assert_eq!(PolicyType::Activity.api_path_segment(), "activity");
    assert_eq!(PolicyType::Anomaly.api_path_segment(), "anomaly");
    assert_eq!(PolicyType::Discovery.api_path_segment(), "discovery");
    assert_eq!(
        PolicyType::DiscoveryAnomaly.api_path_segment(),
        "discovery_anomaly"
    );
    assert_eq!(PolicyType::File.api_path_segment(), "file");
    assert_eq!(
        PolicyType::AppPermissions.api_path_segment(),
        "app_permissions"
    );
    assert_eq!(PolicyType::Session.api_path_segment(), "session");
}

#[tokio::test]
async fn test_policy_type_display() {
    assert_eq!(format!("{}", PolicyType::Activity), "activity");
    assert_eq!(format!("{}", PolicyType::Anomaly), "anomaly");
    assert_eq!(
        format!("{}", PolicyType::DiscoveryAnomaly),
        "discovery_anomaly"
    );
    assert_eq!(format!("{}", PolicyType::AppPermissions), "app_permissions");
}
