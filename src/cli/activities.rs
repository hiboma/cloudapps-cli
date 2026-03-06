use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum ActivitiesCommand {
    /// List activities
    List(ListArgs),
    /// Fetch a single activity by ID
    Fetch {
        /// Activity ID
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

    /// Filter by username
    #[arg(long)]
    pub user: Option<String>,

    /// Filter by IP address
    #[arg(long)]
    pub ip: Option<String>,

    /// Filter by country code
    #[arg(long)]
    pub country: Option<String>,

    /// Full-text search query
    #[arg(long)]
    pub query: Option<String>,
}
