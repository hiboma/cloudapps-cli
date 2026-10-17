use clap::{CommandFactory, Parser};
use clap_complete::{generate, shells};
use std::io;
use std::process;

use cloudapps::auth::token::TokenAuth;
use cloudapps::cli::{Cli, Commands, CompletionShell};
use cloudapps::client::CloudAppsClient;
use cloudapps::config::CloudAppsCredentials;
use cloudapps::error::AppError;

fn main() {
    let cli = Cli::parse();

    // For subcommands that do not talk to the API (`credentials`,
    // `completion`), skip credential resolution entirely. `resolve()`
    // consults the OS credential store, which on macOS may prompt or
    // fail with an ACL error; letting that output leak above
    // `credentials status` would be confusing since `status` is
    // precisely how the user inspects the store.
    //
    // `credentials` itself is Unix-only (the Keychain backend only
    // compiles on macOS; the rest of Unix gets an `UnsupportedStore`
    // that always reports Unavailable, which is still useful for
    // smoke-testing the subcommand surface).
    let skips_resolve = match &cli.command {
        Some(Commands::Completion { .. }) => true,
        // `doctor` must NOT scrub CLOUDAPPS_* before running, or its
        // ENVIRONMENT section would always report the credential vars as
        // (unset). It does its own source-attribution and never forwards a
        // resolved token to the API, so skipping resolution here is safe.
        Some(Commands::Doctor { .. }) => true,
        #[cfg(unix)]
        Some(Commands::Credentials { .. }) => true,
        _ => false,
    };

    // Resolve credentials before creating the runtime.
    let credentials = if skips_resolve {
        CloudAppsCredentials::default()
    } else {
        CloudAppsCredentials::resolve(cli.api_url.as_deref())
    };

    // Scrub CLOUDAPPS_* from this process's environment immediately after
    // resolution so API invocations shed the secret from the kernel's
    // process environment snapshot (visible via `ps -E` /
    // `/proc/<pid>/environ`) for the lifetime of the process. Runs while
    // we are still single-threaded (before tokio runtime creation).
    //
    // SAFETY: Single-threaded context (before tokio runtime creation).
    if !skips_resolve {
        unsafe {
            CloudAppsCredentials::clear_env();
        }
    }

    // Create the tokio runtime after credentials are resolved and scrubbed.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = run(cli, credentials).await {
            eprintln!("Error: {}", e);
            process::exit(e.exit_code());
        }
    });
}

async fn run(cli: Cli, credentials: CloudAppsCredentials) -> Result<(), AppError> {
    let command = match cli.command {
        Some(command) => command,
        None => {
            Cli::command().print_help().ok();
            return Ok(());
        }
    };

    // Handle completion generation (no credentials required).
    if let Commands::Completion { shell } = &command {
        print_completion(*shell);
        return Ok(());
    }

    // Handle doctor diagnostics. Runs before the API
    // credential check because it does its own source-attribution against
    // the environment, the credential store, and credentials.toml — it does
    // not build an API client. Connectivity is probed by default.
    if let Commands::Doctor { no_connectivity } = &command {
        return cloudapps::commands::doctor::handle(!no_connectivity).await;
    }

    // Handle credentials subcommand (does not talk to the API).
    // Unix-only: the Keychain backend is only compiled on macOS and the
    // Unix file-permission APIs (chmod 0600 for backups / tempfiles) do
    // not compile on Windows.
    #[cfg(unix)]
    if let Commands::Credentials { command: creds_cmd } = &command {
        return cloudapps::commands::credentials::handle(creds_cmd);
    }

    if cli.help_for_ai {
        let help = cloudapps::help_for_ai::get_help(&command);
        print!("{}", help);
        return Ok(());
    }

    // Require API credentials.
    let api_url = credentials.api_url.as_ref().ok_or_else(|| {
        AppError::Config("API URL not set. Use --api-url or CLOUDAPPS_API_URL.".to_string())
    })?;

    let token = credentials.api_token.as_ref().ok_or_else(|| {
        AppError::Auth(
            "API token not set. Set CLOUDAPPS_API_TOKEN environment variable.".to_string(),
        )
    })?;

    let auth = TokenAuth::new(token.clone())?;
    let client = CloudAppsClient::new(api_url.clone(), Box::new(auth))?;

    match &command {
        Commands::Activities {
            command: Some(command),
        } => cloudapps::commands::activities::handle(&client, command, cli.output, cli.raw).await,
        Commands::Alerts {
            command: Some(command),
        } => cloudapps::commands::alerts::handle(&client, command, cli.output, cli.raw).await,
        Commands::Entities {
            command: Some(command),
        } => cloudapps::commands::entities::handle(&client, command, cli.output, cli.raw).await,
        Commands::Files {
            command: Some(command),
        } => cloudapps::commands::files::handle(&client, command, cli.output, cli.raw).await,
        Commands::DataEnrichment {
            command: Some(command),
        } => {
            cloudapps::commands::data_enrichment::handle(&client, command, cli.output, cli.raw)
                .await
        }
        Commands::Policies {
            command: Some(command),
        } => cloudapps::commands::policies::handle(&client, command, cli.output, cli.raw).await,
        _ => {
            Cli::command()
                .find_subcommand(command.name())
                .expect("subcommand must exist")
                .clone()
                .print_help()
                .ok();
            Ok(())
        }
    }
}

fn print_completion(shell: CompletionShell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let mut stdout = io::stdout();
    match shell {
        CompletionShell::Bash => generate(shells::Bash, &mut cmd, name, &mut stdout),
        CompletionShell::Zsh => generate(shells::Zsh, &mut cmd, name, &mut stdout),
        CompletionShell::Fish => generate(shells::Fish, &mut cmd, name, &mut stdout),
        CompletionShell::PowerShell => generate(shells::PowerShell, &mut cmd, name, &mut stdout),
        CompletionShell::Elvish => generate(shells::Elvish, &mut cmd, name, &mut stdout),
    }
}
