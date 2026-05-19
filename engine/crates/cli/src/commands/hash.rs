use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, print_success,
    OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::Color;
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

    /// Force JSON output even when stdout is a TTY
    #[arg(long)]
    json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let start = Instant::now();
    let mode = output_mode(args.json || args.output_json != "-");

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
    let elapsed = start.elapsed().as_millis() as u64;

    if mode == OutputMode::Pretty {
        print_header("File Hashing");
        print_section("Stats");
        print_kv_bold("Files hashed", &files.len().to_string(), Color::Green);
        print_kv(
            "Total bytes",
            &files
                .values()
                .map(|f| f.size_bytes)
                .sum::<u64>()
                .to_string(),
            Color::White,
        );
        print_kv("Algorithm", "xxh3_64", Color::DarkGrey);
        print_kv("Elapsed", &format!("{}ms", elapsed), Color::DarkGrey);
        println!();
        print_success("Hashing complete");
        flush();
        Ok(())
    } else {
        let output = HashOutput::new(files, elapsed);
        write_json(&output, &args.output_json)
    }
}
