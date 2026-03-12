use clap::Parser;
use std::sync::OnceLock;
use tokio::sync::Mutex;

use crate::auth::token::TokenAuth;
use crate::cli::{Cli, Commands};
use crate::client::CloudAppsClient;
use crate::config::resolve_value;
use crate::error::AppError;

/// Global mutex to serialize stdout capture across concurrent requests.
/// gag::BufferRedirect operates on process-level fd 1, so only one
/// capture can be active at a time.
static STDOUT_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn stdout_mutex() -> &'static Mutex<()> {
    STDOUT_MUTEX.get_or_init(|| Mutex::new(()))
}

/// Dispatch a command from a CLI args vector (used by agent handler).
/// Captures stdout output and returns it as a string.
pub async fn dispatch_from_args(args: &[String]) -> Result<String, AppError> {
    let cli = Cli::try_parse_from(args).map_err(|e| AppError::InvalidInput(e.to_string()))?;

    let command = match cli.command {
        Some(command) => command,
        None => return Ok(String::new()),
    };

    let api_url = resolve_value(cli.api_url.as_deref(), "CLOUDAPPS_API_URL").ok_or_else(|| {
        AppError::Config("API URL not set. Use --api-url or CLOUDAPPS_API_URL.".to_string())
    })?;

    let token = std::env::var("CLOUDAPPS_API_TOKEN").map_err(|_| {
        AppError::Auth(
            "API token not set. Set CLOUDAPPS_API_TOKEN environment variable.".to_string(),
        )
    })?;

    let auth = TokenAuth::new(token)?;
    let client = CloudAppsClient::new(api_url, Box::new(auth))?;

    dispatch_command(&client, &command, cli.output, cli.raw).await
}

/// Dispatch a command and capture its output as a string.
/// Uses a mutex to serialize stdout capture across concurrent requests.
async fn dispatch_command(
    client: &CloudAppsClient,
    command: &Commands,
    output_format: crate::output::OutputFormat,
    raw: bool,
) -> Result<String, AppError> {
    // Acquire mutex to prevent concurrent stdout captures.
    let _guard = stdout_mutex().lock().await;

    // Redirect stdout to capture output.
    let buf = gag::BufferRedirect::stdout()
        .map_err(|e| AppError::Config(format!("failed to capture stdout: {}", e)))?;

    let result = match command {
        Commands::Activities { command: Some(cmd) } => {
            crate::commands::activities::handle(client, cmd, output_format, raw).await
        }
        Commands::Alerts { command: Some(cmd) } => {
            crate::commands::alerts::handle(client, cmd, output_format, raw).await
        }
        Commands::Entities { command: Some(cmd) } => {
            crate::commands::entities::handle(client, cmd, output_format, raw).await
        }
        Commands::Files { command: Some(cmd) } => {
            crate::commands::files::handle(client, cmd, output_format, raw).await
        }
        Commands::DataEnrichment { command: Some(cmd) } => {
            crate::commands::data_enrichment::handle(client, cmd, output_format, raw).await
        }
        _ => Ok(()),
    };

    // Read captured output.
    let mut output = String::new();
    use std::io::Read;
    let mut reader = buf;
    reader
        .read_to_string(&mut output)
        .map_err(|e| AppError::Config(format!("failed to read captured output: {}", e)))?;

    // _guard is dropped here, releasing the mutex.
    result?;
    Ok(output)
}
