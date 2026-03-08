// F2〜F4で使用する基盤モジュール（現時点ではCLIから未参照）
#![allow(dead_code)]

mod cli;
mod detector;
mod error;
mod parser;
mod types;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
