mod common;

use cloudapps::client::CloudAppsClient;
use cloudapps::commands::entities;
use cloudapps::models::filter::FilterRequest;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_entities_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/list_response.json");

    let mock = server
        .mock("POST", "/api/v1/entities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let filter = FilterRequest::new();
    let resp: serde_json::Value = client
        .post("/api/v1/entities/", &filter)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["total"], 2);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["data"][0]["_id"], "entity-xxxx-1");
    assert_eq!(resp["data"][0]["entityType"], "user");
    assert_eq!(resp["data"][0]["domain"], "example.com");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_entities_fetch_returns_single() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/fetch_response.json");

    let mock = server
        .mock("GET", "/api/v1/entities/entity-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/entities/entity-xxxx-1/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "entity-xxxx-1");
    assert_eq!(resp["entityType"], "user");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_entities_fetch_tree() {
    let mut server = Server::new_async().await;
    let body = r#"{"_id": "entity-xxxx-1", "children": []}"#;

    let mock = server
        .mock("GET", "/api/v1/entities/entity-xxxx-1/tree/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/entities/entity-xxxx-1/tree/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "entity-xxxx-1");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_entities_handle_list() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/entities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::entities::EntitiesCommand::List(cloudapps::cli::entities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            entity_type: None,
            is_admin: false,
            is_external: false,
            domain: None,
            status: None,
        });

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_handle_fetch() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/entities/entity-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::entities::EntitiesCommand::Fetch {
        id: "entity-xxxx-1".to_string(),
    };

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/entities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::entities::EntitiesCommand::List(cloudapps::cli::entities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            entity_type: None,
            is_admin: false,
            is_external: false,
            domain: None,
            status: None,
        });

    let result = entities::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_handle_list_with_filters() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/entities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::entities::EntitiesCommand::List(cloudapps::cli::entities::ListArgs {
            limit: Some(50),
            skip: Some(10),
            all: false,
            filter: None,
            entity_type: Some("user".to_string()),
            is_admin: true,
            is_external: true,
            domain: Some("example.com".to_string()),
            status: Some("active".to_string()),
        });

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_invalid_status() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command =
        cloudapps::cli::entities::EntitiesCommand::List(cloudapps::cli::entities::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: None,
            entity_type: None,
            is_admin: false,
            is_external: false,
            domain: None,
            status: Some("invalid".to_string()),
        });

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_entities_handle_fetch_tree() {
    let mut server = Server::new_async().await;
    let body = r#"{"_id": "entity-xxxx-1", "children": []}"#;

    let _mock = server
        .mock("GET", "/api/v1/entities/entity-xxxx-1/tree/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::entities::EntitiesCommand::FetchTree {
        id: "entity-xxxx-1".to_string(),
    };

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_handle_fetch_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/entities/entity-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::entities::EntitiesCommand::Fetch {
        id: "entity-xxxx-1".to_string(),
    };

    let result = entities::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entities_handle_list_with_raw_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/entities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/entities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::entities::EntitiesCommand::List(cloudapps::cli::entities::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: Some(r#"{"type":{"eq":["user"]}}"#.to_string()),
            entity_type: None,
            is_admin: false,
            is_external: false,
            domain: None,
            status: None,
        });

    let result = entities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}
