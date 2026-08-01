//! Detect commits between latest release tag and HEAD.
//!
//! Critical para AI saber:
//! - Esse bug fix ja foi released ou ainda esta so em main?
//! - Quantos commits acumulados sem release?
//! - Quais autores contribuiram desde ultima release?
//! - Quais arquivos foram tocados?

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use super::releases::ReleasesReport;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnreleasedReport {
    pub since_tag: Option<String>,
    pub since_sha: Option<String>,
    pub commits_count: u32,
    pub authors: Vec<AuthorContribution>,
    pub commits: Vec<UnreleasedCommit>,
    pub files_touched: Vec<FileChange>,
    pub has_breaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreleasedCommit {
    pub sha: String,
    pub short_sha: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub is_breaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorContribution {
    pub name: String,
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub touches: u32,
}

pub fn detect(root: &Path, releases: &ReleasesReport) -> UnreleasedReport {
    let mut report = UnreleasedReport::default();

    if !root.join(".git").exists() {
        return report;
    }

    let latest_tag = releases.latest_tag.clone();
    report.since_tag = latest_tag.clone();
    report.since_sha = releases.latest_sha.clone();

    let range = match &latest_tag {
        Some(tag) => format!("{}..HEAD", tag),
        None => "HEAD".to_string(),
    };

    let commits = fetch_commits(root, &range);
    report.commits_count = commits.len() as u32;

    let mut author_map: HashMap<String, u32> = HashMap::new();
    let mut file_map: HashMap<String, u32> = HashMap::new();

    for c in &commits {
        *author_map.entry(c.author.clone()).or_insert(0) += 1;
        if c.is_breaking {
            report.has_breaking = true;
        }
        for f in &c.files {
            *file_map.entry(f.clone()).or_insert(0) += 1;
        }
    }

    let mut authors: Vec<AuthorContribution> = author_map
        .into_iter()
        .map(|(name, commit_count)| AuthorContribution { name, commit_count })
        .collect();
    authors.sort_by_key(|a| std::cmp::Reverse(a.commit_count));
    report.authors = authors;

    let mut files: Vec<FileChange> = file_map
        .into_iter()
        .map(|(path, touches)| FileChange { path, touches })
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse(f.touches));
    files.truncate(50);
    report.files_touched = files;

    let display_commits: Vec<UnreleasedCommit> = commits
        .into_iter()
        .take(50)
        .map(|c| UnreleasedCommit {
            short_sha: c.sha[..7.min(c.sha.len())].to_string(),
            sha: c.sha,
            author: c.author,
            date: c.date,
            subject: c.subject,
            is_breaking: c.is_breaking,
        })
        .collect();
    report.commits = display_commits;

    report
}

#[derive(Debug)]
struct RawCommit {
    sha: String,
    author: String,
    date: String,
    subject: String,
    files: Vec<String>,
    is_breaking: bool,
}

fn fetch_commits(root: &Path, range: &str) -> Vec<RawCommit> {
    let out = Command::new("git")
        .args([
            "log",
            range,
            "--name-only",
            "--pretty=format:###COMMIT###%H%n%an%n%aI%n%s%n###FILES###",
        ])
        .current_dir(root)
        .output();

    let Ok(out) = out else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let raw = String::from_utf8_lossy(&out.stdout);
    parse_commits(&raw)
}

fn parse_commits(raw: &str) -> Vec<RawCommit> {
    let mut commits = Vec::new();
    let mut current: Option<RawCommit> = None;
    let mut in_files = false;
    let mut seen_files: HashSet<String> = HashSet::new();

    for line in raw.lines() {
        if let Some(sha) = line.strip_prefix("###COMMIT###") {
            if let Some(c) = current.take() {
                commits.push(c);
            }
            current = Some(RawCommit {
                sha: sha.to_string(),
                author: String::new(),
                date: String::new(),
                subject: String::new(),
                files: Vec::new(),
                is_breaking: false,
            });
            in_files = false;
            seen_files.clear();
            continue;
        }

        let Some(c) = current.as_mut() else {
            continue;
        };

        if line == "###FILES###" {
            in_files = true;
            continue;
        }

        if c.author.is_empty() {
            c.author = line.to_string();
        } else if c.date.is_empty() {
            c.date = line.to_string();
        } else if c.subject.is_empty() {
            c.subject = line.to_string();
            let s = &c.subject;
            c.is_breaking = s.contains("BREAKING CHANGE")
                || s.starts_with("feat!")
                || s.starts_with("fix!")
                || s.starts_with("refactor!")
                || s.starts_with("perf!")
                || s.contains(" feat!")
                || s.contains(" fix!");
        } else if in_files && !line.trim().is_empty() {
            let f = line.trim().to_string();
            if !seen_files.contains(&f) {
                seen_files.insert(f.clone());
                c.files.push(f);
            }
        }
    }

    if let Some(c) = current {
        commits.push(c);
    }

    commits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_no_git() {
        let tmp = tempfile::tempdir().unwrap();
        let releases = ReleasesReport::default();
        let report = detect(tmp.path(), &releases);
        assert_eq!(report.commits_count, 0);
    }

    #[test]
    fn detects_breaking_marker_in_subject() {
        let raw = "###COMMIT###abc123\ndev\n2026-01-01\nfeat!: new API\n###FILES###\nsrc/a.rs\n";
        let parsed = parse_commits(raw);
        assert_eq!(parsed.len(), 1);
        assert!(parsed[0].is_breaking);
    }

    #[test]
    fn detects_breaking_in_body_via_subject() {
        let raw = "###COMMIT###abc\ndev\n2026-01-01\nrefactor: BREAKING CHANGE api\n###FILES###\n";
        let parsed = parse_commits(raw);
        assert!(parsed[0].is_breaking);
    }

    #[test]
    fn dedupes_files_per_commit() {
        let raw = "###COMMIT###a\ndev\n2026-01-01\nsubj\n###FILES###\na.rs\na.rs\nb.rs\n";
        let parsed = parse_commits(raw);
        assert_eq!(parsed[0].files.len(), 2);
    }
}
