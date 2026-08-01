//! Coleta de sinais do projeto para alimentar prompts de geração de layer.
//!
//! Objetivo: capturar em texto compacto o suficiente informação para que um
//! LLM consiga produzir cada layer do `.first-plan/` sem precisar navegar
//! por dezenas de arquivos. Prioriza sinais estáveis e universais (manifests
//! de package manager, README, tree, git log recente).

use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct ProjectSignals {
    pub root: PathBuf,
    pub readme: Option<String>,
    pub manifests: Vec<Manifest>,
    pub tree: String,
    pub git_activity: Option<GitActivity>,
    pub detected_stacks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Manifest {
    pub path: String,
    pub excerpt: String,
}

#[derive(Debug, Serialize)]
pub struct GitActivity {
    pub total_commits_90d: usize,
    pub recent_commits: Vec<String>,
    pub top_files: Vec<(String, usize)>,
}

const MAX_README_BYTES: usize = 8_000;
const MAX_MANIFEST_BYTES: usize = 4_000;
const MAX_TREE_DEPTH: usize = 3;
const MAX_TREE_ENTRIES: usize = 200;
const MAX_RECENT_COMMITS: usize = 30;
const MAX_TOP_FILES: usize = 15;

const MANIFEST_FILES: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "Gemfile",
    "composer.json",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "mix.exs",
];

const STACK_INDICATORS: &[(&str, &str)] = &[
    ("Cargo.toml", "rust"),
    ("package.json", "javascript/typescript"),
    ("go.mod", "go"),
    ("pyproject.toml", "python"),
    ("requirements.txt", "python"),
    ("Gemfile", "ruby"),
    ("composer.json", "php"),
    ("pom.xml", "java"),
    ("build.gradle", "java/kotlin"),
    ("build.gradle.kts", "kotlin"),
    ("mix.exs", "elixir"),
    ("Dockerfile", "docker"),
    ("docker-compose.yml", "docker-compose"),
    ("terraform.tf", "terraform"),
    ("main.tf", "terraform"),
];

pub fn collect(root: &Path) -> Result<ProjectSignals> {
    let readme = find_readme(root);
    let manifests = collect_manifests(root)?;
    let tree = build_tree(root)?;
    let git_activity = collect_git_activity(root).ok();
    let detected_stacks = detect_stacks(root);

    Ok(ProjectSignals {
        root: root.to_path_buf(),
        readme,
        manifests,
        tree,
        git_activity,
        detected_stacks,
    })
}

fn find_readme(root: &Path) -> Option<String> {
    for name in &["README.md", "README.rst", "README.txt", "README"] {
        let path = root.join(name);
        if let Ok(content) = fs::read_to_string(&path) {
            return Some(truncate(&content, MAX_README_BYTES));
        }
    }
    None
}

fn collect_manifests(root: &Path) -> Result<Vec<Manifest>> {
    let mut out = Vec::new();
    for name in MANIFEST_FILES {
        let path = root.join(name);
        if let Ok(content) = fs::read_to_string(&path) {
            out.push(Manifest {
                path: name.to_string(),
                excerpt: truncate(&content, MAX_MANIFEST_BYTES),
            });
        }
    }
    Ok(out)
}

fn build_tree(root: &Path) -> Result<String> {
    let mut lines = Vec::new();
    let root_str = root.to_string_lossy().to_string();

    for entry in WalkDir::new(root)
        .max_depth(MAX_TREE_DEPTH)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
        .flatten()
    {
        let path = entry.path();
        if path == root {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string());
        let depth = entry.depth();
        let indent = "  ".repeat(depth.saturating_sub(1));
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let marker = if path.is_dir() { "/" } else { "" };
        lines.push(format!("{}{}{}", indent, name, marker));
        let _ = rel;
        let _ = root_str;
        if lines.len() >= MAX_TREE_ENTRIES {
            lines.push("... (truncated)".to_string());
            break;
        }
    }

    Ok(lines.join("\n"))
}

fn is_excluded(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    matches!(
        name.as_str(),
        "node_modules"
            | "target"
            | ".git"
            | "dist"
            | "build"
            | ".venv"
            | "venv"
            | "__pycache__"
            | ".next"
            | ".nuxt"
            | ".first-plan"
            | "vendor"
            | ".cache"
    )
}

fn collect_git_activity(root: &Path) -> Result<GitActivity> {
    let is_git = root.join(".git").exists();
    if !is_git {
        anyhow::bail!("not a git repository");
    }

    let recent = Command::new("git")
        .args([
            "log",
            "--since=90 days ago",
            "--pretty=format:%h %s",
            "--no-merges",
        ])
        .current_dir(root)
        .output()?;
    let commits_text = String::from_utf8_lossy(&recent.stdout);
    let all_lines: Vec<String> = commits_text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let total_commits_90d = all_lines.len();
    let recent_commits: Vec<String> = all_lines.into_iter().take(MAX_RECENT_COMMITS).collect();

    let name_only = Command::new("git")
        .args([
            "log",
            "--since=90 days ago",
            "--name-only",
            "--pretty=format:",
        ])
        .current_dir(root)
        .output()?;
    let files_text = String::from_utf8_lossy(&name_only.stdout);
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in files_text.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        *counts.entry(l.to_string()).or_insert(0) += 1;
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by_key(|p| std::cmp::Reverse(p.1));
    let top_files: Vec<(String, usize)> = pairs.into_iter().take(MAX_TOP_FILES).collect();

    Ok(GitActivity {
        total_commits_90d,
        recent_commits,
        top_files,
    })
}

fn detect_stacks(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (file, stack) in STACK_INDICATORS {
        if root.join(file).exists() && !out.contains(&stack.to_string()) {
            out.push(stack.to_string());
        }
    }
    out
}

fn truncate(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut cut = max_bytes;
    while !s.is_char_boundary(cut) && cut > 0 {
        cut -= 1;
    }
    format!("{}\n... (truncated at {} bytes)", &s[..cut], max_bytes)
}
