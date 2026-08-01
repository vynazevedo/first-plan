//! Migration detection via git history mining.
//>
//! Heuristicas:
//! 1. Conventional commits com marker breaking: feat!, refactor!, fix!, BREAKING CHANGE
//! 2. Commits com keywords: migrate, replace, refactor, rewrite
//! 3. Files deleted + files added no mesmo commit (candidate para replacement pair)
//! 4. Nome similar A -> B em commits (ex: `old_auth.rs` deletado, `new_auth.rs` criado)
//!
//! Output: lista ordenada por recencia com actionable info para AI.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MigrationsReport {
    pub analyzed_commits: u32,
    pub window_days: u32,
    pub breaking_commits: Vec<BreakingCommit>,
    pub replacement_candidates: Vec<ReplacementPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingCommit {
    pub sha: String,
    pub subject: String,
    pub date: String,
    pub author: String,
    pub kind: BreakingKind,
    pub files_affected: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BreakingKind {
    ConventionalBreaking,
    BreakingChangeFooter,
    RefactorKeyword,
    MigrateKeyword,
    RewriteKeyword,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementPair {
    pub sha: String,
    pub subject: String,
    pub date: String,
    pub removed: String,
    pub added: String,
    pub confidence: f32,
}

const MIGRATION_KEYWORDS: &[(&str, BreakingKind)] = &[
    ("migrate ", BreakingKind::MigrateKeyword),
    ("migration ", BreakingKind::MigrateKeyword),
    ("replace ", BreakingKind::RefactorKeyword),
    ("rewrite ", BreakingKind::RewriteKeyword),
    ("refactor: ", BreakingKind::RefactorKeyword),
];

pub fn detect(root: &Path) -> MigrationsReport {
    detect_with_window(root, 365)
}

pub fn detect_with_window(root: &Path, window_days: u32) -> MigrationsReport {
    let mut report = MigrationsReport {
        analyzed_commits: 0,
        window_days,
        ..Default::default()
    };

    if !root.join(".git").exists() {
        return report;
    }

    let commits = fetch_commits(root, window_days);
    report.analyzed_commits = commits.len() as u32;

    for commit in &commits {
        if let Some(kind) = detect_breaking_kind(&commit.subject, &commit.body) {
            report.breaking_commits.push(BreakingCommit {
                sha: commit.sha.clone(),
                subject: commit.subject.clone(),
                date: commit.date.clone(),
                author: commit.author.clone(),
                kind,
                files_affected: commit.changed_files.len() + commit.deleted_files.len(),
            });
        }

        for pair in infer_replacements(commit) {
            report.replacement_candidates.push(pair);
        }
    }

    report.replacement_candidates.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.replacement_candidates.truncate(50);

    report.breaking_commits.truncate(50);

    report
}

#[derive(Debug)]
struct Commit {
    sha: String,
    subject: String,
    body: String,
    date: String,
    author: String,
    changed_files: Vec<String>,
    deleted_files: Vec<String>,
}

fn fetch_commits(root: &Path, window_days: u32) -> Vec<Commit> {
    let output = Command::new("git")
        .args([
            "log",
            &format!("--since={} days ago", window_days),
            "--name-status",
            "--pretty=format:###COMMIT###%H%n%s%n%an%n%aI%n%b%n###ENDBODY###",
        ])
        .current_dir(root)
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    parse_git_output(&raw)
}

fn parse_git_output(raw: &str) -> Vec<Commit> {
    let mut commits = Vec::new();
    let mut current: Option<Commit> = None;
    let mut in_body = false;
    let mut body_buf = String::new();

    for line in raw.lines() {
        if let Some(sha) = line.strip_prefix("###COMMIT###") {
            if let Some(mut c) = current.take() {
                c.body = std::mem::take(&mut body_buf);
                commits.push(c);
            }
            current = Some(Commit {
                sha: sha.to_string(),
                subject: String::new(),
                body: String::new(),
                date: String::new(),
                author: String::new(),
                changed_files: Vec::new(),
                deleted_files: Vec::new(),
            });
            in_body = false;
            continue;
        }

        let Some(c) = current.as_mut() else {
            continue;
        };

        if c.subject.is_empty() {
            c.subject = line.to_string();
        } else if c.author.is_empty() {
            c.author = line.to_string();
        } else if c.date.is_empty() {
            c.date = line.to_string();
            in_body = true;
        } else if line == "###ENDBODY###" {
            in_body = false;
        } else if in_body {
            if !body_buf.is_empty() {
                body_buf.push('\n');
            }
            body_buf.push_str(line);
        } else {
            let mut parts = line.splitn(2, '\t');
            let (Some(status), Some(path)) = (parts.next(), parts.next()) else {
                continue;
            };
            match status {
                "D" => c.deleted_files.push(path.to_string()),
                "A" | "M" | "R" | "C" => c.changed_files.push(path.to_string()),
                s if s.starts_with('R') => {
                    let mut r_parts = path.split('\t');
                    if let (Some(from), Some(to)) = (r_parts.next(), r_parts.next()) {
                        c.deleted_files.push(from.to_string());
                        c.changed_files.push(to.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(mut c) = current {
        c.body = body_buf;
        commits.push(c);
    }

    commits
}

fn detect_breaking_kind(subject: &str, body: &str) -> Option<BreakingKind> {
    let subj_lower = subject.to_lowercase();

    if subject.contains("BREAKING CHANGE") || body.contains("BREAKING CHANGE") {
        return Some(BreakingKind::BreakingChangeFooter);
    }

    let conventional_types = [
        "feat!",
        "fix!",
        "refactor!",
        "perf!",
        "chore!",
        "revert!",
        "build!",
    ];
    for t in conventional_types {
        if subject.starts_with(t) || subject.contains(&format!(" {}", t)) {
            return Some(BreakingKind::ConventionalBreaking);
        }
    }

    for (kw, kind) in MIGRATION_KEYWORDS {
        if subj_lower.contains(kw) {
            return Some(*kind);
        }
    }

    None
}

fn infer_replacements(commit: &Commit) -> Vec<ReplacementPair> {
    let mut out = Vec::new();

    for removed in &commit.deleted_files {
        for added in &commit.changed_files {
            let confidence = similarity_score(removed, added);
            if confidence >= 0.5 {
                out.push(ReplacementPair {
                    sha: commit.sha.clone(),
                    subject: commit.subject.clone(),
                    date: commit.date.clone(),
                    removed: removed.clone(),
                    added: added.clone(),
                    confidence,
                });
            }
        }
    }

    out
}

fn similarity_score(a: &str, b: &str) -> f32 {
    let a_stem = std::path::Path::new(a)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let b_stem = std::path::Path::new(b)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if a_stem.is_empty() || b_stem.is_empty() {
        return 0.0;
    }

    let a_lower = a_stem.to_lowercase();
    let b_lower = b_stem.to_lowercase();

    if a_lower == b_lower {
        return 0.9;
    }

    let a_parts: std::collections::HashSet<&str> = a_lower.split(['_', '-']).collect();
    let b_parts: std::collections::HashSet<&str> = b_lower.split(['_', '-']).collect();

    let intersection = a_parts.intersection(&b_parts).count();
    let union = a_parts.union(&b_parts).count();

    if union == 0 {
        return 0.0;
    }

    let jaccard = intersection as f32 / union as f32;

    let path_a = std::path::Path::new(a);
    let path_b = std::path::Path::new(b);
    let same_dir = path_a.parent() == path_b.parent();

    if same_dir {
        (jaccard + 0.2).min(1.0)
    } else {
        jaccard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_breaking_change_footer() {
        assert!(matches!(
            detect_breaking_kind("refactor: rename user API", "BREAKING CHANGE: renamed"),
            Some(BreakingKind::BreakingChangeFooter)
        ));
    }

    #[test]
    fn detects_conventional_breaking() {
        assert!(matches!(
            detect_breaking_kind("feat!: new API", ""),
            Some(BreakingKind::ConventionalBreaking)
        ));
    }

    #[test]
    fn detects_migrate_keyword() {
        assert!(matches!(
            detect_breaking_kind("migrate to axum from actix", ""),
            Some(BreakingKind::MigrateKeyword)
        ));
    }

    #[test]
    fn similarity_high_when_same_stem_different_dir() {
        assert!(similarity_score("src/auth.rs", "test/auth.rs") >= 0.9);
    }

    #[test]
    fn similarity_higher_when_same_dir() {
        let s = similarity_score("src/old_auth.rs", "src/new_auth.rs");
        assert!(s > 0.3);
    }

    #[test]
    fn no_similarity_between_unrelated() {
        assert!(similarity_score("src/foo.rs", "docs/bar.md") < 0.3);
    }

    #[test]
    fn report_empty_when_no_git() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.analyzed_commits, 0);
    }
}
