use clap::{Subcommand, ValueEnum};

use crate::config::credential_store::KEY_API_TOKEN;

/// The `cloudapps-cli credentials` subcommand — manages the API token
/// stored in the macOS Keychain.
///
/// `get` is intentionally absent. There is no legitimate workflow that
/// requires reading the plaintext value back out, and exposing one invites
/// accidental leakage into shell history, terminal scrollback, AI-agent
/// transcripts, and PR descriptions. Operators who need to confirm a
/// token should re-issue it from the Microsoft Defender for Cloud Apps
/// portal.
#[derive(Subcommand)]
pub enum CredentialsCommand {
    /// Store a credential in the OS credential store.
    Set {
        #[arg(value_enum)]
        field: CredentialField,
        /// Read the value from stdin instead of prompting.
        #[arg(long)]
        stdin: bool,
    },
    /// Delete a credential from the OS credential store.
    Delete {
        #[arg(value_enum)]
        field: CredentialField,
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Show whether credentials are stored (never prints the value).
    Status,
    /// Migrate credentials from credentials.toml into the OS credential store.
    Migrate {
        /// Print what would happen without making changes.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CredentialField {
    /// Microsoft Defender for Cloud Apps API token.
    ApiToken,
}

impl CredentialField {
    pub fn key(self) -> &'static str {
        match self {
            CredentialField::ApiToken => KEY_API_TOKEN,
        }
    }
}
