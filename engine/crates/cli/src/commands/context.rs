use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    #[arg(long)]
    pub query: String,
    /// Selected content budget in characters (not model tokens).
    #[arg(long, default_value_t = 8000)]
    pub budget: usize,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let pack = first_plan_core::context::build(&args.root, &args.query, args.budget)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&pack)?);
    } else {
        println!("# Context: {}\n", pack.query);
        for item in &pack.items {
            println!(
                "- [{}] {}:{} ({})\n  {}",
                item.category,
                item.evidence.path,
                item.evidence.line,
                item.evidence.hash,
                item.text
            );
        }
        for limitation in &pack.limitations {
            println!("\nNote: {}", limitation);
        }
    }
    Ok(())
}
