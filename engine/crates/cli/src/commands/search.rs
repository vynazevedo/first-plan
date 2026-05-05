use anyhow::Result;
use clap::{Args as ClapArgs, ValueEnum};
use first_plan_core::{
    output::{write_json, SearchOutput},
    search::{search, SearchHit},
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

    /// Search mode: bm25 (default), embed (cosine only), hybrid (BM25 + cosine)
    #[arg(long, value_enum, default_value_t = Mode::Bm25)]
    mode: Mode,

    /// Hybrid alpha: weight of BM25 (0.0 = pure cosine, 1.0 = pure BM25). Default 0.3.
    #[arg(long, default_value_t = 0.3)]
    alpha: f32,

    /// Output JSON path. Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Mode {
    Bm25,
    Embed,
    Hybrid,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();

    let hits: Vec<SearchHit> = match args.mode {
        Mode::Bm25 => search(&args.db_path, &args.query, args.limit)?,
        Mode::Embed => embed_only(&args.db_path, &args.query, args.limit)?,
        Mode::Hybrid => hybrid(&args.db_path, &args.query, args.limit, args.alpha)?,
    };

    let output = SearchOutput::new(
        args.query,
        args.limit as u32,
        hits,
        start.elapsed().as_millis() as u64,
    );

    write_json(&output, &args.output_json)
}

#[cfg(feature = "ml")]
fn embed_only(db_path: &std::path::Path, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    let provider = first_plan_core::embeddings::make_default_provider()?;
    let q = vec![query.to_string()];
    let mut embeddings = provider.embed_batch(&q)?;
    let q_emb = embeddings
        .pop()
        .ok_or_else(|| anyhow::anyhow!("empty embedding result"))?;
    first_plan_core::search::search_embed(db_path, &q_emb, limit)
}

#[cfg(not(feature = "ml"))]
fn embed_only(_db_path: &std::path::Path, _query: &str, _limit: usize) -> Result<Vec<SearchHit>> {
    anyhow::bail!(
        "--mode embed requires the ML-enabled build. Use --mode bm25 instead, or install \
         the binary with the '-ml' suffix."
    )
}

#[cfg(feature = "ml")]
fn hybrid(
    db_path: &std::path::Path,
    query: &str,
    limit: usize,
    alpha: f32,
) -> Result<Vec<SearchHit>> {
    let provider = first_plan_core::embeddings::make_default_provider()?;
    let q = vec![query.to_string()];
    let mut embeddings = provider.embed_batch(&q)?;
    let q_emb = embeddings
        .pop()
        .ok_or_else(|| anyhow::anyhow!("empty embedding result"))?;
    first_plan_core::search::search_hybrid(db_path, query, &q_emb, limit, alpha)
}

#[cfg(not(feature = "ml"))]
fn hybrid(
    _db_path: &std::path::Path,
    _query: &str,
    _limit: usize,
    _alpha: f32,
) -> Result<Vec<SearchHit>> {
    anyhow::bail!(
        "--mode hybrid requires the ML-enabled build. Use --mode bm25 instead, or install \
         the binary with the '-ml' suffix."
    )
}
