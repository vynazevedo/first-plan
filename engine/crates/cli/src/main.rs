use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "first-plan-engine",
    version,
    about = "Native engine for first-plan plugin"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build the co-change graph from git history.
    Cochange(commands::cochange::Args),
    /// Hash files in parallel using xxh3.
    Hash(commands::hash::Args),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Cochange(args) => commands::cochange::run(args),
        Command::Hash(args) => commands::hash::run(args),
    }
}
