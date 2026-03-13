use clap::{CommandFactory, Parser};
use std::path::PathBuf;
use std::process;

use cloudapps::auth::token::TokenAuth;
#[cfg(unix)]
use cloudapps::cli::agent::AgentCommand;
use cloudapps::cli::{Cli, Commands};
use cloudapps::client::CloudAppsClient;
use cloudapps::config::resolve_value;
use cloudapps::error::AppError;

fn main() {
    let cli = Cli::parse();

    // Handle agent start (fork) before creating tokio runtime.
    // fork() is unsafe in multi-threaded processes, so we must do it here.
    #[cfg(unix)]
    if let Some(Commands::Agent {
        command:
            AgentCommand::Start {
                socket,
                config,
                foreground,
            },
    }) = &cli.command
        && !foreground
    {
        let session_token = cloudapps::agent::generate_token();
        let socket_path = socket.as_ref().map(PathBuf::from);
        let config_path = config.as_ref().map(PathBuf::from);

        if let Err(e) = cloudapps::agent::ensure_socket_dir() {
            eprintln!("Error: failed to create socket directory: {}", e);
            process::exit(1);
        }

        match cloudapps::agent::server::fork_into_background(
            socket_path,
            config_path,
            session_token.clone(),
        ) {
            Ok((child_pid, socket_path)) => {
                cloudapps::agent::server::print_shell_vars(&socket_path, &session_token, child_pid);
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error: failed to start agent: {}", e);
                process::exit(1);
            }
        }
    }

    // Create tokio runtime for all other operations.
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = run(cli).await {
            eprintln!("Error: {}", e);
            process::exit(e.exit_code());
        }
    });
}

async fn run(cli: Cli) -> Result<(), AppError> {
    let command = match cli.command {
        Some(command) => command,
        None => {
            Cli::command().print_help().ok();
            return Ok(());
        }
    };

    // Handle agent subcommands.
    #[cfg(unix)]
    if let Commands::Agent { command: agent_cmd } = &command {
        return handle_agent_command(agent_cmd).await;
    }

    if cli.help_for_ai {
        let help = cloudapps::help_for_ai::get_help(&command);
        print!("{}", help);
        return Ok(());
    }

    // Check if we should route through the agent.
    #[cfg(unix)]
    if let Some(ref agent_token) = cli.token {
        let socket_path = cli
            .socket
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(cloudapps::agent::resolve_socket_path);

        return route_through_agent(&command, &socket_path, agent_token).await;
    }

    // Direct execution: require API credentials.
    let api_url = resolve_value(cli.api_url.as_deref(), "CLOUDAPPS_API_URL").ok_or_else(|| {
        AppError::Config("API URL not set. Use --api-url or CLOUDAPPS_API_URL.".to_string())
    })?;

    let token = std::env::var("CLOUDAPPS_API_TOKEN").map_err(|_| {
        AppError::Auth(
            "API token not set. Set CLOUDAPPS_API_TOKEN environment variable.".to_string(),
        )
    })?;

    let auth = TokenAuth::new(token)?;
    let client = CloudAppsClient::new(api_url, Box::new(auth))?;

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

/// Handle agent subcommands (start foreground, stop, status).
#[cfg(unix)]
async fn handle_agent_command(cmd: &AgentCommand) -> Result<(), AppError> {
    match cmd {
        AgentCommand::Start {
            socket,
            config,
            foreground,
        } => {
            // Foreground mode (background is handled before tokio runtime).
            assert!(*foreground, "background mode should be handled in main()");

            let session_token = cloudapps::agent::generate_token();
            let socket_path = socket.as_ref().map(PathBuf::from);
            let config_path = config.as_ref().map(PathBuf::from);

            cloudapps::agent::ensure_socket_dir()
                .map_err(|e| AppError::Config(format!("failed to create socket dir: {}", e)))?;

            let pid = std::process::id();
            let actual_socket =
                socket_path.unwrap_or_else(|| cloudapps::agent::pid_socket_path(pid));

            cloudapps::agent::server::print_shell_vars(&actual_socket, &session_token, pid);

            cloudapps::agent::server::start(Some(actual_socket), config_path, &session_token)
                .await
                .map_err(|e| AppError::Config(format!("agent error: {}", e)))?;

            Ok(())
        }
        AgentCommand::Stop { socket, all } => {
            let msg = if *all {
                cloudapps::agent::client::stop_all()?
            } else {
                let socket_path = socket
                    .as_ref()
                    .map(PathBuf::from)
                    .unwrap_or_else(cloudapps::agent::resolve_socket_path);
                cloudapps::agent::client::stop(&socket_path)?
            };
            println!("{}", msg);
            Ok(())
        }
        AgentCommand::Status { socket } => {
            let socket_path = socket
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(cloudapps::agent::resolve_socket_path);
            let msg = cloudapps::agent::client::status(&socket_path).await?;
            println!("{}", msg);
            Ok(())
        }
    }
}

/// Route a command through the agent via UDS.
#[cfg(unix)]
async fn route_through_agent(
    command: &Commands,
    socket_path: &std::path::Path,
    agent_token: &str,
) -> Result<(), AppError> {
    let (cmd_name, action, args) = extract_command_args(command);

    let output =
        cloudapps::agent::client::send_command(&cmd_name, &action, &args, socket_path, agent_token)
            .await?;

    print!("{}", output);
    Ok(())
}

/// Extract command name, action, and remaining args from a Commands variant.
/// Global flags like --output and --raw are preserved and passed to the agent.
/// Only agent-specific flags (--socket, --token) are stripped.
#[cfg(unix)]
fn extract_command_args(command: &Commands) -> (String, String, Vec<String>) {
    let cmd_name = command.name().to_string();

    let all_args: Vec<String> = std::env::args().collect();
    let mut action = String::new();
    let mut extra_args = Vec::new();
    let mut found_command = false;

    // Only strip agent-specific flags that the server should not see.
    // Global flags like --output, --raw, --api-url are passed through
    // so the agent can honor the requested output format.
    let strip_flags_with_value = ["--socket", "--token"];
    let strip_flags_bool: [&str; 0] = [];

    let mut skip_next = false;
    for arg in all_args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }

        // Check if this flag should be stripped (exact match or --flag=value).
        let should_strip = strip_flags_with_value
            .iter()
            .any(|f| *arg == *f || arg.starts_with(&format!("{}=", f)))
            || strip_flags_bool.iter().any(|f| *arg == *f);

        if should_strip {
            // If it's a --flag value (not --flag=value), skip the next arg too.
            if strip_flags_with_value.iter().any(|f| *arg == *f) && !arg.contains('=') {
                skip_next = true;
            }
            continue;
        }

        if !found_command {
            if *arg == cmd_name || *arg == command.name() {
                found_command = true;
            }
            continue;
        }

        if action.is_empty() {
            action = arg.clone();
        } else {
            extra_args.push(arg.clone());
        }
    }

    (cmd_name, action, extra_args)
}
