use crate::tty::{
    flush, output_mode, print_header, print_kv, print_kv_bold, print_section, OutputMode,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::generate::{generate, list_adapters};
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    /// Tool adapter: codex, cursor, copilot, cline, generic, all
    #[arg(long, default_value = "generic")]
    pub tool: String,

    /// Project root (contains .first-plan/)
    #[arg(long, default_value = ".")]
    pub root: PathBuf,

    /// Output directory (default: same as --root)
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// List available adapters and exit
    #[arg(long)]
    pub list: bool,

    /// Force JSON output
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let mode = output_mode(args.json);

    if args.list {
        return list_command(mode);
    }

    let tools: Vec<String> = if args.tool == "all" {
        list_adapters().into_iter().map(|a| a.name).collect()
    } else {
        vec![args.tool.clone()]
    };

    let mut all_reports = Vec::new();
    for tool in &tools {
        let report = generate(&args.root, tool, args.output.as_deref())?;
        all_reports.push(report);
    }

    if mode == OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&all_reports)?);
        return Ok(());
    }

    render_pretty(&all_reports);
    Ok(())
}

fn list_command(mode: OutputMode) -> Result<()> {
    let adapters = list_adapters();
    if mode == OutputMode::Json {
        println!("{}", serde_json::to_string_pretty(&adapters)?);
        return Ok(());
    }
    print_header("Available adapters");
    for adapter in &adapters {
        println!(
            "  {} - {}",
            adapter.name.as_str().bold().with(Color::Cyan),
            adapter.description.as_str().dim()
        );
        for f in &adapter.output_files {
            println!("    output: {}", f.as_str().with(Color::White));
        }
    }
    println!();
    print_kv(
        "Usage",
        "first-plan-engine generate --tool <name>",
        Color::White,
    );
    print_kv(
        "Or all",
        "first-plan-engine generate --tool all",
        Color::White,
    );
    flush();
    Ok(())
}

fn render_pretty(reports: &[first_plan_core::generate::GenerateReport]) {
    print_header(&format!("Generated {} adapter(s)", reports.len()));
    for report in reports {
        print_section(&format!("{} ({}ms)", report.tool, report.elapsed_ms));
        print_kv_bold(
            "Files written",
            &report.files_written.len().to_string(),
            Color::Green,
        );
        print_kv("Bytes", &report.bytes_written.to_string(), Color::DarkGrey);
        for f in &report.files_written {
            println!("  {} {}", "wrote:".dim(), f.as_str().with(Color::Cyan));
        }
    }
    println!();
    flush();
}
