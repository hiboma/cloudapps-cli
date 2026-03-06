use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum AlertsCommand {
    /// List alerts
    List(ListArgs),
    /// Fetch a single alert by ID
    Fetch(FetchArgs),
    /// Close an alert
    Close(CloseArgs),
    /// Mark alerts as read
    #[command(name = "mark-read")]
    MarkRead {
        /// Alert IDs
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Mark alerts as unread
    #[command(name = "mark-unread")]
    MarkUnread {
        /// Alert IDs
        #[arg(required = true)]
        ids: Vec<String>,
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

    /// Filter by severity: low, medium, high, informational
    #[arg(long)]
    pub severity: Option<String>,

    /// Filter by resolution status: open, dismissed, resolved, false-positive, benign, true-positive
    #[arg(long)]
    pub resolution: Option<String>,

    /// Show only open alerts
    #[arg(long)]
    pub open: bool,

    /// Show only closed alerts
    #[arg(long)]
    pub closed: bool,

    /// Full-text search query
    #[arg(long)]
    pub query: Option<String>,
}

#[derive(Args)]
pub struct FetchArgs {
    /// Alert ID
    pub id: String,

    /// Also fetch related activities
    #[arg(long)]
    pub with_activities: bool,
}

#[derive(Args)]
pub struct CloseArgs {
    /// Alert IDs
    #[arg(required = true)]
    pub ids: Vec<String>,

    /// Close type: benign, false-positive, true-positive
    #[arg(long = "close-as", required = true)]
    pub close_as: String,

    /// Comment for the closure
    #[arg(long)]
    pub comment: Option<String>,
}
