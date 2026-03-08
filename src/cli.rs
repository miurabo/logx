use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};

use crate::analyzer;
use crate::detector::FormatDetector;
use crate::parser;
use crate::renderer::Renderer;

#[derive(Parser)]
#[command(
    name = "logx",
    version,
    about = "Zero-config log investigation CLI tool"
)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Disable color output
    #[arg(long, global = true)]
    no_color: bool,
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

pub fn run() -> Result<()> {
    let args = Args::parse();
    let renderer = Renderer::new(!args.no_color);

    match args.command {
        Command::Scan { .. } => {
            eprintln!("logx scan: not yet implemented");
        }
        Command::Errors { files, since } => {
            run_errors(&renderer, &files, since.as_deref())?;
        }
        Command::Filter { .. } => {
            eprintln!("logx filter: not yet implemented");
        }
    }

    Ok(())
}

fn run_errors(renderer: &Renderer, files: &[PathBuf], since: Option<&str>) -> Result<()> {
    let since_duration = since.map(analyzer::parse_duration).transpose()?;
    let cutoff = since_duration.map(|d| Local::now().fixed_offset() - d);

    let mut all_errors = Vec::new();

    for path in files {
        let file =
            File::open(path).with_context(|| format!("Failed to open: {}", path.display()))?;
        let mut reader = BufReader::new(file);

        // フォーマット検出（先頭行を読むためリーダーを消費する）
        let detection = FormatDetector::detect(&mut reader);

        // ファイルを再度開いてパース
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let log_parser = parser::create_parser(&detection.format);

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };

            let Some(entry) = log_parser.parse_line(&line, line_number + 1) else {
                continue;
            };

            if !analyzer::is_error(&entry) {
                continue;
            }

            // --since フィルタ
            if let Some(ref cutoff_time) = cutoff
                && let Some(ts) = entry.timestamp
                && ts < *cutoff_time
            {
                continue;
            }

            all_errors.push(entry);
        }
    }

    let summary = analyzer::summarize_errors(&all_errors);
    renderer.render_errors(&all_errors, &summary, since);

    Ok(())
}
