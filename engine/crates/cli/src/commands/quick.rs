use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::quick::{glance, render_markdown};
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Project root.
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Output rendered markdown (for piping into a file) instead of pretty output.
    #[arg(long)]
    pub markdown: bool,

    /// Force JSON output even when stdout is a TTY.
    #[arg(long)]
    pub json: bool,

    /// Write rendered markdown to this path (typically `.first-plan/quick/00-glance.md`).
    #[arg(long)]
    pub output: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let mode = output_mode(args.json);
    let report = glance(&args.root);

    if let Some(out_path) = &args.output {
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let md = render_markdown(&report);
        std::fs::write(out_path, md)?;
    }

    if args.markdown {
        println!("{}", render_markdown(&report));
        return Ok(());
    }

    if mode == OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    print_header(&format!("Quick glance ({}ms)", report.elapsed_ms));

    print_section("Stacks");
    if report.stacks.is_empty() {
        println!("  {}", "nenhum manifest reconhecido".dim());
    } else {
        for s in &report.stacks {
            println!(
                "  {} {}",
                s.language.as_str().bold().with(Color::Cyan),
                format!("({})", s.manifest).dim()
            );
        }
    }

    if !report.entry_points.is_empty() {
        print_section("Entry points");
        for e in &report.entry_points {
            println!("  {}", e.as_str().with(Color::White));
        }
    }

    if !report.top_symbols.is_empty() {
        print_section(&format!("Top symbols ({})", report.top_symbols.len()));
        for sym in report.top_symbols.iter().take(15) {
            println!(
                "  {} {} {}:{}",
                sym.name.as_str().bold().with(Color::Cyan),
                format!("[{}]", sym.kind).dim(),
                sym.file.as_str().with(Color::White),
                sym.line.to_string().with(Color::Yellow)
            );
        }
        if report.top_symbols.len() > 15 {
            println!(
                "  {}",
                format!("... and {} more", report.top_symbols.len() - 15).dim()
            );
        }
    }

    if let Some(git) = &report.git_activity {
        if !git.recent_commits.is_empty() {
            print_section("Recent commits");
            for c in git.recent_commits.iter().take(5) {
                println!("  {}", c.as_str().with(Color::DarkGrey));
            }
        }
        if !git.hot_files.is_empty() {
            print_section("Hot files (90d)");
            for h in &git.hot_files {
                println!(
                    "  {} {}",
                    h.path.as_str().with(Color::White),
                    format!("({}x)", h.change_count).dim()
                );
            }
        }
        if !git.active_authors.is_empty() {
            print_section("Active authors");
            for a in &git.active_authors {
                println!(
                    "  {} {}",
                    a.name.as_str().with(Color::White),
                    format!("({})", a.commit_count).dim()
                );
            }
        }
    }

    if report.conventions.naming.is_some() || !report.conventions.test_frameworks.is_empty() {
        print_section("Conventions (heuristic)");
        if let Some(n) = &report.conventions.naming {
            print_kv("Naming", n, Color::White);
        }
        for f in &report.conventions.test_frameworks {
            println!("  {} {}", "test:".dim(), f.as_str().with(Color::White));
        }
    }

    if !report.commands.is_empty() {
        print_section("Suggested commands");
        for c in report.commands.iter().take(8) {
            println!("  {}", c.as_str().with(Color::Cyan));
        }
    }

    if let Some(out_path) = &args.output {
        println!();
        print_kv_bold("Saved to", &out_path.to_string_lossy(), Color::Green);
    }

    print_kv(
        "Elapsed",
        &format!("{}ms", report.elapsed_ms),
        Color::DarkGrey,
    );
    flush();
    Ok(())
}
