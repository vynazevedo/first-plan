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
    /// Build a BM25 search index of code symbols.
    Index(commands::index::Args),
    /// Query the BM25 index with a natural-language string.
    Search(commands::search::Args),
    /// Watch filesystem and emit debounced events for relevant source changes.
    Watch(commands::watch::Args),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Cochange(args) => commands::cochange::run(args),
        Command::Hash(args) => commands::hash::run(args),
        Command::Index(args) => commands::index::run(args),
        Command::Search(args) => commands::search::run(args),
        Command::Watch(args) => commands::watch::run(args),
    }
}
