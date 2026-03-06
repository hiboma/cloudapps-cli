use clap::{CommandFactory, Parser};
use std::process;

use cloudapps::auth::token::TokenAuth;
use cloudapps::cli::{Cli, Commands};
use cloudapps::client::CloudAppsClient;
use cloudapps::config::{Config, resolve_value};
use cloudapps::error::AppError;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(e.exit_code());
    }
}

async fn run(cli: Cli) -> Result<(), AppError> {
    let command = match cli.command {
        Some(command) => command,
        None => {
            Cli::command().print_help().ok();
            return Ok(());
        }
    };

    if cli.help_for_ai {
        let help = cloudapps::help_for_ai::get_help(&command);
        print!("{}", help);
        return Ok(());
    }

    let config = Config::load().unwrap_or_default();

    let api_url = resolve_value(
        cli.api_url.as_deref(),
        "CLOUDAPPS_API_URL",
        config.api.url.as_deref(),
    )
    .ok_or_else(|| {
        AppError::Config(
            "API URL not set. Use --api-url, CLOUDAPPS_API_URL, or config file.".to_string(),
        )
    })?;

    let token = resolve_value(
        cli.token.as_deref(),
        "CLOUDAPPS_API_TOKEN",
        config.auth.token.as_deref(),
    )
    .ok_or_else(|| {
        AppError::Auth(
            "API token not set. Use --token, CLOUDAPPS_API_TOKEN, or config file.".to_string(),
        )
    })?;

    let auth = TokenAuth::new(token)?;
    let client = CloudAppsClient::new(api_url, Box::new(auth))?;

    match &command {
        Commands::Activities {
            command: Some(command),
        } => cloudapps::commands::activities::handle(&client, command, cli.output, cli.raw).await,
        Commands::Alerts {
            command: Some(command),
        } => cloudapps::commands::alerts::handle(&client, command, cli.output, cli.raw).await,
        Commands::Entities {
            command: Some(command),
        } => cloudapps::commands::entities::handle(&client, command, cli.output, cli.raw).await,
        Commands::Files {
            command: Some(command),
        } => cloudapps::commands::files::handle(&client, command, cli.output, cli.raw).await,
        Commands::DataEnrichment {
            command: Some(command),
        } => {
            cloudapps::commands::data_enrichment::handle(&client, command, cli.output, cli.raw)
                .await
        }
        _ => {
            Cli::command()
                .find_subcommand(command.name())
                .expect("subcommand must exist")
                .clone()
                .print_help()
                .ok();
            Ok(())
        }
    }
}
