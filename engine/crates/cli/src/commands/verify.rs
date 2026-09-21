use anyhow::{ensure, Result};
use clap::{Args as ClapArgs, Subcommand};
use first_plan_core::rules;
use std::path::PathBuf;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, default_value = ".", global = true)]
    pub root: PathBuf,
    #[command(subcommand)]
    pub command: Option<Operation>,
}

#[derive(Subcommand)]
pub enum Operation {
    /// Export a policy CANDIDATE for independent review; does not execute checks.
    Policy {
        #[arg(long)]
        out: PathBuf,
    },
    /// Compare current requirements/verifiers with an externally reviewed policy.
    Review {
        #[arg(long)]
        policy: PathBuf,
    },
    /// Execute repository checks explicitly authorized by an external policy.
    Run {
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Check local report freshness and consistency; does not rerun or authenticate proofs.
    Check {
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        report: PathBuf,
    },
}

pub fn run(args: Args) -> Result<()> {
    match args.command {
        None => println!(
            "{}",
            serde_json::to_string_pretty(&rules::load(&args.root)?)?
        ),
        Some(Operation::Policy { out }) => {
            let policy = rules::candidate_policy(&args.root)?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&out)?;
            file.write_all(serde_json::to_string_pretty(&policy)?.as_bytes())?;
            println!(
                "{}",
                serde_json::json!({"status":"review_required", "candidate":out, "note":"An operator must independently review and supply this policy from trusted read-only storage; generation is not approval."})
            );
        }
        Some(Operation::Review { policy }) => {
            let reviewed = rules::external_policy(&args.root, &policy)?;
            let result = rules::review(&args.root, &reviewed)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            ensure!(
                result.status == "authorized",
                "review_required: policy no longer authorizes this registry"
            );
        }
        Some(Operation::Run { policy, out }) => {
            let result = rules::run(&args.root, &policy, &out)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            ensure!(result.status == "passed", "verification {}", result.status);
        }
        Some(Operation::Check { policy, report }) => {
            let status = rules::check_report(&args.root, &report, &policy)?;
            println!(
                "{}",
                serde_json::json!({"status":status,"assurance":"local_report_consistency_only"})
            );
            ensure!(status == "passed", "verification evidence {status}");
        }
    }
    Ok(())
}
