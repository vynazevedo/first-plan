use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ImpactArgs {
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}

pub fn impact(args: ImpactArgs) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&first_plan_core::impact::analyze(&args.root)?)?
    );
    Ok(())
}

#[derive(Args)]
pub struct DeploymentArgs {
    #[arg(long, default_value = ".", global = true)]
    pub root: PathBuf,
    #[command(subcommand)]
    pub op: DeploymentOp,
}

#[derive(Subcommand)]
pub enum DeploymentOp {
    /// Show explicit observations; Git tags alone never establish deployment.
    Status,
    /// Record evidence supplied by a deployment pipeline or operator.
    Record {
        #[arg(long)]
        environment: String,
        #[arg(long)]
        commit: String,
        #[arg(long)]
        source: String,
        /// RFC3339 observation time; defaults to now.
        #[arg(long)]
        observed_at: Option<String>,
    },
}

pub fn deployment(args: DeploymentArgs) -> Result<()> {
    if let DeploymentOp::Record {
        environment,
        commit,
        source,
        observed_at,
    } = args.op
    {
        first_plan_core::deployment::record(
            &args.root,
            first_plan_core::deployment::Deployment {
                environment,
                commit,
                source,
                observed_at: observed_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            },
        )?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&first_plan_core::deployment::inspect(&args.root))?
    );
    Ok(())
}
