use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum FilesCommand {
    /// List files
    List(ListArgs),
    /// Fetch a single file by ID
    Fetch {
        /// File ID
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

    /// Filter by service app ID
    #[arg(long)]
    pub service: Option<i64>,

    /// Filter by file type: other, document, spreadsheet, presentation, text, image, folder
    #[arg(long)]
    pub filetype: Option<String>,

    /// Filter by file name
    #[arg(long)]
    pub filename: Option<String>,

    /// Filter by file extension
    #[arg(long)]
    pub extension: Option<String>,

    /// Filter by sharing level: private, internal, external, public, internet
    #[arg(long)]
    pub sharing: Option<String>,
}
