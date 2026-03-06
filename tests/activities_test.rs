mod common;

use cloudapps::client::CloudAppsClient;
use cloudapps::commands::activities;
use cloudapps::models::filter::FilterRequest;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_activities_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let filter = FilterRequest::new().with_limit(100);
    let resp: serde_json::Value = client
        .post("/api/v1/activities/", &filter)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["total"], 2);
    assert_eq!(resp["hasNext"], false);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["data"][0]["_id"], "act-xxxx-1");
    assert_eq!(resp["data"][0]["actionType"], "LOGIN");
    assert_eq!(resp["data"][0]["ipAddress"], "192.0.2.1");
    assert_eq!(resp["data"][1]["user"]["userName"], "user2@example.com");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_activities_fetch_returns_single() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/fetch_response.json");

    let mock = server
        .mock("GET", "/api/v1/activities/act-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/v1/activities/act-xxxx-1/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["_id"], "act-xxxx-1");
    assert_eq!(resp["actionType"], "LOGIN");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_activities_list_with_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let mock = server
        .mock("POST", "/api/v1/activities/")
        .match_header("Authorization", "Token test-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let mut filter = FilterRequest::new().with_limit(10);
    filter.add_filter(
        "user.username",
        cloudapps::models::filter::FilterCondition::eq(vec!["user1@example.com"]),
    );

    let resp: serde_json::Value = client
        .post("/api/v1/activities/", &filter)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(resp["data"].as_array().unwrap().len() > 0);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_activities_handle_list() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::activities::ActivitiesCommand::List(cloudapps::cli::activities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            user: None,
            ip: None,
            country: None,
            query: None,
        });

    let result = activities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activities_auth_error() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(401)
        .with_body(r#"{"error": "unauthorized"}"#)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let filter = FilterRequest::new();
    let result = client.post("/api/v1/activities/", &filter).await;

    assert!(result.is_err());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_activities_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::activities::ActivitiesCommand::List(cloudapps::cli::activities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            user: None,
            ip: None,
            country: None,
            query: None,
        });

    let result = activities::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activities_handle_list_with_user_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::activities::ActivitiesCommand::List(cloudapps::cli::activities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            user: Some("user1@example.com".to_string()),
            ip: Some("192.0.2.1".to_string()),
            country: Some("US".to_string()),
            query: Some("login".to_string()),
        });

    let result = activities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activities_handle_fetch_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/fetch_response.json");

    let _mock = server
        .mock("GET", "/api/v1/activities/act-xxxx-1/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::activities::ActivitiesCommand::Fetch {
        id: "act-xxxx-1".to_string(),
    };

    let result = activities::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activities_handle_list_with_raw_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/activities/list_response.json");

    let _mock = server
        .mock("POST", "/api/v1/activities/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command =
        cloudapps::cli::activities::ActivitiesCommand::List(cloudapps::cli::activities::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: Some(r#"{"actionType":{"eq":["LOGIN"]}}"#.to_string()),
            user: None,
            ip: None,
            country: None,
            query: None,
        });

    let result = activities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_activities_handle_list_invalid_raw_filter() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());
    let command =
        cloudapps::cli::activities::ActivitiesCommand::List(cloudapps::cli::activities::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: Some("not valid json".to_string()),
            user: None,
            ip: None,
            country: None,
            query: None,
        });

    let result = activities::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}
