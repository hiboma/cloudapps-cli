mod common;

use cloudapps::client::CloudAppsClient;
use cloudapps::commands::data_enrichment;
use cloudapps::output::OutputFormat;
use mockito::Server;

fn create_client(base_url: &str) -> CloudAppsClient {
    common::create_client(base_url)
}

#[tokio::test]
async fn test_data_enrichment_list_returns_data() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/list_response.json");

    let mock = server
        .mock("GET", "/api/subnet/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let resp: serde_json::Value = client
        .get("/api/subnet/")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["total"], 2);
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
    assert_eq!(resp["data"][0]["_id"], "subnet-xxxx-1");
    assert_eq!(resp["data"][0]["name"], "Headquarters");
    assert_eq!(resp["data"][0]["organization"], "Example Corp");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_create() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/create_response.json");

    let mock = server
        .mock("POST", "/api/subnet/create_rule/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Create(
        cloudapps::cli::data_enrichment::CreateArgs {
            name: "New Range".to_string(),
            subnets: "203.0.113.0/24".to_string(),
            category: "corporate".to_string(),
            organization: Some("Example Corp".to_string()),
            tags: None,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_delete() {
    let mut server = Server::new_async().await;

    let mock = server
        .mock("DELETE", "/api/subnet/subnet-xxxx-1/")
        .with_status(200)
        .with_body("{}")
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Delete {
        id: "subnet-xxxx-1".to_string(),
    };

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_update() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/create_response.json");

    let mock = server
        .mock("POST", "/api/subnet/subnet-xxxx-1/update_rule/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Update(
        cloudapps::cli::data_enrichment::UpdateArgs {
            id: "subnet-xxxx-1".to_string(),
            name: Some("Updated Range".to_string()),
            subnets: None,
            category: None,
            organization: None,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_handle_list() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/list_response.json");

    let _mock = server
        .mock("GET", "/api/subnet/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::List(
        cloudapps::cli::data_enrichment::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            category: None,
            tag: None,
            builtin: false,
            custom: false,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_data_enrichment_invalid_category() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Create(
        cloudapps::cli::data_enrichment::CreateArgs {
            name: "Test".to_string(),
            subnets: "192.0.2.0/24".to_string(),
            category: "invalid".to_string(),
            organization: None,
            tags: None,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_data_enrichment_handle_list_table_output() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/list_response.json");

    let _mock = server
        .mock("GET", "/api/subnet/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::List(
        cloudapps::cli::data_enrichment::ListArgs {
            limit: Some(100),
            skip: None,
            all: false,
            filter: None,
            category: None,
            tag: None,
            builtin: false,
            custom: false,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Table, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_data_enrichment_handle_list_with_category_filter() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/list_response.json");

    let _mock = server
        .mock("GET", "/api/subnet/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::List(
        cloudapps::cli::data_enrichment::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: None,
            category: Some("corporate".to_string()),
            tag: Some("tag-001".to_string()),
            builtin: true,
            custom: false,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_data_enrichment_handle_list_custom_flag() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/list_response.json");

    let _mock = server
        .mock("GET", "/api/subnet/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::List(
        cloudapps::cli::data_enrichment::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: None,
            category: None,
            tag: None,
            builtin: false,
            custom: true,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_data_enrichment_create_with_tags() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/create_response.json");

    let mock = server
        .mock("POST", "/api/subnet/create_rule/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Create(
        cloudapps::cli::data_enrichment::CreateArgs {
            name: "New Range".to_string(),
            subnets: "203.0.113.0/24, 198.51.100.0/24".to_string(),
            category: "corporate".to_string(),
            organization: None,
            tags: Some("tag1, tag2".to_string()),
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_update_multiple_fields() {
    let mut server = Server::new_async().await;
    let body = include_str!("../testdata/data_enrichment/create_response.json");

    let mock = server
        .mock("POST", "/api/subnet/subnet-xxxx-1/update_rule/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = create_client(&server.url());
    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Update(
        cloudapps::cli::data_enrichment::UpdateArgs {
            id: "subnet-xxxx-1".to_string(),
            name: Some("Updated Range".to_string()),
            subnets: Some("192.0.2.0/24".to_string()),
            category: Some("corporate".to_string()),
            organization: Some("Example Corp".to_string()),
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_data_enrichment_update_invalid_category() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::Update(
        cloudapps::cli::data_enrichment::UpdateArgs {
            id: "subnet-xxxx-1".to_string(),
            name: None,
            subnets: None,
            category: Some("invalid".to_string()),
            organization: None,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_data_enrichment_list_invalid_category() {
    let server = Server::new_async().await;
    let client = create_client(&server.url());

    let command = cloudapps::cli::data_enrichment::DataEnrichmentCommand::List(
        cloudapps::cli::data_enrichment::ListArgs {
            limit: None,
            skip: None,
            all: false,
            filter: None,
            category: Some("invalid".to_string()),
            tag: None,
            builtin: false,
            custom: false,
        },
    );

    let result = data_enrichment::handle(&client, &command, OutputFormat::Json, false).await;
    assert!(result.is_err());
}
