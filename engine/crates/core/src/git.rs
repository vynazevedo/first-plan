//! Lightweight wrapper around `git log` to extract commits and changed files.
//!
//! We shell out to git instead of using libgit2/gix because:
//! - zero compile-time dependencies on C code
//! - git log is already extremely fast for our window sizes
//! - users already have git installed (it is the project we are analyzing)

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// One commit in the analysis window with the set of files it modified.
#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub files: Vec<String>,
}

/// Parse `git log --since=<days>` output and return commits with affected files.
///
/// Uses `--name-only --no-merges --pretty=format:"COMMIT:%H"` so each commit is
/// followed by its file list, separated by blank lines.
pub fn parse_log(repo: &Path, since_days: u32) -> Result<Vec<Commit>> {
    if !repo.join(".git").exists() {
        anyhow::bail!("not a git repository: {}", repo.display());
    }

    let since_arg = format!("--since={} days ago", since_days);

    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "log",
            "--name-only",
            "--no-merges",
            "--pretty=format:COMMIT:%H",
            &since_arg,
        ])
        .output()
        .context("failed to spawn git log - is git installed?")?;

    if !output.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_log_output(&stdout))
}

fn parse_log_output(output: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    let mut current: Option<Commit> = None;

    for line in output.lines() {
        if let Some(sha) = line.strip_prefix("COMMIT:") {
            if let Some(commit) = current.take() {
                commits.push(commit);
            }
            current = Some(Commit {
                sha: sha.to_string(),
                files: Vec::new(),
            });
        } else if !line.is_empty() {
            if let Some(commit) = current.as_mut() {
                commit.files.push(line.to_string());
            }
        }
    }

    if let Some(commit) = current.take() {
        commits.push(commit);
    }

    commits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_git_log_output() {
        let raw = "COMMIT:abc123\nfile_a.go\nfile_b.go\n\nCOMMIT:def456\nfile_a.go\n";
        let commits = parse_log_output(raw);

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "abc123");
        assert_eq!(commits[0].files, vec!["file_a.go", "file_b.go"]);
        assert_eq!(commits[1].sha, "def456");
        assert_eq!(commits[1].files, vec!["file_a.go"]);
    }

    #[test]
    fn handles_empty_output() {
        let commits = parse_log_output("");
        assert!(commits.is_empty());
    }

    #[test]
    fn handles_commit_with_no_files() {
        let raw = "COMMIT:abc123\n\nCOMMIT:def456\nfile_a.go\n";
        let commits = parse_log_output(raw);
        assert_eq!(commits.len(), 2);
        assert!(commits[0].files.is_empty());
        assert_eq!(commits[1].files, vec!["file_a.go"]);
    }
}
