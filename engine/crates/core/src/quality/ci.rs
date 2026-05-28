//! CI workflow detection and parsing.
//!
//! Suporta:
//! - GitHub Actions: `.github/workflows/*.yml`
//! - GitLab CI: `.gitlab-ci.yml`
//! - CircleCI: `.circleci/config.yml`
//! - Jenkins: `Jenkinsfile` (parsing limitado - declarative pipeline detection only)
//!
//! Output: lista de jobs com triggers, runs_on, steps simplificados.

use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CiReport {
    pub providers_detected: Vec<String>,
    pub workflows: Vec<Workflow>,
    pub total_jobs: usize,
    pub triggers_summary: TriggersSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub provider: String,
    pub file: String,
    pub name: Option<String>,
    pub triggers: Vec<String>,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub name: String,
    pub runs_on: Option<String>,
    pub steps_count: usize,
    pub steps_summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TriggersSummary {
    pub on_push: bool,
    pub on_pull_request: bool,
    pub on_schedule: bool,
    pub on_release: bool,
    pub on_workflow_dispatch: bool,
    pub on_other: Vec<String>,
}

pub fn detect(root: &Path) -> CiReport {
    let mut report = CiReport::default();

    detect_github_actions(root, &mut report);
    detect_gitlab_ci(root, &mut report);
    detect_circleci(root, &mut report);
    detect_jenkins(root, &mut report);

    report.total_jobs = report.workflows.iter().map(|w| w.jobs.len()).sum();
    for wf in &report.workflows {
        for t in &wf.triggers {
            match t.as_str() {
                "push" => report.triggers_summary.on_push = true,
                "pull_request" | "merge_request" => report.triggers_summary.on_pull_request = true,
                "schedule" | "cron" => report.triggers_summary.on_schedule = true,
                "release" => report.triggers_summary.on_release = true,
                "workflow_dispatch" | "manual" => {
                    report.triggers_summary.on_workflow_dispatch = true
                }
                other => {
                    let s = other.to_string();
                    if !report.triggers_summary.on_other.contains(&s) {
                        report.triggers_summary.on_other.push(s);
                    }
                }
            }
        }
    }

    report
}

fn detect_github_actions(root: &Path, report: &mut CiReport) {
    let dir = root.join(".github/workflows");
    if !dir.exists() {
        return;
    }
    let mut found = false;
    for entry in WalkDir::new(&dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "yml" && ext != "yaml" {
            continue;
        }
        if let Some(wf) = parse_github_workflow(path, root) {
            report.workflows.push(wf);
            found = true;
        }
    }
    if found {
        report.providers_detected.push("github-actions".to_string());
    }
}

fn parse_github_workflow(path: &Path, root: &Path) -> Option<Workflow> {
    let content = std::fs::read_to_string(path).ok()?;
    let yaml: YamlValue = serde_yaml::from_str(&content).ok()?;
    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    let name = yaml.get("name").and_then(|v| v.as_str()).map(String::from);

    let triggers = parse_github_triggers(&yaml);
    let jobs = parse_github_jobs(&yaml);

    Some(Workflow {
        provider: "github-actions".to_string(),
        file: rel,
        name,
        triggers,
        jobs,
    })
}

fn parse_github_triggers(yaml: &YamlValue) -> Vec<String> {
    let mut out = Vec::new();
    let on = yaml.get("on").or_else(|| yaml.get(YamlValue::Bool(true)));
    let Some(on) = on else {
        return out;
    };
    match on {
        YamlValue::String(s) => out.push(s.clone()),
        YamlValue::Sequence(seq) => {
            for v in seq {
                if let YamlValue::String(s) = v {
                    out.push(s.clone());
                }
            }
        }
        YamlValue::Mapping(map) => {
            for (k, _) in map {
                if let YamlValue::String(s) = k {
                    out.push(s.clone());
                }
            }
        }
        _ => {}
    }
    out
}

fn parse_github_jobs(yaml: &YamlValue) -> Vec<Job> {
    let mut out = Vec::new();
    let Some(jobs) = yaml.get("jobs").and_then(|v| v.as_mapping()) else {
        return out;
    };
    for (k, v) in jobs {
        let job_name = k.as_str().unwrap_or("?").to_string();
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .map(String::from)
            .unwrap_or_else(|| job_name.clone());
        let runs_on = v
            .get("runs-on")
            .map(value_to_string)
            .unwrap_or_else(|| "unknown".to_string());
        let (steps_count, steps_summary) = match v.get("steps").and_then(|s| s.as_sequence()) {
            Some(seq) => {
                let count = seq.len();
                let summary: Vec<String> = seq
                    .iter()
                    .take(10)
                    .map(|step| {
                        step.get("name")
                            .and_then(|n| n.as_str())
                            .map(String::from)
                            .or_else(|| {
                                step.get("uses")
                                    .and_then(|u| u.as_str())
                                    .map(|u| format!("uses: {}", u))
                            })
                            .or_else(|| {
                                step.get("run").and_then(|r| r.as_str()).map(|r| {
                                    let first = r.lines().next().unwrap_or("").trim();
                                    if first.len() > 60 {
                                        format!("run: {}...", &first[..60])
                                    } else {
                                        format!("run: {}", first)
                                    }
                                })
                            })
                            .unwrap_or_else(|| "(unnamed)".to_string())
                    })
                    .collect();
                (count, summary)
            }
            None => (0, Vec::new()),
        };
        out.push(Job {
            name,
            runs_on: Some(runs_on),
            steps_count,
            steps_summary,
        });
    }
    out
}

fn value_to_string(v: &YamlValue) -> String {
    match v {
        YamlValue::String(s) => s.clone(),
        YamlValue::Sequence(seq) => seq
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        _ => format!("{:?}", v),
    }
}

fn detect_gitlab_ci(root: &Path, report: &mut CiReport) {
    let path = root.join(".gitlab-ci.yml");
    if !path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let yaml: YamlValue = match serde_yaml::from_str(&content) {
        Ok(y) => y,
        Err(_) => return,
    };

    let Some(map) = yaml.as_mapping() else {
        return;
    };

    let mut jobs = Vec::new();
    for (k, v) in map {
        let key = match k.as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        if key.starts_with('.')
            || matches!(
                key.as_str(),
                "stages"
                    | "variables"
                    | "default"
                    | "include"
                    | "workflow"
                    | "image"
                    | "services"
                    | "before_script"
                    | "after_script"
                    | "cache"
            )
        {
            continue;
        }
        let Some(job_map) = v.as_mapping() else {
            continue;
        };
        let script_count = job_map
            .get(YamlValue::String("script".into()))
            .and_then(|s| s.as_sequence())
            .map(|s| s.len())
            .unwrap_or(0);
        let stage = job_map
            .get(YamlValue::String("stage".into()))
            .and_then(|s| s.as_str())
            .map(String::from)
            .unwrap_or_else(|| "test".to_string());

        jobs.push(Job {
            name: key,
            runs_on: Some(format!("stage:{}", stage)),
            steps_count: script_count,
            steps_summary: Vec::new(),
        });
    }

    let triggers = vec!["push".to_string(), "merge_request".to_string()];

    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    report.providers_detected.push("gitlab-ci".to_string());
    report.workflows.push(Workflow {
        provider: "gitlab-ci".to_string(),
        file: rel,
        name: None,
        triggers,
        jobs,
    });
}

fn detect_circleci(root: &Path, report: &mut CiReport) {
    let path = root.join(".circleci/config.yml");
    if !path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let yaml: YamlValue = match serde_yaml::from_str(&content) {
        Ok(y) => y,
        Err(_) => return,
    };

    let mut jobs = Vec::new();
    if let Some(jobs_map) = yaml.get("jobs").and_then(|v| v.as_mapping()) {
        for (k, v) in jobs_map {
            let name = k.as_str().unwrap_or("?").to_string();
            let executor = v
                .get("docker")
                .and_then(|d| d.as_sequence())
                .and_then(|s| s.first())
                .and_then(|f| f.get("image"))
                .and_then(|i| i.as_str())
                .map(String::from)
                .or_else(|| v.get("machine").and_then(|m| m.as_str()).map(String::from));
            let steps_count = v
                .get("steps")
                .and_then(|s| s.as_sequence())
                .map(|s| s.len())
                .unwrap_or(0);
            jobs.push(Job {
                name,
                runs_on: executor,
                steps_count,
                steps_summary: Vec::new(),
            });
        }
    }

    let triggers = vec!["push".to_string()];
    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    report.providers_detected.push("circleci".to_string());
    report.workflows.push(Workflow {
        provider: "circleci".to_string(),
        file: rel,
        name: None,
        triggers,
        jobs,
    });
}

fn detect_jenkins(root: &Path, report: &mut CiReport) {
    let path = root.join("Jenkinsfile");
    if !path.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut jobs = Vec::new();
    let mut stage_count = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("stage(") {
            let name = rest
                .trim_start_matches(['\'', '"'])
                .split(['\'', '"'])
                .next()
                .unwrap_or("")
                .to_string();
            if !name.is_empty() {
                stage_count += 1;
                jobs.push(Job {
                    name,
                    runs_on: None,
                    steps_count: 0,
                    steps_summary: Vec::new(),
                });
            }
        }
    }

    let triggers = if content.contains("triggers {") {
        vec!["scm-poll".to_string()]
    } else {
        vec!["manual".to_string()]
    };

    let rel = path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned());

    report.providers_detected.push("jenkins".to_string());
    report.workflows.push(Workflow {
        provider: "jenkins".to_string(),
        file: rel,
        name: Some(format!("Jenkinsfile ({} stages)", stage_count)),
        triggers,
        jobs,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_simple_github_workflow() {
        let tmp = tempfile::tempdir().unwrap();
        let wf_dir = tmp.path().join(".github/workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(
            wf_dir.join("ci.yml"),
            r#"
name: CI
on:
  push:
    branches: [main]
  pull_request:
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test --workspace
  lint:
    runs-on: ubuntu-latest
    steps:
      - run: cargo clippy
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(report
            .providers_detected
            .contains(&"github-actions".to_string()));
        assert_eq!(report.workflows.len(), 1);
        let wf = &report.workflows[0];
        assert_eq!(wf.name.as_deref(), Some("CI"));
        assert_eq!(wf.jobs.len(), 2);
        assert!(report.triggers_summary.on_push);
        assert!(report.triggers_summary.on_pull_request);
    }

    #[test]
    fn parses_gitlab_ci() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join(".gitlab-ci.yml"),
            r#"
stages:
  - test
  - deploy
unit-test:
  stage: test
  script:
    - cargo test
deploy-prod:
  stage: deploy
  script:
    - ./deploy.sh
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(report.providers_detected.contains(&"gitlab-ci".to_string()));
        assert_eq!(report.workflows[0].jobs.len(), 2);
    }

    #[test]
    fn detects_no_ci_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.providers_detected.len(), 0);
        assert_eq!(report.workflows.len(), 0);
    }

    #[test]
    fn parses_jenkins_stages() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("Jenkinsfile"),
            r#"
pipeline {
    agent any
    triggers {
        cron('H 0 * * *')
    }
    stages {
        stage('Build') {
            steps { sh 'make' }
        }
        stage('Test') {
            steps { sh 'make test' }
        }
        stage('Deploy') {
            steps { sh 'make deploy' }
        }
    }
}
"#,
        )
        .unwrap();

        let report = detect(tmp.path());
        assert!(report.providers_detected.contains(&"jenkins".to_string()));
        assert_eq!(report.workflows[0].jobs.len(), 3);
    }
}
