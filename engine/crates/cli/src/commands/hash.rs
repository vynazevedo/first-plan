use anyhow::Result;
use clap::Args as ClapArgs;
use first_plan_core::{
    hash::hash_files_parallel,
    output::{write_json, HashOutput},
};
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::time::Instant;

#[derive(ClapArgs)]
pub struct Args {
    /// Files to hash. Repeatable.
    #[arg(long)]
    paths: Vec<PathBuf>,

    /// Read additional paths from stdin (one per line).
    #[arg(long)]
    paths_from_stdin: bool,

    /// Output JSON path. Use `-` for stdout.
    #[arg(long, default_value = "-")]
    output_json: String,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();
    let mut paths = args.paths.clone();

    if args.paths_from_stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                paths.push(PathBuf::from(trimmed));
            }
        }
    }

    if paths.is_empty() {
        anyhow::bail!("no paths provided (use --paths or --paths-from-stdin)");
    }

    let files = hash_files_parallel(&paths)?;
    let output = HashOutput::new(files, start.elapsed().as_millis() as u64);

    write_json(&output, &args.output_json)
}
