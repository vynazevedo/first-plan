//! Release history detection via git tags + optional CHANGELOG parsing.
//!
//! Detecta:
//! - Todas as git tags (semver-ordenadas quando possivel)
//! - Data de criacao de cada tag (commit date)
//! - Commit range entre tags (base..head)
//! - Contagem de commits/autores por release
//! - Cross-refere com CHANGELOG.md quando entries batem com tags

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleasesReport {
    pub total_releases: usize,
    pub releases: Vec<Release>,
    pub latest_tag: Option<String>,
    pub latest_sha: Option<String>,
    pub changelog_matched: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag: String,
    pub sha: String,
    pub date: String,
    pub previous_tag: Option<String>,
    pub commit_count: u32,
    pub author_count: u32,
    pub changelog_entry: Option<String>,
    pub is_semver: bool,
}

pub fn detect(root: &Path) -> ReleasesReport {
    let mut report = ReleasesReport::default();

    if !root.join(".git").exists() {
        return report;
    }

    let raw_tags = match list_tags(root) {
        Some(t) => t,
        None => return report,
    };

    let mut tags: Vec<(String, String, String)> = raw_tags;
    tags.sort_by(|a, b| compare_versions(&a.0, &b.0));

    let changelog_versions = parse_changelog_versions(root);

    let mut previous: Option<String> = None;
    for (tag, sha, date) in tags.iter() {
        let (commit_count, author_count) = count_range(root, previous.as_deref(), sha);
        let changelog_entry = find_changelog_entry(&changelog_versions, tag);
        let is_semver = looks_like_semver(tag);

        report.releases.push(Release {
            tag: tag.clone(),
            sha: sha.clone(),
            date: date.clone(),
            previous_tag: previous.clone(),
            commit_count,
            author_count,
            changelog_entry,
            is_semver,
        });

        previous = Some(tag.clone());
    }

    report.total_releases = report.releases.len();
    report.changelog_matched = report
        .releases
        .iter()
        .filter(|r| r.changelog_entry.is_some())
        .count();

    if let Some(last) = report.releases.last() {
        report.latest_tag = Some(last.tag.clone());
        report.latest_sha = Some(last.sha.clone());
    }

    report
}

fn list_tags(root: &Path) -> Option<Vec<(String, String, String)>> {
    let out = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)|%(objectname)|%(creatordate:short)",
            "--sort=creatordate",
            "refs/tags",
        ])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let mut tags = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 {
            continue;
        }
        tags.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
        ));
    }
    Some(tags)
}

fn count_range(root: &Path, from: Option<&str>, to: &str) -> (u32, u32) {
    let range = match from {
        Some(f) => format!("{}..{}", f, to),
        None => to.to_string(),
    };
    let commits = Command::new("git")
        .args(["rev-list", "--count", &range])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0);

    let authors_out = Command::new("git")
        .args(["log", &range, "--format=%an"])
        .current_dir(root)
        .output();

    let author_count = if let Ok(o) = authors_out {
        let raw = String::from_utf8_lossy(&o.stdout);
        let set: HashSet<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
        set.len() as u32
    } else {
        0
    };

    (commits, author_count)
}

fn parse_changelog_versions(root: &Path) -> Vec<(String, String)> {
    let candidates = ["CHANGELOG.md", "CHANGES.md", "HISTORY.md"];
    for name in candidates {
        let path = root.join(name);
        if !path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        return extract_changelog_versions(&content);
    }
    Vec::new()
}

fn extract_changelog_versions(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("## [") {
            if let Some(end) = rest.find(']') {
                let version = rest[..end].to_string();
                let after = &rest[end + 1..];
                let date = after
                    .trim_start_matches([' ', '-', '('])
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(')')
                    .to_string();
                out.push((version, date));
            }
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            if let Some(ver) = rest.split_whitespace().next() {
                out.push((ver.to_string(), String::new()));
            }
        }
    }
    out
}

fn find_changelog_entry(entries: &[(String, String)], tag: &str) -> Option<String> {
    let normalized = tag.trim_start_matches('v');
    for (version, date) in entries {
        let entry_ver = version.trim_start_matches('v');
        if entry_ver == normalized {
            return Some(if date.is_empty() {
                version.clone()
            } else {
                format!("[{}] - {}", version, date)
            });
        }
    }
    None
}

fn looks_like_semver(tag: &str) -> bool {
    let stripped = tag.trim_start_matches('v');
    let parts: Vec<&str> = stripped.split('.').collect();
    if parts.len() < 2 || parts.len() > 4 {
        return false;
    }
    parts.iter().all(|p| {
        let base = p.split(&['-', '+'][..]).next().unwrap_or("");
        !base.is_empty() && base.chars().all(|c| c.is_ascii_digit())
    })
}

fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_norm = a.trim_start_matches('v');
    let b_norm = b.trim_start_matches('v');
    let a_parts: Vec<u32> = a_norm
        .split(&['.', '-'][..])
        .filter_map(|p| p.parse().ok())
        .collect();
    let b_parts: Vec<u32> = b_norm
        .split(&['.', '-'][..])
        .filter_map(|p| p.parse().ok())
        .collect();
    if !a_parts.is_empty() && !b_parts.is_empty() {
        return a_parts.cmp(&b_parts);
    }
    a_norm.cmp(b_norm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_semver_tags() {
        assert!(looks_like_semver("v1.0.0"));
        assert!(looks_like_semver("1.0.0"));
        assert!(looks_like_semver("v0.8.1"));
        assert!(looks_like_semver("v2.0.0-rc.1"));
        assert!(!looks_like_semver("release-2024"));
        assert!(!looks_like_semver("hotfix"));
    }

    #[test]
    fn compares_semver_correctly() {
        assert!(compare_versions("v0.1.0", "v0.2.0").is_lt());
        assert!(compare_versions("v0.9.0", "v0.10.0").is_lt());
        assert!(compare_versions("v1.0.0", "v0.99.0").is_gt());
        assert!(compare_versions("v0.8.1", "v0.8.1").is_eq());
    }

    #[test]
    fn parses_changelog_versions_kac_format() {
        let content = r#"# Changelog

## [1.0.0] - 2026-01-01

Added stuff

## [0.9.0] - 2025-12-15

Beta
"#;
        let entries = extract_changelog_versions(content);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "1.0.0");
        assert_eq!(entries[0].1, "2026-01-01");
    }

    #[test]
    fn matches_tag_with_v_prefix() {
        let entries = vec![("1.0.0".to_string(), "2026-01-01".to_string())];
        let matched = find_changelog_entry(&entries, "v1.0.0");
        assert!(matched.is_some());
    }

    #[test]
    fn empty_when_no_git() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.total_releases, 0);
    }
}
