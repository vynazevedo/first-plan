//! Agrega IR de múltiplos repos em um overview cross-repo.

use super::config::{resolved_path, MultiRepoConfig, RepoEntry};
use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct AggregateReport {
    pub project_root: PathBuf,
    pub output_path: PathBuf,
    pub repos: Vec<RepoStatus>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct RepoStatus {
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub exists: bool,
    pub has_first_plan: bool,
    pub layers_found: Vec<String>,
    pub purpose_excerpt: Option<String>,
    pub stacks_excerpt: Option<String>,
}

pub fn aggregate(project_root: &Path, cfg: &MultiRepoConfig) -> Result<AggregateReport> {
    let mut statuses = Vec::new();
    for entry in &cfg.repos {
        statuses.push(collect_status(project_root, entry));
    }

    let output_path = project_root.join(".first-plan/multi/OVERVIEW.md");
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let generated_at = Utc::now().to_rfc3339();
    let markdown = render_markdown(project_root, &statuses, &generated_at);
    fs::write(&output_path, markdown)?;

    Ok(AggregateReport {
        project_root: project_root.to_path_buf(),
        output_path,
        repos: statuses,
        generated_at,
    })
}

fn collect_status(project_root: &Path, entry: &RepoEntry) -> RepoStatus {
    let abs = resolved_path(project_root, entry);
    let exists = abs.exists();
    let first_plan_dir = abs.join(".first-plan");
    let has_first_plan = first_plan_dir.is_dir();

    let mut layers_found = Vec::new();
    if has_first_plan {
        if let Ok(iter) = fs::read_dir(&first_plan_dir) {
            for e in iter.flatten() {
                if let Some(name) = e.file_name().to_str() {
                    layers_found.push(name.to_string());
                }
            }
            layers_found.sort();
        }
    }

    let purpose_excerpt = read_excerpt(&first_plan_dir.join("00-mission/purpose.md"), 400);
    let stacks_excerpt = read_excerpt(&first_plan_dir.join("01-topology/stacks.md"), 400);

    RepoStatus {
        name: entry.name.clone(),
        path: abs,
        tags: entry.tags.clone(),
        exists,
        has_first_plan,
        layers_found,
        purpose_excerpt,
        stacks_excerpt,
    }
}

fn read_excerpt(path: &Path, max_chars: usize) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let body = strip_frontmatter(&text);
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(clip_chars(trimmed, max_chars))
}

fn strip_frontmatter(text: &str) -> &str {
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---") {
            let after = &rest[end + 4..];
            return after.trim_start_matches('\n');
        }
    }
    text
}

fn clip_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut buf = String::new();
    for (i, c) in text.chars().enumerate() {
        if i >= max {
            break;
        }
        buf.push(c);
    }
    buf.push_str("...");
    buf
}

fn render_markdown(project_root: &Path, statuses: &[RepoStatus], generated_at: &str) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str("section: multi/overview\n");
    out.push_str(&format!("generated_at: {}\n", generated_at));
    out.push_str(&format!(
        "generated_by: first-plan-engine {}\n",
        crate::ENGINE_VERSION
    ));
    out.push_str(&format!("project_root: {}\n", project_root.display()));
    out.push_str(&format!("total_repos: {}\n", statuses.len()));
    out.push_str("---\n\n");

    out.push_str("# Overview cross-repo\n\n");
    out.push_str(
        "Snapshot agregado dos repos registrados em `.first-plan/multi.yaml`. \
Regenerado por `first-plan-engine multi aggregate`.\n\n",
    );

    out.push_str("## Repos registrados\n\n");
    if statuses.is_empty() {
        out.push_str("_Nenhum repo registrado. Use `multi register` para adicionar._\n\n");
    } else {
        out.push_str("| Nome | Path | Tags | Existe | IR presente | Layers |\n");
        out.push_str("|------|------|------|--------|-------------|--------|\n");
        for s in statuses {
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} | {} |\n",
                s.name,
                s.path.display(),
                if s.tags.is_empty() {
                    "-".to_string()
                } else {
                    s.tags.join(", ")
                },
                if s.exists { "sim" } else { "não" },
                if s.has_first_plan { "sim" } else { "não" },
                s.layers_found.len(),
            ));
        }
        out.push('\n');
    }

    for s in statuses {
        out.push_str(&format!("## {}\n\n", s.name));
        out.push_str(&format!("- Path: `{}`\n", s.path.display()));
        if !s.tags.is_empty() {
            out.push_str(&format!("- Tags: {}\n", s.tags.join(", ")));
        }
        out.push_str(&format!(
            "- Existe: {} | IR presente: {}\n",
            if s.exists { "sim" } else { "não" },
            if s.has_first_plan { "sim" } else { "não" }
        ));

        if let Some(p) = &s.purpose_excerpt {
            out.push_str("\n### Propósito (mission/purpose)\n\n");
            out.push_str(p);
            out.push_str("\n\n");
        }
        if let Some(t) = &s.stacks_excerpt {
            out.push_str("\n### Stacks (topology/stacks)\n\n");
            out.push_str(t);
            out.push_str("\n\n");
        }
        if !s.has_first_plan {
            out.push_str(
                "\n> Este repo não tem `.first-plan/`. \
Considere rodar `first-plan-engine init` nele para popular o contexto.\n\n",
            );
        }
    }

    out
}
