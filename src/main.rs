mod commands;
mod git;
mod models;
mod storage;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Lore - A reasoning engine for code
///
/// Stores the "why" behind code changes, making it easy for future developers
/// (human or AI) to understand the context and reasoning behind decisions.
#[derive(Parser)]
#[command(name = "lore")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Lore repository
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Default agent/author ID
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Record reasoning for code changes
    Record {
        /// Brief description of intent/purpose
        #[arg(short, long)]
        message: Option<String>,

        /// Full reasoning trace/chain-of-thought
        #[arg(short, long)]
        trace: Option<String>,

        /// File containing the reasoning trace
        #[arg(long)]
        trace_file: Option<PathBuf>,

        /// Specific files to record (auto-detects from git if not specified)
        #[arg(short, long, action = clap::ArgAction::Append)]
        file: Vec<String>,

        /// Agent/author ID (overrides default)
        #[arg(short, long)]
        agent: Option<String>,

        /// Rejected alternatives (can be used multiple times)
        #[arg(short, long, action = clap::ArgAction::Append)]
        rejected: Vec<String>,

        /// Tags for categorization (can be used multiple times)
        #[arg(short = 'T', long, action = clap::ArgAction::Append)]
        tag: Vec<String>,

        /// Line range in format "start-end" (e.g., "10-45")
        #[arg(short, long)]
        lines: Option<String>,

        /// Read reasoning trace from stdin
        #[arg(long)]
        stdin: bool,
    },

    /// Explain the reasoning behind a file
    Explain {
        /// File to explain
        file: String,

        /// Show all history, not just most recent
        #[arg(short, long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Limit number of entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Search through reasoning history
    Search {
        /// Search query (searches intent, reasoning, rejected alternatives)
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Filter by file path (substring match)
        #[arg(short, long)]
        file: Option<String>,

        /// Filter by agent ID (substring match)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// List all recorded entries
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Limit number of entries to show
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show Lore status for the current repository
    Status,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path, agent } => commands::init::execute(path, agent),

        Commands::Record {
            message,
            trace,
            trace_file,
            file,
            agent,
            rejected,
            tag,
            lines,
            stdin,
        } => {
            let line_range = lines.and_then(|l| {
                let parts: Vec<&str> = l.split('-').collect();
                if parts.len() == 2 {
                    let start = parts[0].parse().ok()?;
                    let end = parts[1].parse().ok()?;
                    Some((start, end))
                } else {
                    None
                }
            });

            commands::record::execute(commands::record::RecordOptions {
                message,
                trace,
                trace_file,
                files: file,
                agent_id: agent,
                rejected,
                tags: tag,
                line_range,
                stdin,
            })
        }

        Commands::Explain {
            file,
            all,
            json,
            limit,
        } => commands::explain::execute(commands::explain::ExplainOptions {
            file,
            all,
            json,
            limit,
        }),

        Commands::Search {
            query,
            json,
            limit,
            file,
            agent,
        } => commands::search::execute(commands::search::SearchOptions {
            query,
            json,
            limit,
            file_filter: file,
            agent_filter: agent,
        }),

        Commands::List { json, limit } => {
            commands::list::execute(commands::list::ListOptions { json, limit })
        }

        Commands::Status => commands::status::execute(),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
