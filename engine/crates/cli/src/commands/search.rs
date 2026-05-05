use anyhow::Result;
use clap::Args as ClapArgs;
use first_plan_core::{
    output::{write_json, SearchOutput},
    search::search,
};
use std::path::PathBuf;
use std::time::Instant;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to the SQLite index built by `index`
    #[arg(long, default_value = ".first-plan/cache/search.db")]
    db_path: PathBuf,

    /// Query string (free text - tokenized like identifiers)
    #[arg(long)]
    query: String,

    /// Maximum number of hits
    #[arg(long, default_value_t = 10)]
    limit: usize,

    /// Output JSON path. Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();

    let hits = search(&args.db_path, &args.query, args.limit)?;

    let output = SearchOutput::new(
        args.query,
        args.limit as u32,
        hits,
        start.elapsed().as_millis() as u64,
    );

    write_json(&output, &args.output_json)
}
