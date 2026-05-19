use crate::tty::{
    flush, output_mode, print_header, print_kv, print_section, print_warning, score_bar, OutputMode,
};
use anyhow::Result;
use clap::{Args as ClapArgs, ValueEnum};
use crossterm::style::{Color, Stylize};
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

    /// Force JSON output even when stdout is a TTY
    #[arg(long)]
    json: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Mode {
    Bm25,
    Embed,
    Hybrid,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();
    let mode_out = output_mode(args.json || args.output_json != "-");

    let hits: Vec<SearchHit> = match args.mode {
        Mode::Bm25 => search(&args.db_path, &args.query, args.limit)?,
        Mode::Embed => embed_only(&args.db_path, &args.query, args.limit)?,
        Mode::Hybrid => hybrid(&args.db_path, &args.query, args.limit, args.alpha)?,
    };

    let elapsed = start.elapsed().as_millis() as u64;

    if mode_out == OutputMode::Pretty {
        render_pretty(&args.query, &args.mode, &hits, elapsed);
        Ok(())
    } else {
        let output = SearchOutput::new(args.query, args.limit as u32, hits, elapsed);
        write_json(&output, &args.output_json)
    }
}

fn render_pretty(query: &str, mode: &Mode, hits: &[SearchHit], elapsed: u64) {
    let mode_label = match mode {
        Mode::Bm25 => "BM25",
        Mode::Embed => "Embeddings",
        Mode::Hybrid => "Hybrid",
    };
    print_header(&format!("Search: {} [{}]", query, mode_label));

    if hits.is_empty() {
        print_warning("No matches found");
        return;
    }

    let max_score = hits
        .iter()
        .map(|h| h.score)
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    print_section(&format!("Top {} results", hits.len()));
    println!();

    for (i, hit) in hits.iter().enumerate() {
        let rank = format!("#{}", i + 1).bold().with(Color::Yellow);
        let name = hit.symbol.name.as_str().bold().with(Color::Cyan);
        let kind = format!("[{:?}]", hit.symbol.kind).to_lowercase();
        let bar = score_bar(hit.score, max_score, 12);
        let score_text = format!("{:.2}", hit.score);

        println!(
            "  {} {} {} {} {}",
            rank,
            name,
            kind.with(Color::DarkGrey),
            bar,
            score_text.with(Color::Green)
        );
        println!(
            "      {} {}:{}",
            "in".dim(),
            hit.symbol.path.as_str().with(Color::White),
            hit.symbol.line.to_string().with(Color::Yellow)
        );
        if !hit.matched_tokens.is_empty() {
            let tokens = hit.matched_tokens.join(", ");
            println!("      {} {}", "matched:".dim(), tokens.as_str().dim());
        }
        if let Some(doc) = &hit.symbol.doc {
            let preview = doc.lines().next().unwrap_or(doc);
            let preview = if preview.len() > 70 {
                format!("{}...", &preview[..70])
            } else {
                preview.to_string()
            };
            println!("      {} {}", "doc:".dim(), preview.as_str().dim());
        }
        println!();
    }

    print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
    flush();
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
