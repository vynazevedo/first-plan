//! Map each source file to release(s) where it was introduced and last modified.
//!
//! Serve AI para saber:
//! - Codigo introduzido em v0.5.0 eh production-stable ha muito tempo
//! - Codigo modificado apenas em unreleased eh 'edge' code
//! - Ordenacao por 'idade' ajuda priorizar reviews e testes
//!
//! Approach: para cada arquivo source, git log com --follow busca primeiro
//! e ultimo commit. Cruza com release tags para mapear em qual release cada
//! commit foi included.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

use super::releases::ReleasesReport;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileReleasesReport {
    pub total_files_analyzed: usize,
    pub files: Vec<FileRelease>,
    pub introduced_by_release: Vec<ReleaseFileCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRelease {
    pub path: String,
    pub introduced_in: Option<String>,
    pub last_modified_in: Option<String>,
    pub is_unreleased: bool,
    pub commit_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseFileCount {
    pub release: String,
    pub file_count: u32,
}

const EXCLUDED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    ".git",
    "dist",
    "build",
    ".first-plan",
];

const SOURCE_EXTS: &[&str] = &[
    "rs", "go", "py", "ts", "tsx", "js", "jsx", "java", "kt", "rb", "php", "cs",
];

const MAX_FILES: usize = 500;

pub fn detect(root: &Path, releases: &ReleasesReport) -> FileReleasesReport {
    let mut report = FileReleasesReport::default();

    if !root.join(".git").exists() || releases.releases.is_empty() {
        return report;
    }

    let files = collect_source_files(root);
    report.total_files_analyzed = files.len();

    let sha_to_release = build_sha_release_map(root, releases);

    let file_releases: Vec<FileRelease> = files
        .par_iter()
        .filter_map(|path| {
            let rel_path = path.strip_prefix(root).ok()?;
            let rel_str = rel_path.to_string_lossy().into_owned();
            let (first_sha, last_sha, count) = git_log_sha_range(root, rel_path);
            first_sha.as_ref()?;
            let introduced_in = first_sha
                .as_ref()
                .and_then(|s| sha_to_release.get(s).cloned());
            let last_modified_in = last_sha
                .as_ref()
                .and_then(|s| sha_to_release.get(s).cloned());
            let is_unreleased = last_modified_in.is_none();
            Some(FileRelease {
                path: rel_str,
                introduced_in,
                last_modified_in,
                is_unreleased,
                commit_count: count,
            })
        })
        .collect();

    let mut intro_counts: HashMap<String, u32> = HashMap::new();
    for fr in &file_releases {
        if let Some(rel_name) = &fr.introduced_in {
            *intro_counts.entry(rel_name.clone()).or_insert(0) += 1;
        }
    }

    let mut file_releases = file_releases;
    file_releases.sort_by(|a, b| a.path.cmp(&b.path));
    report.files = file_releases;

    let mut intro_by_release: Vec<ReleaseFileCount> = intro_counts
        .into_iter()
        .map(|(release, file_count)| ReleaseFileCount {
            release,
            file_count,
        })
        .collect();
    intro_by_release.sort_by_key(|r| std::cmp::Reverse(r.file_count));
    report.introduced_by_release = intro_by_release;

    report
}

fn collect_source_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .max_depth(10)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !EXCLUDED_DIRS.iter().any(|d| name == *d)
        })
        .filter_map(|e| e.ok())
    {
        if out.len() >= MAX_FILES {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if SOURCE_EXTS.contains(&ext) {
            out.push(entry.path().to_path_buf());
        }
    }
    out
}

fn git_log_sha_range(root: &Path, rel_path: &Path) -> (Option<String>, Option<String>, u32) {
    let out = Command::new("git")
        .args([
            "log",
            "--follow",
            "--format=%H",
            "--",
            rel_path.to_str().unwrap_or(""),
        ])
        .current_dir(root)
        .output();

    let Ok(out) = out else {
        return (None, None, 0);
    };
    if !out.status.success() {
        return (None, None, 0);
    }

    let raw = String::from_utf8_lossy(&out.stdout);
    let shas: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    if shas.is_empty() {
        return (None, None, 0);
    }

    let last_sha = shas.first().map(|s| s.to_string());
    let first_sha = shas.last().map(|s| s.to_string());
    (first_sha, last_sha, shas.len() as u32)
}

fn build_sha_release_map(root: &Path, releases: &ReleasesReport) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut previous: Option<&str> = None;
    for release in &releases.releases {
        let range = match previous {
            Some(p) => format!("{}..{}", p, release.tag),
            None => release.tag.clone(),
        };
        let shas = git_rev_list(root, &range);
        for sha in shas {
            map.entry(sha).or_insert_with(|| release.tag.clone());
        }
        previous = Some(&release.tag);
    }
    map
}

fn git_rev_list(root: &Path, range: &str) -> Vec<String> {
    Command::new("git")
        .args(["rev-list", range])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_when_no_git() {
        let tmp = tempfile::tempdir().unwrap();
        let releases = ReleasesReport::default();
        let report = detect(tmp.path(), &releases);
        assert_eq!(report.total_files_analyzed, 0);
    }

    #[test]
    fn empty_when_no_releases() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();
        let releases = ReleasesReport::default();
        let report = detect(tmp.path(), &releases);
        assert_eq!(report.files.len(), 0);
    }
}
