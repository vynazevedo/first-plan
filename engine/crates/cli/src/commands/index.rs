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

    /// Also generate semantic embeddings (requires --features=ml build)
    #[arg(long)]
    embed: bool,

    /// Output JSON path (stats). Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();

    let symbols = collect_symbols(&args.repo)?;

    let stats = if args.embed {
        embed_index(&args.db_path, &symbols)?
    } else {
        build_index(&args.db_path, &symbols)?
    };

    let output = IndexOutput::new(
        args.repo.to_string_lossy().into_owned(),
        args.db_path.to_string_lossy().into_owned(),
        stats,
        start.elapsed().as_millis() as u64,
    );

    write_json(&output, &args.output_json)
}

#[cfg(feature = "ml")]
fn embed_index(
    db_path: &std::path::Path,
    symbols: &[first_plan_core::symbols::Symbol],
) -> Result<first_plan_core::index::IndexStats> {
    let provider = first_plan_core::embeddings::make_default_provider()?;
    first_plan_core::index::build_index_with_embeddings(db_path, symbols, provider.as_ref())
}

#[cfg(not(feature = "ml"))]
fn embed_index(
    _db_path: &std::path::Path,
    _symbols: &[first_plan_core::symbols::Symbol],
) -> Result<first_plan_core::index::IndexStats> {
    anyhow::bail!(
        "--embed requires the ML-enabled build. Reinstall the binary with the \
         '-ml' suffix from the project releases, or rebuild with 'cargo install --features=ml'."
    )
}
