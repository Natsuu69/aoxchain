use aoxcmd::cli::run_cli;
use std::process;

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("AOXCMD_ERROR: {error}");
        process::exit(1);
    }
}
