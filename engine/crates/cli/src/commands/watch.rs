use anyhow::Result;
use clap::Args as ClapArgs;
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
}

pub fn run(args: Args) -> Result<()> {
    let debounce = Duration::from_secs(args.debounce_seconds);
    let repo = args.repo.clone();
    let exec_cmd = args.exec.clone();

    let callback = Box::new(move |event: WatchEvent| {
        // Emit event as JSON line on stdout (stream-friendly).
        match serde_json::to_string(&event) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("serialize event: {}", e),
        }

        // Optional exec command per event
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

    eprintln!(
        "watching {} with {}s debounce (Ctrl-C to stop)",
        repo.display(),
        debounce.as_secs()
    );

    watch(&repo, debounce, callback)
}
