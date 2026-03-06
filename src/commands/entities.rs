use crate::cli::entities::EntitiesCommand;
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::models::entity::EntityStatus;
use crate::models::filter::{FilterCondition, FilterRequest};
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &EntitiesCommand,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    match command {
        EntitiesCommand::List(args) => list(client, args, output_format, raw).await,
        EntitiesCommand::Fetch { id } => fetch(client, id, output_format).await,
        EntitiesCommand::FetchTree { id } => fetch_tree(client, id, output_format).await,
    }
}

async fn list(
    client: &CloudAppsClient,
    args: &crate::cli::entities::ListArgs,
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

    if let Some(ref entity_type) = args.entity_type {
        filter.add_filter("type", FilterCondition::eq(vec![entity_type.as_str()]));
    }
    if args.is_admin {
        filter.add_filter("isAdmin", FilterCondition::eq(vec!["true"]));
    }
    if args.is_external {
        filter.add_filter("isExternal", FilterCondition::eq_bool(true));
    }
    if let Some(ref domain) = args.domain {
        filter.add_filter("domain", FilterCondition::eq(vec![domain.as_str()]));
    }
    if let Some(ref status) = args.status {
        let st = EntityStatus::from_str_loose(status)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown status: {}", status)))?;
        filter.add_filter("status", FilterCondition::eq(vec![st.to_api()]));
    }

    if let Some(ref raw) = args.filter {
        let raw_filter: serde_json::Value =
            serde_json::from_str(raw).map_err(|e| AppError::InvalidInput(e.to_string()))?;
        if let Some(obj) = raw_filter.as_object() {
            for (k, v) in obj {
                if k == "skip" || k == "limit" {
                    continue;
                }
                if let Ok(cond) = serde_json::from_value(v.clone()) {
                    filter.filters.insert(k.clone(), cond);
                }
            }
        }
    }

    let resp: serde_json::Value = if args.all {
        let data: Vec<serde_json::Value> = client.list_all("/api/v1/entities/", filter).await?;
        serde_json::json!({ "data": data, "total": data.len() })
    } else {
        client
            .post("/api/v1/entities/", &filter)
            .await?
            .json()
            .await?
    };

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_data(&resp, raw, output_format.is_minify()),
        OutputFormat::Table => {
            print_entities_table(&resp);
            Ok(())
        }
    }
}

fn print_entities_table(value: &serde_json::Value) {
    use crate::output::table::truncate;

    println!(
        "{:<20} {:<12} {:<30} {:<20} {:<10}",
        "ID", "TYPE", "NAME", "DOMAIN", "STATUS"
    );

    if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        for item in data {
            let id = item.get("_id").and_then(|i| i.as_str()).unwrap_or("-");
            let etype = item
                .get("entityType")
                .and_then(|t| t.as_str())
                .unwrap_or("-");
            let name = item
                .get("displayName")
                .and_then(|n| n.as_str())
                .unwrap_or("-");
            let domain = item.get("domain").and_then(|d| d.as_str()).unwrap_or("-");
            let status = item
                .get("status")
                .and_then(|s| s.as_i64())
                .and_then(|v| EntityStatus::from_api(v as i32))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:<20} {:<12} {:<30} {:<20} {:<10}",
                truncate(id, 18),
                truncate(etype, 10),
                truncate(name, 28),
                truncate(domain, 18),
                status
            );
        }
    }
}

async fn fetch(
    client: &CloudAppsClient,
    id: &str,
    output_format: OutputFormat,
) -> Result<(), AppError> {
    let resp: serde_json::Value = client
        .get(&format!("/api/v1/entities/{}/", id))
        .await?
        .json()
        .await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_raw(&resp, output_format.is_minify()),
        OutputFormat::Table => crate::output::json::print_json_raw(&resp, false),
    }
}

async fn fetch_tree(
    client: &CloudAppsClient,
    id: &str,
    output_format: OutputFormat,
) -> Result<(), AppError> {
    let resp: serde_json::Value = client
        .get(&format!("/api/v1/entities/{}/tree/", id))
        .await?
        .json()
        .await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_raw(&resp, output_format.is_minify()),
        OutputFormat::Table => crate::output::json::print_json_raw(&resp, false),
    }
}
