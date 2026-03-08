mod analyzer;
mod cli;
mod detector;
mod error;
mod parser;
mod renderer;
mod types;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
