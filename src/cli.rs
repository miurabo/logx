use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "logx",
    version,
    about = "Zero-config log investigation CLI tool"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan log files and show summary
    Scan {
        /// Log file paths (supports glob)
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
    /// Extract errors from log files
    Errors {
        /// Log file paths (supports glob)
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Time range filter (e.g., 1h, 30m, 2d)
        #[arg(long)]
        since: Option<String>,
    },
    /// Filter log entries by conditions
    Filter {
        /// Log file paths
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Filter by HTTP status code (e.g., 500, 5xx)
        #[arg(long)]
        status: Option<String>,

        /// Filter by log level (e.g., error, warn)
        #[arg(long)]
        level: Option<String>,

        /// Filter by time range (e.g., 1h, 30m)
        #[arg(long)]
        since: Option<String>,

        /// Filter by request path (e.g., /api/users)
        #[arg(long)]
        path: Option<String>,

        /// Filter by IP address or CIDR (e.g., 192.168.1.0/24)
        #[arg(long)]
        ip: Option<String>,
    },
}

pub fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Scan { .. } => {
            eprintln!("logx scan: not yet implemented");
        }
        Command::Errors { .. } => {
            eprintln!("logx errors: not yet implemented");
        }
        Command::Filter { .. } => {
            eprintln!("logx filter: not yet implemented");
        }
    }

    Ok(())
}
