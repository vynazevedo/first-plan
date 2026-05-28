//! Flaky test detection via git history mining.
//!
//! Heuristicas (sem precisar de CI logs externos):
//!
//! 1. Test file editado isoladamente (sem mudanca em codigo nao-test no mesmo commit)
//!    com mensagem suspeita: "fix flaky", "stabilize test", "retry", "race condition",
//!    "timeout", "intermittent", "unstable test".
//! 2. Test file revertido apos commit ("revert", "revert pr").
//! 3. Test file com alta frequencia de mudanca isolada nos ultimos 180 dias.
//!
//! Score combina os 3 sinais. Top N como flaky candidates.
//!
//! Note: heuristica, nao verdade absoluta. Output indica confidence baixa quando
//! sinal eh fraco. Ideal complementar com CI logs no futuro.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlakyReport {
    pub analyzed_commits: u32,
    pub window_days: u32,
    pub candidates: Vec<FlakyCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyCandidate {
    pub path: String,
    pub score: f32,
    pub signals: Vec<String>,
    pub isolated_edits_count: u32,
    pub suspicious_msg_count: u32,
    pub reverts_count: u32,
}

const FLAKY_KEYWORDS: &[&str] = &[
    "flaky",
    "stabilize",
    "stabili", // covers "destabilize" too
    "race",
    "timeout",
    "intermittent",
    "unstable",
    "skip test",
    "disable test",
    "retry",
    "rerun",
    "flake",
];

const REVERT_KEYWORDS: &[&str] = &["revert ", "reverts "];

pub fn detect(root: &Path) -> FlakyReport {
    detect_with_window(root, 180)
}

pub fn detect_with_window(root: &Path, window_days: u32) -> FlakyReport {
    let mut report = FlakyReport {
        analyzed_commits: 0,
        window_days,
        candidates: Vec::new(),
    };

    if !root.join(".git").exists() {
        return report;
    }

    let log_output = Command::new("git")
        .args([
            "log",
            &format!("--since={} days ago", window_days),
            "--name-only",
            "--pretty=format:###COMMIT###%H%n%s",
        ])
        .current_dir(root)
        .output();

    let log_output = match log_output {
        Ok(o) if o.status.success() => o,
        _ => return report,
    };

    let raw = String::from_utf8_lossy(&log_output.stdout);

    let mut commits: Vec<Commit> = Vec::new();
    let mut current: Option<Commit> = None;
    let mut expect_subject = false;

    for line in raw.lines() {
        if let Some(sha) = line.strip_prefix("###COMMIT###") {
            if let Some(c) = current.take() {
                commits.push(c);
            }
            current = Some(Commit {
                sha: sha.to_string(),
                subject: String::new(),
                files: Vec::new(),
            });
            expect_subject = true;
        } else if expect_subject {
            if let Some(ref mut c) = current {
                c.subject = line.to_string();
            }
            expect_subject = false;
        } else if !line.trim().is_empty() {
            if let Some(ref mut c) = current {
                c.files.push(line.to_string());
            }
        }
    }
    if let Some(c) = current {
        commits.push(c);
    }

    report.analyzed_commits = commits.len() as u32;

    let mut per_file: HashMap<String, FlakyStats> = HashMap::new();

    for commit in &commits {
        let test_files: Vec<&String> = commit.files.iter().filter(|f| is_test_file(f)).collect();
        let non_test_files: Vec<&String> =
            commit.files.iter().filter(|f| !is_test_file(f)).collect();

        if test_files.is_empty() {
            continue;
        }

        let isolated = non_test_files.is_empty();
        let subject_lower = commit.subject.to_lowercase();
        let has_flaky_keyword = FLAKY_KEYWORDS.iter().any(|k| subject_lower.contains(k));
        let has_revert_keyword = REVERT_KEYWORDS.iter().any(|k| subject_lower.contains(k));

        for f in test_files {
            let stats = per_file.entry(f.clone()).or_default();
            stats.total_touches += 1;
            if isolated {
                stats.isolated_edits += 1;
            }
            if has_flaky_keyword {
                stats.suspicious_msgs += 1;
            }
            if has_revert_keyword {
                stats.reverts += 1;
            }
        }
    }

    let mut candidates: Vec<FlakyCandidate> = per_file
        .into_iter()
        .filter_map(|(path, stats)| {
            let score = compute_score(&stats);
            if score < 0.2 {
                return None;
            }
            let mut signals = Vec::new();
            if stats.suspicious_msgs > 0 {
                signals.push(format!(
                    "{} commit(s) com keyword suspeita (flaky/race/timeout/etc)",
                    stats.suspicious_msgs
                ));
            }
            if stats.reverts > 0 {
                signals.push(format!("{} revert(s) tocando este test", stats.reverts));
            }
            if stats.isolated_edits >= 3 {
                signals.push(format!(
                    "{} edits isoladas (sem mudanca em codigo correspondente)",
                    stats.isolated_edits
                ));
            }
            Some(FlakyCandidate {
                path,
                score,
                signals,
                isolated_edits_count: stats.isolated_edits,
                suspicious_msg_count: stats.suspicious_msgs,
                reverts_count: stats.reverts,
            })
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(50);
    report.candidates = candidates;

    report
}

#[derive(Default)]
struct FlakyStats {
    total_touches: u32,
    isolated_edits: u32,
    suspicious_msgs: u32,
    reverts: u32,
}

#[derive(Debug)]
struct Commit {
    #[allow(dead_code)]
    sha: String,
    subject: String,
    files: Vec<String>,
}

fn compute_score(stats: &FlakyStats) -> f32 {
    let kw_weight = stats.suspicious_msgs as f32 * 0.5;
    let revert_weight = stats.reverts as f32 * 0.7;
    let isolation_weight = if stats.isolated_edits >= 3 {
        ((stats.isolated_edits - 2) as f32 * 0.15).min(0.6)
    } else {
        0.0
    };
    (kw_weight + revert_weight + isolation_weight).min(2.0)
}

fn is_test_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("/__tests__/")
        || lower.contains("/spec/")
        || lower.starts_with("test/")
        || lower.starts_with("tests/")
        || lower.starts_with("spec/")
    {
        return true;
    }
    let filename = lower.rsplit('/').next().unwrap_or(&lower);
    if filename.ends_with("_test.go")
        || filename.ends_with("_test.py")
        || filename.ends_with(".test.ts")
        || filename.ends_with(".test.tsx")
        || filename.ends_with(".test.js")
        || filename.ends_with(".test.jsx")
        || filename.ends_with(".spec.ts")
        || filename.ends_with(".spec.tsx")
        || filename.ends_with(".spec.js")
        || filename.ends_with(".spec.jsx")
        || filename.ends_with("_spec.rb")
        || filename.ends_with("_test.rs")
        || filename.starts_with("test_")
    {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_test_files() {
        assert!(is_test_file("internal/auth/jwt_test.go"));
        assert!(is_test_file("src/utils/foo.test.ts"));
        assert!(is_test_file("src/utils/bar.spec.tsx"));
        assert!(is_test_file("tests/test_user.py"));
        assert!(is_test_file("src/__tests__/component.test.jsx"));
        assert!(is_test_file("spec/models/user_spec.rb"));
        assert!(is_test_file("test/integration.rs"));

        assert!(!is_test_file("src/main.go"));
        assert!(!is_test_file("README.md"));
        assert!(!is_test_file("internal/auth/jwt.go"));
    }

    #[test]
    fn computes_score_zero_when_no_signals() {
        let stats = FlakyStats {
            total_touches: 1,
            isolated_edits: 0,
            suspicious_msgs: 0,
            reverts: 0,
        };
        assert_eq!(compute_score(&stats), 0.0);
    }

    #[test]
    fn computes_score_with_keyword_signal() {
        let stats = FlakyStats {
            total_touches: 2,
            isolated_edits: 1,
            suspicious_msgs: 2,
            reverts: 0,
        };
        let score = compute_score(&stats);
        assert!(score >= 1.0, "expected >= 1.0, got {}", score);
    }

    #[test]
    fn computes_score_caps_at_2() {
        let stats = FlakyStats {
            total_touches: 100,
            isolated_edits: 50,
            suspicious_msgs: 20,
            reverts: 20,
        };
        let score = compute_score(&stats);
        assert!(score <= 2.0);
    }

    #[test]
    fn report_empty_when_no_git() {
        let tmp = tempfile::tempdir().unwrap();
        let report = detect(tmp.path());
        assert_eq!(report.analyzed_commits, 0);
        assert_eq!(report.candidates.len(), 0);
    }
}
