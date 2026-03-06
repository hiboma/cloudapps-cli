use crate::cli::activities::ActivitiesCommand;
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::models::filter::FilterRequest;
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &ActivitiesCommand,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    match command {
        ActivitiesCommand::List(args) => list(client, args, output_format, raw).await,
        ActivitiesCommand::Fetch { id } => fetch(client, id, output_format).await,
    }
}

async fn list(
    client: &CloudAppsClient,
    args: &crate::cli::activities::ListArgs,
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

    if let Some(ref user) = args.user {
        filter.add_filter(
            "user.username",
            crate::models::filter::FilterCondition::eq(vec![user.as_str()]),
        );
    }
    if let Some(ref ip) = args.ip {
        filter.add_filter(
            "ip.address",
            crate::models::filter::FilterCondition::eq(vec![ip.as_str()]),
        );
    }
    if let Some(ref country) = args.country {
        filter.add_filter(
            "location.country",
            crate::models::filter::FilterCondition::eq(vec![country.as_str()]),
        );
    }
    if let Some(ref query) = args.query {
        filter.add_filter("text", crate::models::filter::FilterCondition::text(query));
    }

    let resp: serde_json::Value = if args.all {
        let data: Vec<serde_json::Value> = client.list_all("/api/v1/activities/", filter).await?;
        serde_json::json!({ "data": data, "total": data.len() })
    } else {
        client
            .post("/api/v1/activities/", &filter)
            .await?
            .json()
            .await?
    };

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_data(&resp, raw, output_format.is_minify()),
        OutputFormat::Table => {
            print_activities_table(&resp);
            Ok(())
        }
    }
}

fn print_activities_table(value: &serde_json::Value) {
    use crate::output::table::{format_timestamp, truncate};

    println!(
        "{:<24} {:<30} {:<20} {:<16} {:<6}",
        "TIMESTAMP", "USER", "ACTION", "IP", "COUNTRY"
    );

    if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        for item in data {
            let ts = format_timestamp(item.get("timestamp").and_then(|t| t.as_i64()));
            let user = item
                .get("user")
                .and_then(|u| u.get("userName"))
                .and_then(|u| u.as_str())
                .unwrap_or("-");
            let action = item
                .get("actionType")
                .and_then(|a| a.as_str())
                .unwrap_or("-");
            let ip = item
                .get("ipAddress")
                .and_then(|i| i.as_str())
                .unwrap_or("-");
            let country = item
                .get("location")
                .and_then(|l| l.get("country"))
                .and_then(|c| c.as_str())
                .unwrap_or("-");

            println!(
                "{:<24} {:<30} {:<20} {:<16} {:<6}",
                ts,
                truncate(user, 28),
                truncate(action, 18),
                ip,
                country
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
        .get(&format!("/api/v1/activities/{}/", id))
        .await?
        .json()
        .await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_raw(&resp, output_format.is_minify()),
        OutputFormat::Table => {
            crate::output::json::print_json_raw(&resp, false)?;
            Ok(())
        }
    }
}
