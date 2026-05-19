use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_success,
    OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::Color;
use first_plan_core::{
    index::{build_index, collect_symbols},
    output::{write_json, IndexOutput},
};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::{Duration, Instant};

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

    /// Force JSON output even when stdout is a TTY
    #[arg(long)]
    json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(args.json || args.output_json != "-");

    let spinner = if mode == OutputMode::Pretty {
        print_header("Building Index");
        let pb = make_spinner("Collecting symbols");
        Some(pb)
    } else {
        None
    };

    let symbols = collect_symbols(&args.repo)?;

    if let Some(pb) = &spinner {
        pb.finish_with_message(format!("Collected {} symbols", symbols.len()));
    }

    let pb2 = if mode == OutputMode::Pretty {
        if args.embed {
            Some(make_spinner("Generating embeddings (may take seconds)"))
        } else {
            Some(make_spinner("Writing index"))
        }
    } else {
        None
    };

    let stats = if args.embed {
        embed_index(&args.db_path, &symbols)?
    } else {
        build_index(&args.db_path, &symbols)?
    };

    if let Some(pb) = &pb2 {
        pb.finish_with_message("Index ready");
    }

    let elapsed = start.elapsed().as_millis() as u64;

    if mode == OutputMode::Pretty {
        print_section("Stats");
        print_kv_bold(
            "Total symbols",
            &stats.total_symbols.to_string(),
            Color::Green,
        );
        print_kv(
            "Total doc length",
            &stats.total_doc_length.to_string(),
            Color::White,
        );
        print_kv(
            "Avg doc length",
            &format!("{:.2}", stats.avg_doc_length),
            Color::White,
        );
        print_kv(
            "Embeddings",
            if stats.has_embeddings { "yes" } else { "no" },
            if stats.has_embeddings {
                Color::Green
            } else {
                Color::DarkGrey
            },
        );
        print_kv(
            "Db path",
            args.db_path.to_string_lossy().as_ref(),
            Color::DarkGrey,
        );
        print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
        println!();
        print_success("Index built successfully");
        flush();
        Ok(())
    } else {
        let output = IndexOutput::new(
            args.repo.to_string_lossy().into_owned(),
            args.db_path.to_string_lossy().into_owned(),
            stats,
            elapsed,
        );
        write_json(&output, &args.output_json)
    }
}

fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    pb
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
