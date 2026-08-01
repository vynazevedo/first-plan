//! Detecção automática de sibling repos em um diretório-pai.

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DetectedRepo {
    pub name: String,
    pub path: PathBuf,
    pub has_first_plan: bool,
    pub detected_stacks: Vec<String>,
}

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
    ("mix.exs", "elixir"),
];

/// Escaneia o diretório-pai buscando por sibling repos (dirs contendo `.git`).
/// Exclui o `self_root` da lista.
pub fn scan(parent_dir: &Path, self_root: &Path) -> Result<Vec<DetectedRepo>> {
    let self_canonical = self_root.canonicalize().unwrap_or(self_root.to_path_buf());
    let mut out = Vec::new();

    for entry in WalkDir::new(parent_dir).max_depth(2).into_iter().flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join(".git").exists() {
            continue;
        }
        let canonical = path.canonicalize().unwrap_or(path.to_path_buf());
        if canonical == self_canonical {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let has_first_plan = path.join(".first-plan").exists();
        let stacks = detect_stacks(path);
        out.push(DetectedRepo {
            name,
            path: canonical,
            has_first_plan,
            detected_stacks: stacks,
        });
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
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
