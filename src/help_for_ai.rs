use crate::cli::Commands;

pub const ACTIVITIES_HELP: &str = include_str!("../docs/specifications/04-resources-activities.md");
pub const ALERTS_HELP: &str = include_str!("../docs/specifications/05-resources-alerts.md");
pub const ENTITIES_HELP: &str = include_str!("../docs/specifications/06-resources-entities.md");
pub const FILES_HELP: &str = include_str!("../docs/specifications/07-resources-files.md");
pub const DATA_ENRICHMENT_HELP: &str =
    include_str!("../docs/specifications/08-resources-data-enrichment.md");
#[cfg(unix)]
pub const AGENT_HELP: &str = include_str!("../docs/specifications/16-agent-mode.md");
pub const COMPLETION_HELP: &str = "# completion\n\nGenerate shell completion script.\n\nUsage:\n  cloudapps-cli completion <shell>\n\nSupported shells: bash, zsh, fish, powershell, elvish\n\nExamples:\n  cloudapps-cli completion zsh > \"${fpath[1]}/_cloudapps-cli\"\n  cloudapps-cli completion bash > /etc/bash_completion.d/cloudapps-cli\n";

pub const POLICIES_HELP: &str = include_str!("../docs/specifications/17-resources-policies.md");

pub fn get_help(command: &Commands) -> &'static str {
    match command {
        Commands::Activities { .. } => ACTIVITIES_HELP,
        Commands::Alerts { .. } => ALERTS_HELP,
        Commands::Entities { .. } => ENTITIES_HELP,
        Commands::Files { .. } => FILES_HELP,
        Commands::DataEnrichment { .. } => DATA_ENRICHMENT_HELP,
        Commands::Policies { .. } => POLICIES_HELP,
        #[cfg(unix)]
        Commands::Agent { .. } => AGENT_HELP,
        Commands::Completion { .. } => COMPLETION_HELP,
    }
}
