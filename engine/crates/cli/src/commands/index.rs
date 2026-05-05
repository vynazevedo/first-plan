use anyhow::Result;
use clap::Args as ClapArgs;
use first_plan_core::{
    index::{build_index, collect_symbols},
    output::{write_json, IndexOutput},
};
use std::path::PathBuf;
use std::time::Instant;

#[derive(ClapArgs)]
pub struct Args {
    /// Project root to scan
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// Path to write the SQLite index
    #[arg(long, default_value = ".first-plan/cache/search.db")]
    db_path: PathBuf,

    /// Output JSON path (stats). Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();

    let symbols = collect_symbols(&args.repo)?;
    let stats = build_index(&args.db_path, &symbols)?;

    let output = IndexOutput::new(
        args.repo.to_string_lossy().into_owned(),
        args.db_path.to_string_lossy().into_owned(),
        stats,
        start.elapsed().as_millis() as u64,
    );

    write_json(&output, &args.output_json)
}
