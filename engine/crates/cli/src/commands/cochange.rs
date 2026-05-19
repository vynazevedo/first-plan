use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_warning,
    strength_color, OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::{
    cochange::{build_matrix, CoChangeMatrix, Filters},
    git::parse_log,
    output::{write_json, CoChangeOutput},
};
use std::path::PathBuf;
use std::time::Instant;

#[derive(ClapArgs)]
pub struct Args {
    /// Path to the git repository
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// History window in days
    #[arg(long, default_value_t = 180)]
    since: u32,

    /// Minimum commits per file to consider for pairing
    #[arg(long, default_value_t = 5)]
    min_occurrences: u32,

    /// Minimum co-change ratio (0.0..1.0) to keep a pair
    #[arg(long, default_value_t = 0.5)]
    min_ratio: f32,

    /// Patterns to exclude (in addition to defaults). Repeatable.
    #[arg(long)]
    exclude: Vec<String>,

    /// Output JSON path. Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,

    /// Force JSON output even when stdout is a TTY
    #[arg(long)]
    json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(args.json || args.output_json != "-");

    if mode == OutputMode::Pretty {
        print_header(&format!("Co-change Graph ({} days)", args.since));
        eprintln!("  {}", "Parsing git log...".dim());
        flush();
    }

    let commits = parse_log(&args.repo, args.since)?;
    let total_commits = commits.len() as u32;

    let mut filters = Filters {
        min_occurrences: args.min_occurrences,
        min_ratio: args.min_ratio,
        exclude_patterns: Filters::default().exclude_patterns,
    };
    filters.exclude_patterns.extend(args.exclude);

    let matrix = build_matrix(&commits, &filters);
    let elapsed = start.elapsed().as_millis() as u64;

    if mode == OutputMode::Pretty {
        render_pretty(&matrix, total_commits, args.since, elapsed);
        Ok(())
    } else {
        let output = CoChangeOutput::new(
            args.repo.to_string_lossy().into_owned(),
            args.since,
            total_commits,
            matrix.total_files_analyzed,
            matrix.pairs,
            matrix.clusters,
            elapsed,
        );
        write_json(&output, &args.output_json)
    }
}

fn render_pretty(matrix: &CoChangeMatrix, total_commits: u32, window_days: u32, elapsed: u64) {
    print_section("Summary");
    print_kv("Commits analyzed", &total_commits.to_string(), Color::White);
    print_kv(
        "Files analyzed",
        &matrix.total_files_analyzed.to_string(),
        Color::White,
    );
    print_kv_bold(
        "Pairs detected",
        &matrix.pairs.len().to_string(),
        Color::Green,
    );
    print_kv_bold(
        "Clusters detected",
        &matrix.clusters.len().to_string(),
        Color::Green,
    );
    print_kv("Window", &format!("{} days", window_days), Color::DarkGrey);
    print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);

    if matrix.pairs.is_empty() {
        print_warning("No co-change pairs found above threshold");
        return;
    }

    print_section("Top pairs");
    let top = matrix.pairs.iter().take(10);
    for pair in top {
        let strength = format!("{:?}", pair.strength).to_lowercase();
        let strength_text = format!("[{}]", strength).with(strength_color(&strength));
        let ratio_text = format!("{:.2}", pair.co_change_ratio).bold();
        println!(
            "  {} {} {} {} {}",
            ratio_text,
            strength_text,
            pair.file_a.as_str().with(Color::Cyan),
            "<->".dim(),
            pair.file_b.as_str().with(Color::Cyan)
        );
    }
    if matrix.pairs.len() > 10 {
        println!(
            "  {}",
            format!("... and {} more pairs", matrix.pairs.len() - 10).dim()
        );
    }

    if !matrix.clusters.is_empty() {
        print_section("Clusters");
        for cluster in &matrix.clusters {
            println!(
                "  {} {} files | cohesion {:.2}",
                cluster.id.as_str().bold().with(Color::Yellow),
                cluster.files.len(),
                cluster.internal_cohesion
            );
            for file in cluster.files.iter().take(5) {
                println!("    - {}", file.as_str().with(Color::DarkGrey));
            }
            if cluster.files.len() > 5 {
                println!(
                    "    {}",
                    format!("... and {} more", cluster.files.len() - 5).dim()
                );
            }
        }
    }
    println!();
}
