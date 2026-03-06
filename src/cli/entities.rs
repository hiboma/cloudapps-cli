use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum EntitiesCommand {
    /// List entities
    List(ListArgs),
    /// Fetch a single entity by ID
    Fetch {
        /// Entity ID
        id: String,
    },
    /// Fetch an entity tree by ID
    #[command(name = "fetch-tree")]
    FetchTree {
        /// Entity ID
        id: String,
    },
}

#[derive(Args)]
pub struct ListArgs {
    /// Maximum number of results
    #[arg(long, default_value = "100")]
    pub limit: Option<u64>,

    /// Number of results to skip
    #[arg(long)]
    pub skip: Option<u64>,

    /// Fetch all results with auto-pagination
    #[arg(long)]
    pub all: bool,

    /// Raw JSON filter
    #[arg(long)]
    pub filter: Option<String>,

    /// Filter by entity type
    #[arg(long = "type")]
    pub entity_type: Option<String>,

    /// Filter admin entities only
    #[arg(long)]
    pub is_admin: bool,

    /// Filter external entities only
    #[arg(long)]
    pub is_external: bool,

    /// Filter by domain
    #[arg(long)]
    pub domain: Option<String>,

    /// Filter by status: na, staged, active, suspended, deleted
    #[arg(long)]
    pub status: Option<String>,
}
