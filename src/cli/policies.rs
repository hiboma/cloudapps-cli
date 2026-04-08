use clap::{Args, Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum PoliciesCommand {
    /// List all policies
    List(ListArgs),
    /// Fetch a single policy by type and ID
    Fetch(FetchArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Output format filter (not sent to API, just for completeness)
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Args)]
pub struct FetchArgs {
    /// Policy type
    #[arg(long, value_enum)]
    pub r#type: PolicyType,

    /// Policy ID
    #[arg(long)]
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum PolicyType {
    Activity,
    Anomaly,
    Discovery,
    DiscoveryAnomaly,
    File,
    AppPermissions,
    Session,
}

impl PolicyType {
    pub fn api_path_segment(&self) -> &'static str {
        match self {
            PolicyType::Activity => "activity",
            PolicyType::Anomaly => "anomaly",
            PolicyType::Discovery => "discovery",
            PolicyType::DiscoveryAnomaly => "discovery_anomaly",
            PolicyType::File => "file",
            PolicyType::AppPermissions => "app_permissions",
            PolicyType::Session => "session",
        }
    }
}

impl std::fmt::Display for PolicyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.api_path_segment())
    }
}
