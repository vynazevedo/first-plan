use anyhow::Result;
use clap::Args as ClapArgs;
use first_plan_core::{
    cochange::{build_matrix, Filters},
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
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();

    let commits = parse_log(&args.repo, args.since)?;
    let total_commits = commits.len() as u32;

    let mut filters = Filters {
        min_occurrences: args.min_occurrences,
        min_ratio: args.min_ratio,
        exclude_patterns: Filters::default().exclude_patterns,
    };
    filters.exclude_patterns.extend(args.exclude);

    let matrix = build_matrix(&commits, &filters);

    let output = CoChangeOutput::new(
        args.repo.to_string_lossy().into_owned(),
        args.since,
        total_commits,
        matrix.total_files_analyzed,
        matrix.pairs,
        matrix.clusters,
        start.elapsed().as_millis() as u64,
    );

    write_json(&output, &args.output_json)
}
