//! Explicit deployment observations, separate from Git release history.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub environment: String,
    pub commit: String,
    pub observed_at: String,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeploymentReport {
    pub observations: Vec<Deployment>,
    pub warnings: Vec<String>,
    pub status: String,
}

pub fn inspect(root: &Path) -> DeploymentReport {
    let path = root.join(".first-plan/deployments.json");
    let mut report = DeploymentReport {
        status: "unknown".into(),
        ..Default::default()
    };
    if !path.exists() {
        return report;
    }
    match std::fs::read_to_string(&path)
        .map_err(anyhow::Error::from)
        .and_then(|s| Ok(serde_json::from_str::<Vec<Deployment>>(&s)?))
    {
        Ok(records) => {
            for record in records {
                if validate(root, &record).is_err() {
                    report
                        .warnings
                        .push(format!("Invalid observation for {}", record.environment));
                } else {
                    let age = chrono::DateTime::parse_from_rfc3339(&record.observed_at).unwrap();
                    if (chrono::Utc::now() - age.with_timezone(&chrono::Utc)).num_hours() >= 24 {
                        report.warnings.push(format!(
                            "Stale observation for {} (24h or older)",
                            record.environment
                        ));
                    }
                    report.observations.push(record);
                }
            }
            if !report.observations.is_empty() {
                report.status = "observed_not_live_verified".into();
            }
        }
        Err(e) => report
            .warnings
            .push(format!("Cannot read deployment evidence: {}", e)),
    }
    report
}

fn validate(root: &Path, observation: &Deployment) -> Result<()> {
    ensure!(
        !observation.environment.trim().is_empty() && !observation.source.trim().is_empty(),
        "environment and evidence source are required"
    );
    ensure!(
        [40, 64].contains(&observation.commit.len())
            && observation.commit.chars().all(|c| c.is_ascii_hexdigit()),
        "commit must be a full Git object ID"
    );
    let observed = chrono::DateTime::parse_from_rfc3339(&observation.observed_at)?;
    ensure!(
        observed <= chrono::Utc::now(),
        "observation cannot be in the future"
    );
    let result = Command::new("git")
        .args(["cat-file", "-t", &observation.commit])
        .current_dir(root)
        .output()?;
    ensure!(
        result.status.success() && result.stdout == b"commit\n",
        "deployment commit not available locally; fetch it first"
    );
    Ok(())
}

pub fn record(root: &Path, observation: Deployment) -> Result<()> {
    validate(root, &observation)?;
    let path = root.join(".first-plan/deployments.json");
    let mut records: Vec<Deployment> = if path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&path)?)
            .context("invalid existing deployment evidence; not overwriting")?
    } else {
        vec![]
    };
    records.retain(|r| r.environment != observation.environment);
    records.push(observation);
    records.sort_by(|a, b| a.environment.cmp(&b.environment));
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, serde_json::to_string_pretty(&records)?)?;
    Ok(())
}
