mod common;

use cloudapps::client::CloudAppsClient;
use cloudapps::commands::files;
use cloudapps::models::filter::FilterRequest;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_files_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/list_response.json");

    let mock = server
        .mock("POST", "/api/v1/files/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let filter = FilterRequest::new();
    let resp: serde_json::Value = client
        .post("/api/v1/files/", &filter)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["total"], 2);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["data"][0]["_id"], "file-xxxx-1");
    assert_eq!(resp["data"][0]["fileName"], "report-2024.xlsx");
    assert_eq!(resp["data"][1]["ownerName"], "user2@example.com");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_files_fetch_returns_single() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/fetch_response.json");

    let mock = server
        .mock("GET", "/api/v1/files/file-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/files/file-xxxx-1/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "file-xxxx-1");
    assert_eq!(resp["extension"], "xlsx");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_files_handle_list() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/files/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: Some(100),
        skip: None,
        all: false,
        filter: None,
        service: None,
        filetype: None,
        filename: None,
        extension: None,
        sharing: None,
    });

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_handle_fetch() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/files/file-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::Fetch {
        id: "file-xxxx-1".to_string(),
    };

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_404_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/files/nonexistent/")
        .with_status(404)
        .with_body(r#"{"error": "not found"}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let result = client.get("/api/v1/files/nonexistent/").await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_files_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/files/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: Some(100),
        skip: None,
        all: false,
        filter: None,
        service: None,
        filetype: None,
        filename: None,
        extension: None,
        sharing: None,
    });

    let result = files::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_handle_list_with_filters() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/files/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: Some(50),
        skip: Some(5),
        all: false,
        filter: None,
        service: Some(11770),
        filetype: Some("document".to_string()),
        filename: Some("report".to_string()),
        extension: Some("xlsx".to_string()),
        sharing: Some("private".to_string()),
    });

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_invalid_filetype() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        service: None,
        filetype: Some("invalid".to_string()),
        filename: None,
        extension: None,
        sharing: None,
    });

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_files_invalid_sharing() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: None,
        service: None,
        filetype: None,
        filename: None,
        extension: None,
        sharing: Some("invalid".to_string()),
    });

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_files_handle_fetch_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/files/file-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::Fetch {
        id: "file-xxxx-1".to_string(),
    };

    let result = files::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_files_handle_list_with_raw_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/files/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/files/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::files::FilesCommand::List(cloudapps::cli::files::ListArgs {
        limit: None,
        skip: None,
        all: false,
        filter: Some(r#"{"fileType":{"eq":[1]}}"#.to_string()),
        service: None,
        filetype: None,
        filename: None,
        extension: None,
        sharing: None,
    });

    let result = files::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}
