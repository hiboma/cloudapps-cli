use crate::cli::Commands;

pub const ACTIVITIES_HELP: &str = include_str!("../docs/specifications/04-resources-activities.md");
pub const ALERTS_HELP: &str = include_str!("../docs/specifications/05-resources-alerts.md");
pub const ENTITIES_HELP: &str = include_str!("../docs/specifications/06-resources-entities.md");
pub const FILES_HELP: &str = include_str!("../docs/specifications/07-resources-files.md");
pub const DATA_ENRICHMENT_HELP: &str =
    include_str!("../docs/specifications/08-resources-data-enrichment.md");

pub fn get_help(command: &Commands) -> &'static str {
    match command {
        Commands::Activities { .. } => ACTIVITIES_HELP,
        Commands::Alerts { .. } => ALERTS_HELP,
        Commands::Entities { .. } => ENTITIES_HELP,
        Commands::Files { .. } => FILES_HELP,
        Commands::DataEnrichment { .. } => DATA_ENRICHMENT_HELP,
    }
}
