use crate::cli::alerts::AlertsCommand;
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::models::alert::{CloseType, ResolutionStatus, Severity};
use crate::models::filter::{FilterCondition, FilterRequest};
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &AlertsCommand,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    match command {
        AlertsCommand::List(args) => list(client, args, output_format, raw).await,
        AlertsCommand::Fetch(args) => fetch(client, args, output_format).await,
        AlertsCommand::Close(args) => close(client, args).await,
        AlertsCommand::MarkRead { ids } => bulk_action(client, ids, "read").await,
        AlertsCommand::MarkUnread { ids } => bulk_action(client, ids, "unread").await,
    }
}

async fn list(
    client: &CloudAppsClient,
    args: &crate::cli::alerts::ListArgs,
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

    if let Some(ref severity) = args.severity {
        let sev = Severity::from_str_loose(severity)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown severity: {}", severity)))?;
        filter.add_filter("severity", FilterCondition::eq(vec![sev.to_api()]));
    }

    if let Some(ref resolution) = args.resolution {
        let res = ResolutionStatus::from_str_loose(resolution).ok_or_else(|| {
            AppError::InvalidInput(format!("unknown resolution status: {}", resolution))
        })?;
        filter.add_filter("resolutionStatus", FilterCondition::eq(vec![res.to_api()]));
    }

    if args.open {
        filter.add_filter("alertOpen", FilterCondition::eq_bool(true));
    }
    if args.closed {
        filter.add_filter("alertOpen", FilterCondition::eq_bool(false));
    }

    if let Some(ref query) = args.query {
        filter.add_filter("text", FilterCondition::text(query));
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
        let data: Vec<serde_json::Value> = client.list_all("/api/v1/alerts/", filter).await?;
        serde_json::json!({ "data": data, "total": data.len() })
    } else {
        client
            .post("/api/v1/alerts/", &filter)
            .await?
            .json()
            .await?
    };

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => {
            crate::output::json::print_json_data(&resp, raw, output_format.is_minify())
        }
        OutputFormat::Table => {
            print_alerts_table(&resp);
            Ok(())
        }
    }
}

fn print_alerts_table(value: &serde_json::Value) {
    use crate::output::table::{format_timestamp, truncate};

    println!(
        "{:<14} {:<40} {:<15} {:<16} {:<24}",
        "ID", "TITLE", "SEVERITY", "RESOLUTION", "TIMESTAMP"
    );

    if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        for item in data {
            let id = item.get("_id").and_then(|i| i.as_str()).unwrap_or("-");
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("-");
            let severity = item
                .get("severityValue")
                .and_then(|s| s.as_i64())
                .and_then(|v| Severity::from_api(v as i32))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string());
            let resolution = item
                .get("resolutionStatusValue")
                .and_then(|r| r.as_i64())
                .and_then(|v| ResolutionStatus::from_api(v as i32))
                .map(|r| r.to_string())
                .unwrap_or_else(|| "-".to_string());
            let ts = format_timestamp(item.get("timestamp").and_then(|t| t.as_i64()));

            println!(
                "{:<14} {:<40} {:<15} {:<16} {:<24}",
                truncate(id, 12),
                truncate(title, 38),
                severity,
                resolution,
                ts
            );
        }
    }
}

async fn fetch(
    client: &CloudAppsClient,
    args: &crate::cli::alerts::FetchArgs,
    output_format: OutputFormat,
) -> Result<(), AppError> {
    let resp: serde_json::Value = client
        .get(&format!("/api/v1/alerts/{}/", args.id))
        .await?
        .json()
        .await?;

    if args.with_activities {
        let mut filter = FilterRequest::new();
        filter.add_filter(
            "activity.alertId",
            FilterCondition::raw("eq".to_string(), serde_json::Value::String(args.id.clone())),
        );
        let activities: Vec<serde_json::Value> =
            client.list_all("/api/v1/activities/", filter).await?;
        let combined = serde_json::json!({
            "alert": resp,
            "activities": activities,
        });
        match output_format {
            OutputFormat::Json | OutputFormat::JsonMinify => {
                crate::output::json::print_json_raw(&combined, output_format.is_minify())
            }
            OutputFormat::Table => crate::output::json::print_json_raw(&combined, false),
        }
    } else {
        match output_format {
            OutputFormat::Json | OutputFormat::JsonMinify => {
                crate::output::json::print_json_raw(&resp, output_format.is_minify())
            }
            OutputFormat::Table => crate::output::json::print_json_raw(&resp, false),
        }
    }
}

async fn close(
    client: &CloudAppsClient,
    args: &crate::cli::alerts::CloseArgs,
) -> Result<(), AppError> {
    let close_type = CloseType::from_str_loose(&args.close_as).ok_or_else(|| {
        AppError::InvalidInput(format!(
            "unknown close type: {}. Use: benign, false-positive, true-positive",
            args.close_as
        ))
    })?;

    let mut body = serde_json::json!({
        "filters": {
            "id": {
                "eq": args.ids,
            }
        }
    });

    if let Some(ref comment) = args.comment {
        body["comment"] = serde_json::json!(comment);
    }

    let path = format!("/api/v1/alerts/{}/", close_type.api_path_segment());
    let resp: serde_json::Value = client.post_json(&path, &body).await?.json().await?;
    crate::output::json::print_json_raw(&resp, false)?;

    Ok(())
}

async fn bulk_action(
    client: &CloudAppsClient,
    ids: &[String],
    action: &str,
) -> Result<(), AppError> {
    let body = serde_json::json!({});
    for id in ids {
        let path = format!("/api/v1/alerts/{}/{}/", id, action);
        client.post_json(&path, &body).await?;
        eprintln!("Marked alert {} as {}", id, action);
    }
    Ok(())
}
