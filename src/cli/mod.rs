pub mod activities;
pub mod alerts;
pub mod data_enrichment;
pub mod entities;
pub mod files;

use clap::{Parser, Subcommand};
use std::env;

use crate::output::OutputFormat;

fn token_help() -> String {
    if env::var("CLOUDAPPS_API_TOKEN").is_ok() {
        "API token [env: CLOUDAPPS_API_TOKEN=****]".to_string()
    } else {
        "API token [env: CLOUDAPPS_API_TOKEN=]".to_string()
    }
}

#[derive(Parser)]
#[command(
    name = "cloudapps",
    version,
    about = "CLI tool for Microsoft Defender for Cloud Apps REST API",
    subcommand_required = false,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// API base URL
    #[arg(long, env = "CLOUDAPPS_API_URL", global = true)]
    pub api_url: Option<String>,

    #[arg(long, env = "CLOUDAPPS_API_TOKEN", global = true, hide_env = true, help = token_help())]
    pub token: Option<String>,

    /// Output format
    #[arg(
        long,
        env = "CLOUDAPPS_OUTPUT_FORMAT",
        global = true,
        default_value = "json"
    )]
    pub output: OutputFormat,

    /// Output raw API response without extracting data
    #[arg(long, global = true)]
    pub raw: bool,

    /// Enable verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Show AI-friendly help (specification markdown) for the specified resource
    #[arg(long, global = true)]
    pub help_for_ai: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage activities
    #[command(subcommand_required = false, arg_required_else_help = true)]
    Activities {
        #[command(subcommand)]
        command: Option<activities::ActivitiesCommand>,
    },
    /// Manage alerts
    #[command(subcommand_required = false, arg_required_else_help = true)]
    Alerts {
        #[command(subcommand)]
        command: Option<alerts::AlertsCommand>,
    },
    /// Manage entities
    #[command(subcommand_required = false, arg_required_else_help = true)]
    Entities {
        #[command(subcommand)]
        command: Option<entities::EntitiesCommand>,
    },
    /// Manage files
    #[command(subcommand_required = false, arg_required_else_help = true)]
    Files {
        #[command(subcommand)]
        command: Option<files::FilesCommand>,
    },
    /// Manage IP address ranges (data enrichment)
    #[command(name = "data-enrichment", subcommand_required = false, arg_required_else_help = true)]
    DataEnrichment {
        #[command(subcommand)]
        command: Option<data_enrichment::DataEnrichmentCommand>,
    },
}

impl Commands {
    pub fn name(&self) -> &'static str {
        match self {
            Commands::Activities { .. } => "activities",
            Commands::Alerts { .. } => "alerts",
            Commands::Entities { .. } => "entities",
            Commands::Files { .. } => "files",
            Commands::DataEnrichment { .. } => "data-enrichment",
        }
    }
}
