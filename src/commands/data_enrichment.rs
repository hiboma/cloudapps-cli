use crate::cli::data_enrichment::DataEnrichmentCommand;
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::models::data_enrichment::SubnetCategory;
use crate::models::filter::{FilterCondition, FilterRequest};
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &DataEnrichmentCommand,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    match command {
        DataEnrichmentCommand::List(args) => list(client, args, output_format, raw).await,
        DataEnrichmentCommand::Create(args) => create(client, args).await,
        DataEnrichmentCommand::Update(args) => update(client, args).await,
        DataEnrichmentCommand::Delete { id } => delete(client, id).await,
    }
}

async fn list(
    client: &CloudAppsClient,
    args: &crate::cli::data_enrichment::ListArgs,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    let mut filter = FilterRequest::new();

    if let Some(limit) = args.limit {
        filter = filter.with_limit(limit);
    }
    if let Some(skip) = args.skip {
        filter = filter.with_skip(skip);
    }

    if let Some(ref category) = args.category {
        let cat = SubnetCategory::from_str_loose(category)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown category: {}", category)))?;
        filter.add_filter("category", FilterCondition::eq(vec![cat.to_api()]));
    }
    if let Some(ref tag) = args.tag {
        filter.add_filter("tags", FilterCondition::eq(vec![tag.as_str()]));
    }
    if args.builtin {
        filter.add_filter("builtIn", FilterCondition::eq_bool(true));
    }
    if args.custom {
        filter.add_filter("builtIn", FilterCondition::eq_bool(false));
    }

    // Data enrichment list uses GET, but filters are applied as query parameters.
    // For simplicity, we fetch all and filter client-side if filters are complex.
    // The API also supports query parameters for basic filtering.
    let resp: serde_json::Value = client.get("/api/subnet/").await?.json().await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => {
            crate::output::json::print_json_data(&resp, raw, output_format.is_minify())
        }
        OutputFormat::Table => {
            print_data_enrichment_table(&resp);
            Ok(())
        }
    }
}

fn print_data_enrichment_table(value: &serde_json::Value) {
    use crate::output::table::truncate;

    println!(
        "{:<20} {:<25} {:<30} {:<16} {:<20}",
        "ID", "NAME", "SUBNETS", "CATEGORY", "ORGANIZATION"
    );

    let items = if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        data.clone()
    } else if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        return;
    };

    for item in &items {
        let id = item.get("_id").and_then(|i| i.as_str()).unwrap_or("-");
        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("-");
        let subnets = item
            .get("subnets")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| {
                        s.get("originalString")
                            .and_then(|o| o.as_str())
                            .or_else(|| s.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "-".to_string());
        let category = item
            .get("category")
            .and_then(|c| c.as_i64())
            .and_then(|v| SubnetCategory::from_api(v as i32))
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        let org = item
            .get("organization")
            .and_then(|o| o.as_str())
            .unwrap_or("-");

        println!(
            "{:<20} {:<25} {:<30} {:<16} {:<20}",
            truncate(id, 18),
            truncate(name, 23),
            truncate(&subnets, 28),
            category,
            truncate(org, 18)
        );
    }
}

async fn create(
    client: &CloudAppsClient,
    args: &crate::cli::data_enrichment::CreateArgs,
) -> Result<(), AppError> {
    let category = SubnetCategory::from_str_loose(&args.category)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown category: {}", args.category)))?;

    let subnets: Vec<&str> = args.subnets.split(',').map(|s| s.trim()).collect();

    let body = serde_json::json!({
        "name": args.name,
        "subnets": subnets,
        "category": category.to_api(),
        "organization": args.organization,
        "tags": args.tags.as_ref().map(|t| t.split(',').map(|s| s.trim()).collect::<Vec<_>>()),
    });

    let resp: serde_json::Value = client
        .post_json("/api/subnet/create_rule/", &body)
        .await?
        .json()
        .await?;

    crate::output::json::print_json_raw(&resp, false)
}

async fn update(
    client: &CloudAppsClient,
    args: &crate::cli::data_enrichment::UpdateArgs,
) -> Result<(), AppError> {
    let mut body = serde_json::Map::new();

    if let Some(ref name) = args.name {
        body.insert("name".to_string(), serde_json::json!(name));
    }
    if let Some(ref subnets) = args.subnets {
        let list: Vec<&str> = subnets.split(',').map(|s| s.trim()).collect();
        body.insert("subnets".to_string(), serde_json::json!(list));
    }
    if let Some(ref category) = args.category {
        let cat = SubnetCategory::from_str_loose(category)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown category: {}", category)))?;
        body.insert("category".to_string(), serde_json::json!(cat.to_api()));
    }
    if let Some(ref org) = args.organization {
        body.insert("organization".to_string(), serde_json::json!(org));
    }

    let resp: serde_json::Value = client
        .post_json(
            &format!("/api/subnet/{}/update_rule/", args.id),
            &serde_json::Value::Object(body),
        )
        .await?
        .json()
        .await?;

    crate::output::json::print_json_raw(&resp, false)
}

async fn delete(client: &CloudAppsClient, id: &str) -> Result<(), AppError> {
    client.delete(&format!("/api/subnet/{}/", id)).await?;
    eprintln!("Deleted IP range {}", id);
    Ok(())
}
