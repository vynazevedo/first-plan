use crate::tty::{is_tty, print_header, print_kv};
use anyhow::Result;
use clap::Args as ClapArgs;
use crossterm::style::{Color, Stylize};
use first_plan_core::watch::{watch, WatchEvent};
use std::path::PathBuf;
use std::time::Duration;

#[derive(ClapArgs)]
pub struct Args {
    /// Project root to monitor recursively
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    /// Debounce window in seconds (default 5; common: 5 interactive, 300 production)
    #[arg(long, default_value_t = 5)]
    debounce_seconds: u64,

    /// Optional command to spawn for each event. Use {paths} as placeholder
    /// for the affected paths (space-separated, relative to --repo).
    /// Example: --exec 'echo "stale: {paths}"'
    #[arg(long)]
    exec: Option<String>,

    /// Force JSON output even when stdout is a TTY
    #[arg(long)]
    json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let debounce = Duration::from_secs(args.debounce_seconds);
    let repo = args.repo.clone();
    let exec_cmd = args.exec.clone();
    let pretty = is_tty() && !args.json;

    if pretty {
        print_header("Watch Mode");
        print_kv("Repo", &repo.display().to_string(), Color::White);
        print_kv(
            "Debounce",
            &format!("{}s", debounce.as_secs()),
            Color::White,
        );
        if let Some(cmd) = &exec_cmd {
            print_kv("On event", cmd, Color::Yellow);
        }
        println!();
        println!(
            "{}",
            "Watching for changes (Ctrl-C to stop)..."
                .dim()
                .with(Color::White)
        );
        println!();
    } else {
        eprintln!(
            "watching {} with {}s debounce (Ctrl-C to stop)",
            repo.display(),
            debounce.as_secs()
        );
    }

    let callback = Box::new(move |event: WatchEvent| {
        if pretty {
            let ts = event.timestamp.format("%H:%M:%S").to_string();
            let lang_text = if event.languages.is_empty() {
                "?".to_string()
            } else {
                event.languages.join(",")
            };
            println!(
                "  {} {} {} files {} [{}]",
                ts.as_str().with(Color::DarkGrey),
                "Δ".bold().with(Color::Green),
                event.affected_paths.len(),
                "lang:".dim(),
                lang_text.with(Color::Cyan)
            );
            for path in &event.affected_paths {
                println!("      {}", path.as_str().with(Color::White));
            }
        } else {
            match serde_json::to_string(&event) {
                Ok(json) => println!("{}", json),
                Err(e) => eprintln!("serialize event: {}", e),
            }
        }

        if let Some(cmd) = &exec_cmd {
            let paths_arg = event.affected_paths.join(" ");
            let resolved = cmd.replace("{paths}", &paths_arg);
            if let Err(e) = std::process::Command::new("sh")
                .arg("-c")
                .arg(&resolved)
                .status()
            {
                eprintln!("exec failed: {}", e);
            }
        }
    });

    watch(&repo, debounce, callback)
}
