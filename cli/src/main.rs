use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod api;
mod auth;
mod commands;
mod config;
mod utils;

use auth::AuthManager;
use commands::{healthcheck, list, pull, search, upload};
use utils::error::CarpResult;

#[derive(Parser)]
#[command(
    name = "carp",
    version = env!("CARGO_PKG_VERSION"),
    about = "Command-line tool for the Claude Agent Registry Portal",
    long_about = "Carp is a CLI tool for discovering, pulling, and publishing Claude AI agents from the registry."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Enable verbose output")]
    verbose: bool,

    #[arg(long, global = true, help = "Suppress all output except errors")]
    quiet: bool,

    #[arg(
        long,
        global = true,
        env = "CARP_API_KEY",
        hide_env_values = true,
        help = "API key for authentication (can also be set via CARP_API_KEY environment variable)"
    )]
    api_key: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the health status of the API
    Healthcheck,

    /// List all available agents in the registry
    List,

    /// Search for agents in the registry
    Search {
        /// Search query
        query: String,

        #[arg(short, long, help = "Number of results to show")]
        limit: Option<usize>,

        #[arg(long, help = "Show only exact matches")]
        exact: bool,
    },

    /// Pull an agent from the registry
    Pull {
        /// Agent name in format 'name' or 'name@version' (optional - if not provided, shows interactive selection)
        agent: Option<String>,

        #[arg(short, long, help = "Target directory")]
        output: Option<String>,

        #[arg(long, help = "Force overwrite existing files")]
        force: bool,
    },

    /// Upload agents from the local filesystem to the registry
    Upload {
        #[arg(
            short,
            long,
            help = "Directory to scan for agents (prompts if not provided)"
        )]
        directory: Option<String>,
    },

    /// Authentication commands
    Auth {
        #[command(subcommand)]
        auth_command: AuthCommands,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Set API key for authentication
    SetApiKey,
    /// Show authentication status
    Status,
    /// Clear stored API key (logout)
    Logout,
    /// Legacy login command (deprecated)
    Login,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

async fn run(cli: Cli) -> CarpResult<()> {
    match cli.command {
        Commands::Healthcheck => healthcheck::execute(cli.verbose).await,
        Commands::List => list::execute(cli.verbose).await,
        Commands::Search {
            query,
            limit,
            exact,
        } => search::execute(query, limit, exact, cli.verbose).await,
        Commands::Pull {
            agent,
            output,
            force,
        } => pull::execute(agent, output, force, cli.verbose).await,
        Commands::Upload { directory } => {
            upload::execute(directory, cli.api_key, cli.verbose).await
        }
        Commands::Auth { auth_command } => match auth_command {
            AuthCommands::SetApiKey => AuthManager::set_api_key().await,
            AuthCommands::Status => AuthManager::status_with_key(cli.api_key.as_deref()).await,
            AuthCommands::Logout => AuthManager::logout().await,
            AuthCommands::Login => AuthManager::set_api_key().await,
        },
    }
}
