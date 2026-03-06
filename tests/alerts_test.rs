mod common;

use cloudapps::client::CloudAppsClient;
use cloudapps::commands::alerts;
use cloudapps::models::filter::FilterRequest;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_alerts_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let filter = FilterRequest::new();
    let resp: serde_json::Value = client
        .post("/api/v1/alerts/", &filter)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["total"], 2);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["data"][0]["_id"], "alert-xxxx-1");
    assert_eq!(resp["data"][0]["title"], "Impossible travel activity");
    assert_eq!(resp["data"][0]["severityValue"], 2);
    assert_eq!(resp["data"][0]["resolutionStatusValue"], 0);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_fetch_returns_single() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/fetch_response.json");

    let mock = server
        .mock("GET", "/api/v1/alerts/alert-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/alerts/alert-xxxx-1/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "alert-xxxx-1");
    assert_eq!(resp["severityValue"], 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_close_benign() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/alerts/close_benign/")
        .match_header("Authorization", "Token test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"closed_benign": 1}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::Close(cloudapps::cli::alerts::CloseArgs {
        ids: vec!["alert-xxxx-1".to_string()],
        close_as: "benign".to_string(),
        comment: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_close_with_comment() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/alerts/close_true_positive/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"closed_true_positive": 1}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::Close(cloudapps::cli::alerts::CloseArgs {
        ids: vec!["alert-xxxx-1".to_string()],
        close_as: "true-positive".to_string(),
        comment: Some("confirmed threat".to_string()),
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_mark_read() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/alerts/alert-xxxx-1/read/")
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::MarkRead {
        ids: vec!["alert-xxxx-1".to_string()],
    };

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_mark_unread() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/alerts/alert-xxxx-1/unread/")
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::MarkUnread {
        ids: vec!["alert-xxxx-1".to_string()],
    };

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_bulk_close() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/alerts/close_false_positive/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"closed_false_positive": 2}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::Close(cloudapps::cli::alerts::CloseArgs {
        ids: vec!["alert-xxxx-1".to_string(), "alert-xxxx-2".to_string()],
        close_as: "false-positive".to_string(),
        comment: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_alerts_handle_list_with_severity_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: Some(100),
        skip: None,
        all: false,
        filter: None,
        severity: Some("high".to_string()),
        resolution: None,
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_invalid_close_type() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::alerts::AlertsCommand::Close(cloudapps::cli::alerts::CloseArgs {
        ids: vec!["alert-xxxx-1".to_string()],
        close_as: "invalid".to_string(),
        comment: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_alerts_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: Some(100),
        skip: None,
        all: false,
        filter: None,
        severity: None,
        resolution: None,
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_handle_list_with_resolution_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: Some(100),
        skip: None,
        all: false,
        filter: None,
        severity: None,
        resolution: Some("open".to_string()),
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_handle_list_open_flag() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        severity: None,
        resolution: None,
        open: true,
        closed: false,
        query: Some("test".to_string()),
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_handle_list_closed_flag() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        severity: None,
        resolution: None,
        open: false,
        closed: true,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_invalid_severity() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        severity: Some("invalid".to_string()),
        resolution: None,
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_alerts_invalid_resolution() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        severity: None,
        resolution: Some("invalid".to_string()),
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_alerts_handle_fetch_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/alerts/alert-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::Fetch(cloudapps::cli::alerts::FetchArgs {
        id: "alert-xxxx-1".to_string(),
        with_activities: false,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_alerts_handle_list_with_raw_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/alerts/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/alerts/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::alerts::AlertsCommand::List(cloudapps::cli::alerts::ListArgs {
        limit: None,
        skip: Some(10),
        all: false,
        filter: Some(r#"{"severity":{"eq":[2]}}"#.to_string()),
        severity: None,
        resolution: None,
        open: false,
        closed: false,
        query: None,
    });

    let result = alerts::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}
