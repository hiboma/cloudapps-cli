pub mod activities;
#[cfg(unix)]
pub mod agent;
pub mod alerts;
#[cfg(unix)]
pub mod credentials;
pub mod data_enrichment;
pub mod entities;
pub mod files;
pub mod policies;

use clap::{Parser, Subcommand, ValueEnum};

use crate::output::OutputFormat;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Elvish,
}

#[derive(Parser)]
#[command(
    name = "cloudapps-cli",
    version,
    about = "CLI tool for Microsoft Defender for Cloud Apps REST API",
    subcommand_required = false,
    arg_required_else_help = true,
    subcommand_help_heading = "Resources",
    after_help = "System:\n  agent       Manage the credential isolation agent (Unix only)\n  completion  Generate shell completion script"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// API base URL
    #[arg(long, env = "CLOUDAPPS_API_URL", global = true, hide = true)]
    pub api_url: Option<String>,

    /// Output format
    #[arg(
        long,
        env = "CLOUDAPPS_OUTPUT_FORMAT",
        global = true,
        default_value = "json",
        hide = true
    )]
    pub output: OutputFormat,

    /// Output raw API response without extracting data
    #[arg(long, global = true, hide = true)]
    pub raw: bool,

    /// Enable verbose output
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Show AI-friendly help (specification markdown) for the specified resource
    #[arg(long, global = true)]
    pub help_for_ai: bool,

    /// Agent socket path (hidden, set by agent start)
    #[arg(long, env = "CLOUDAPPS_AGENT_SOCKET", global = true, hide = true)]
    pub socket: Option<String>,

    /// Agent session token (hidden, set by agent start)
    #[arg(long, env = "CLOUDAPPS_AGENT_TOKEN", global = true, hide = true)]
    pub token: Option<String>,

    /// Skip agent auto-detection and use direct API mode
    #[arg(long, global = true, hide = true)]
    pub no_agent: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    // -- Resources --
    /// Manage activities
    #[command(
        visible_alias = "activity",
        subcommand_required = false,
        arg_required_else_help = true
    )]
    Activities {
        #[command(subcommand)]
        command: Option<activities::ActivitiesCommand>,
    },
    /// Manage alerts
    #[command(
        visible_alias = "alert",
        subcommand_required = false,
        arg_required_else_help = true
    )]
    Alerts {
        #[command(subcommand)]
        command: Option<alerts::AlertsCommand>,
    },
    /// Manage entities
    #[command(
        visible_alias = "entity",
        subcommand_required = false,
        arg_required_else_help = true
    )]
    Entities {
        #[command(subcommand)]
        command: Option<entities::EntitiesCommand>,
    },
    /// Manage files
    #[command(
        visible_alias = "file",
        subcommand_required = false,
        arg_required_else_help = true
    )]
    Files {
        #[command(subcommand)]
        command: Option<files::FilesCommand>,
    },
    /// Manage IP address ranges (data enrichment)
    #[command(
        name = "data-enrichment",
        visible_aliases = ["data-enrich", "enrichment"],
        subcommand_required = false,
        arg_required_else_help = true
    )]
    DataEnrichment {
        #[command(subcommand)]
        command: Option<data_enrichment::DataEnrichmentCommand>,
    },
    /// Manage policies (undocumented API)
    #[command(
        visible_alias = "policy",
        subcommand_required = false,
        arg_required_else_help = true
    )]
    Policies {
        #[command(subcommand)]
        command: Option<policies::PoliciesCommand>,
    },

    // -- System --
    /// Manage API token stored in the OS credential store (macOS Keychain)
    #[cfg(unix)]
    #[command(
        subcommand_required = true,
        arg_required_else_help = true,
        long_about = "Manage the Microsoft Defender for Cloud Apps API token stored in the OS credential store.\n\
                      \n\
                      On macOS this is the login Keychain, service `dev.cloudapps-cli`, account `api_token`.\n\
                      \n\
                      `get` is intentionally absent — there is no legitimate workflow that requires \n\
                      reading the plaintext token back out."
    )]
    Credentials {
        #[command(subcommand)]
        command: credentials::CredentialsCommand,
    },

    /// Manage the credential isolation agent (Unix only)
    #[cfg(unix)]
    #[command(subcommand_required = true, arg_required_else_help = true, hide = true)]
    Agent {
        #[command(subcommand)]
        command: agent::AgentCommand,
    },

    /// Generate shell completion script
    #[command(hide = true)]
    Completion {
        /// Target shell
        #[arg(value_enum)]
        shell: CompletionShell,
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
            Commands::Policies { .. } => "policies",
            #[cfg(unix)]
            Commands::Credentials { .. } => "credentials",
            #[cfg(unix)]
            Commands::Agent { .. } => "agent",
            Commands::Completion { .. } => "completion",
        }
    }
}
