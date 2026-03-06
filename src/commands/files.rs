use crate::cli::files::FilesCommand;
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::models::file::{FileType, SharingLevel};
use crate::models::filter::{FilterCondition, FilterRequest};
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &FilesCommand,
    output_format: OutputFormat,
    raw: bool,
) -> Result<(), AppError> {
    match command {
        FilesCommand::List(args) => list(client, args, output_format, raw).await,
        FilesCommand::Fetch { id } => fetch(client, id, output_format).await,
    }
}

async fn list(
    client: &CloudAppsClient,
    args: &crate::cli::files::ListArgs,
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

    if let Some(ref service) = args.service {
        filter.add_filter("service", FilterCondition::eq(vec![*service]));
    }
    if let Some(ref filetype) = args.filetype {
        let ft = FileType::from_str_loose(filetype)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown file type: {}", filetype)))?;
        filter.add_filter("fileType", FilterCondition::eq(vec![ft.to_api()]));
    }
    if let Some(ref filename) = args.filename {
        filter.add_filter("filename", FilterCondition::eq(vec![filename.as_str()]));
    }
    if let Some(ref extension) = args.extension {
        filter.add_filter("extension", FilterCondition::eq(vec![extension.as_str()]));
    }
    if let Some(ref sharing) = args.sharing {
        let sl = SharingLevel::from_str_loose(sharing)
            .ok_or_else(|| AppError::InvalidInput(format!("unknown sharing level: {}", sharing)))?;
        filter.add_filter("sharing", FilterCondition::eq(vec![sl.to_api()]));
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
        let data: Vec<serde_json::Value> = client.list_all("/api/v1/files/", filter).await?;
        serde_json::json!({ "data": data, "total": data.len() })
    } else {
        client.post("/api/v1/files/", &filter).await?.json().await?
    };

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_data(&resp, raw, output_format.is_minify()),
        OutputFormat::Table => {
            print_files_table(&resp);
            Ok(())
        }
    }
}

fn print_files_table(value: &serde_json::Value) {
    use crate::output::table::{format_timestamp, truncate};

    println!(
        "{:<20} {:<30} {:<14} {:<10} {:<20} {:<24}",
        "ID", "FILENAME", "TYPE", "SHARING", "OWNER", "MODIFIED"
    );

    if let Some(data) = value.get("data").and_then(|d| d.as_array()) {
        for item in data {
            let id = item.get("_id").and_then(|i| i.as_str()).unwrap_or("-");
            let filename = item.get("fileName").and_then(|f| f.as_str()).unwrap_or("-");
            let filetype = item
                .get("fileType")
                .and_then(|f| f.as_i64())
                .and_then(|v| FileType::from_api(v as i32))
                .map(|f| f.to_string())
                .unwrap_or_else(|| "-".to_string());
            let sharing = item
                .get("sharing")
                .and_then(|s| s.as_i64())
                .and_then(|v| SharingLevel::from_api(v as i32))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string());
            let owner = item
                .get("ownerName")
                .and_then(|o| o.as_str())
                .unwrap_or("-");
            let modified = format_timestamp(item.get("modifiedDate").and_then(|m| m.as_i64()));

            println!(
                "{:<20} {:<30} {:<14} {:<10} {:<20} {:<24}",
                truncate(id, 18),
                truncate(filename, 28),
                filetype,
                sharing,
                truncate(owner, 18),
                modified
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
        .get(&format!("/api/v1/files/{}/", id))
        .await?
        .json()
        .await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => crate::output::json::print_json_raw(&resp, output_format.is_minify()),
        OutputFormat::Table => crate::output::json::print_json_raw(&resp, false),
    }
}
