//! Bounded, local evidence shared by discovery and task context.

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub path: String,
    pub line: usize,
    pub hash: String,
    pub kind: String,
}

pub fn hash(content: &[u8]) -> String {
    format!("xxh3:{:016x}", xxhash_rust::xxh3::xxh3_64(content))
}

pub fn revision(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

pub fn allowed(path: &Path) -> bool {
    !path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        matches!(
            s.as_ref(),
            ".git"
                | "node_modules"
                | "target"
                | "vendor"
                | ".venv"
                | "venv"
                | "dist"
                | "build"
                | ".first-plan"
                | ".cache"
                | "__pycache__"
        ) || s.starts_with(".env")
            || s.ends_with(".pem")
            || s.ends_with(".key")
    })
}

pub fn files(root: &Path) -> Result<Vec<PathBuf>> {
    ensure!(root.is_dir(), "project root must be a directory");
    let out = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            ".",
        ])
        .current_dir(root)
        .output();
    let mut paths: Vec<PathBuf> = if let Some(out) = out.ok().filter(|o| o.status.success()) {
        out.stdout
            .split(|b| *b == 0)
            .filter(|b| !b.is_empty())
            .filter_map(|b| std::str::from_utf8(b).ok())
            .map(PathBuf::from)
            .collect()
    } else {
        WalkDir::new(root)
            .max_depth(12)
            .into_iter()
            .filter_entry(|e| e.path().strip_prefix(root).map(allowed).unwrap_or(false))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.path().strip_prefix(root).ok().map(PathBuf::from))
            .collect()
    };
    paths.retain(|p| {
        allowed(p)
            && !p.is_absolute()
            && !p
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
    });
    paths.sort();
    paths.dedup();
    ensure!(
        paths.len() <= 50_000,
        "project exceeds 50,000 files; select a smaller --root"
    );
    Ok(paths)
}

pub fn read(root: &Path, relative: &Path) -> Option<String> {
    let root = root.canonicalize().ok()?;
    let path = root.join(relative).canonicalize().ok()?;
    if !path.starts_with(&root) || std::fs::metadata(&path).ok()?.len() > 256_000 {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    (!content.contains('\0')).then_some(content)
}

pub fn source(path: &Path, text: &str, line: usize, kind: &str) -> Evidence {
    Evidence {
        path: path.to_string_lossy().replace('\\', "/"),
        line,
        hash: hash(text.as_bytes()),
        kind: kind.into(),
    }
}
