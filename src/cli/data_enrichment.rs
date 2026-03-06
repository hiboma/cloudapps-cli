use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum DataEnrichmentCommand {
    /// List IP address ranges
    List(ListArgs),
    /// Create a new IP address range
    Create(CreateArgs),
    /// Update an existing IP address range
    Update(UpdateArgs),
    /// Delete an IP address range
    Delete {
        /// IP range ID
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

    /// Filter by category: corporate, administrative, risky, vpn, cloud-provider, other
    #[arg(long)]
    pub category: Option<String>,

    /// Filter by tag ID
    #[arg(long)]
    pub tag: Option<String>,

    /// Show only built-in ranges
    #[arg(long)]
    pub builtin: bool,

    /// Show only custom ranges
    #[arg(long)]
    pub custom: bool,
}

#[derive(Args)]
pub struct CreateArgs {
    /// Name of the IP range
    #[arg(long, required = true)]
    pub name: String,

    /// Comma-separated list of subnets (e.g., "192.168.1.0/24,10.0.0.0/8")
    #[arg(long, required = true)]
    pub subnets: String,

    /// Category: corporate, administrative, risky, vpn, cloud-provider, other
    #[arg(long, required = true)]
    pub category: String,

    /// Registered ISP / organization
    #[arg(long)]
    pub organization: Option<String>,

    /// Comma-separated list of tag IDs
    #[arg(long)]
    pub tags: Option<String>,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// IP range ID
    pub id: String,

    /// Name of the IP range
    #[arg(long)]
    pub name: Option<String>,

    /// Comma-separated list of subnets
    #[arg(long)]
    pub subnets: Option<String>,

    /// Category: corporate, administrative, risky, vpn, cloud-provider, other
    #[arg(long)]
    pub category: Option<String>,

    /// Registered ISP / organization
    #[arg(long)]
    pub organization: Option<String>,
}
