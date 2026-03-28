//! # updown
//!
//! Command-line interface for the [updown.io](https://updown.io) website monitoring API.
//!
//! Supports full CRUD for checks, recipients, and status pages, plus read access to
//! monitoring nodes and downtime history. Output defaults to Axi format (token-efficient
//! TOON format); pass `--table` for human-readable table output or `--json` for
//! machine-readable output.
//!
//! ## Configuration
//!
//! The API key is resolved in this order:
//! 1. `--api-key` flag
//! 2. `UPDOWN_API_KEY` environment variable
//! 3. `~/.config/updown/config.toml`

mod cmd;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;
use updown_lib::client::{ApiError, Client};
use updown_lib::config::Config;

/// Top-level CLI entry point parsed by clap.
#[derive(Parser)]
#[command(name = "updown", about = "CLI for the updown.io monitoring API")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON
    #[arg(long, global = true, conflicts_with = "table")]
    pub json: bool,

    /// Output as human-readable table
    #[arg(long, global = true, conflicts_with = "json")]
    pub table: bool,

    /// API key (overrides config file and env var)
    #[arg(long, global = true)]
    pub api_key: Option<String>,
}

/// Top-level subcommands, each mapping to an updown.io API resource.
#[derive(Subcommand)]
pub enum Commands {
    /// Manage monitoring checks
    Checks {
        #[command(subcommand)]
        action: cmd::checks::ChecksAction,
    },
    /// View monitoring node locations and IPs
    Nodes {
        #[command(subcommand)]
        action: cmd::nodes::NodesAction,
    },
    /// Manage alert recipients
    Recipients {
        #[command(subcommand)]
        action: cmd::recipients::RecipientsAction,
    },
    /// Manage public status pages
    #[command(name = "status-pages")]
    StatusPages {
        #[command(subcommand)]
        action: cmd::status_pages::StatusPagesAction,
    },
}

fn main() {
    let cli = Cli::parse();
    let mode = output::OutputMode::from_flags(cli.json, cli.table);

    if let Err(e) = run(cli, mode) {
        // In AXI mode, known API errors go to stdout with exit 0
        if mode == output::OutputMode::Axi {
            if let Some(api_err) = e.downcast_ref::<ApiError>() {
                output::print_axi_error(api_err.status_code, &api_err.message);
                return;
            }
        }

        eprintln!("Error: {:#}", e);

        let exit_code = if format!("{:#}", e).contains("No API key found")
            || format!("{:#}", e).contains("Failed to parse config")
            || format!("{:#}", e).contains("HOME environment variable")
        {
            2
        } else {
            1
        };
        process::exit(exit_code);
    }
}

/// Resolves configuration, builds the HTTP client, and dispatches to the appropriate command.
///
/// Returns an error if the API key is missing, the HTTP client cannot be constructed,
/// or the subcommand itself fails.
fn run(cli: Cli, mode: output::OutputMode) -> Result<()> {
    let config = Config::load(cli.api_key.as_deref())?;
    let client = Client::new(config.api_key, config.base_url)?;

    match cli.command {
        Commands::Checks { action } => cmd::checks::run(action, &client, mode),
        Commands::Nodes { action } => cmd::nodes::run(action, &client, mode),
        Commands::Recipients { action } => cmd::recipients::run(action, &client, mode),
        Commands::StatusPages { action } => cmd::status_pages::run(action, &client, mode),
    }
}
