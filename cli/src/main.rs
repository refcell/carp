use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod api;
mod auth;
mod commands;
mod config;
mod utils;

use commands::{healthcheck, new, publish, pull, search};
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
}

#[derive(Subcommand)]
enum Commands {
    /// Check the health status of the API
    Healthcheck,

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
        /// Agent name in format 'name' or 'name@version'
        agent: String,

        #[arg(short, long, help = "Target directory")]
        output: Option<String>,

        #[arg(long, help = "Force overwrite existing files")]
        force: bool,
    },

    /// Publish an agent to the registry
    Publish {
        #[arg(short, long, help = "Path to agent manifest")]
        manifest: Option<String>,

        #[arg(long, help = "Skip confirmation prompts")]
        yes: bool,

        #[arg(long, help = "Perform a dry run without publishing")]
        dry_run: bool,
    },

    /// Create a new agent template
    New {
        /// Name of the new agent
        name: String,

        #[arg(short, long, help = "Target directory")]
        path: Option<String>,

        #[arg(long, help = "Agent template type")]
        template: Option<String>,
    },
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
        Commands::Publish {
            manifest,
            yes,
            dry_run,
        } => publish::execute(manifest, yes, dry_run, cli.verbose).await,
        Commands::New {
            name,
            path,
            template,
        } => new::execute(name, path, template, cli.verbose).await,
    }
}
