use crate::cli::policies::{FetchArgs, PoliciesCommand};
use crate::client::CloudAppsClient;
use crate::error::AppError;
use crate::output::OutputFormat;

pub async fn handle(
    client: &CloudAppsClient,
    command: &PoliciesCommand,
    output_format: OutputFormat,
    _raw: bool,
) -> Result<(), AppError> {
    match command {
        PoliciesCommand::List => list(client, output_format).await,
        PoliciesCommand::Fetch(args) => fetch(client, args, output_format).await,
    }
}

async fn list(client: &CloudAppsClient, output_format: OutputFormat) -> Result<(), AppError> {
    let resp: serde_json::Value = client.get("/api/v1/policies/").await?.json().await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => {
            crate::output::json::print_json_raw(&resp, output_format.is_minify())
        }
        OutputFormat::Table => {
            print_policies_table(&resp);
            Ok(())
        }
    }
}

fn print_policies_table(value: &serde_json::Value) {
    use crate::output::table::truncate;

    println!(
        "{:<26} {:<12} {:<10} {:<50}",
        "ID", "TYPE", "ENABLED", "NAME"
    );

    // The API returns a JSON array directly (not wrapped in {data: [...]}).
    let items = value
        .as_array()
        .or_else(|| value.get("data").and_then(|d| d.as_array()));

    if let Some(data) = items {
        for item in data {
            let id = item.get("_id").and_then(|v| v.as_str()).unwrap_or("-");
            let policy_type = item
                .get("policyType")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let enabled = item
                .get("enabled")
                .and_then(|v| v.as_bool())
                .map(|b| if b { "true" } else { "false" })
                .unwrap_or("-");
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("-");

            println!(
                "{:<26} {:<12} {:<10} {:<50}",
                truncate(id, 24),
                truncate(policy_type, 10),
                enabled,
                truncate(name, 48),
            );
        }
    }
}

async fn fetch(
    client: &CloudAppsClient,
    args: &FetchArgs,
    output_format: OutputFormat,
) -> Result<(), AppError> {
    let path = format!(
        "/api/v1/policy/{}/{}/",
        args.r#type.api_path_segment(),
        args.id
    );
    let resp: serde_json::Value = client.get(&path).await?.json().await?;

    match output_format {
        OutputFormat::Json | OutputFormat::JsonMinify => {
            crate::output::json::print_json_raw(&resp, output_format.is_minify())
        }
        OutputFormat::Table => crate::output::json::print_json_raw(&resp, false),
    }
}
