//! Generate tool-specific instruction files from `.first-plan/` IR (v1.0.0).
//!
//! Framework pivot: transforma first-plan de "Claude Code plugin" para
//! "context layer for any AI coding tool". Le o IR compilado e cospe arquivos
//! no formato que cada tool espera nativamente:
//!
//! - Codex (OpenAI): AGENTS.md
//! - Cursor: .cursorrules + .cursor/rules/*.mdc
//! - GitHub Copilot: .github/copilot-instructions.md
//! - Cline (VS Code): .clinerules
//! - Generic: CONVENTIONS.md (universal, funciona pra Aider e outros)
//!
//! Sem integracao viva, sem API - so file generation. AI tool le o arquivo
//! nativamente e ganha contexto compilado que first-plan produziu.

pub mod adapters;
pub mod context;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tera::Tera;

pub use context::IrContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub tool: String,
    pub output_path: String,
    pub files_written: Vec<String>,
    pub bytes_written: usize,
}

pub fn generate(root: &Path, tool: &str, output_dir: Option<&Path>) -> Result<GenerateReport> {
    let start = std::time::Instant::now();

    let ir = context::load_ir(root)?;
    let adapter = adapters::get(tool).context(format!("unknown tool adapter: {}", tool))?;

    let output_base = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.to_path_buf());

    std::fs::create_dir_all(&output_base).context("failed to create output root")?;
    let output_base = output_base
        .canonicalize()
        .context("failed to resolve output root")?;
    let rendered = adapter.render(&ir, &output_base)?;
    let mut files = Vec::new();
    for (path, content) in rendered {
        for ancestor in path.ancestors().take_while(|p| *p != output_base.as_path()) {
            if let Ok(meta) = std::fs::symlink_metadata(ancestor) {
                anyhow::ensure!(
                    !meta.file_type().is_symlink(),
                    "refusing symlink output: {}",
                    ancestor.display()
                );
            }
        }
        let existing = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };
        files.push((path, merge_managed(&existing, &content)?));
    }

    let mut bytes_written = 0;
    for (path, content) in &files {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("failed to create output dir")?;
        }
        std::fs::write(path, content)
            .with_context(|| format!("failed to write {}", path.display()))?;
        bytes_written += content.len();
    }

    Ok(GenerateReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        tool: tool.to_string(),
        output_path: output_base.to_string_lossy().into_owned(),
        files_written: files
            .iter()
            .map(|(p, _)| p.to_string_lossy().into_owned())
            .collect(),
        bytes_written,
    })
}

pub fn list_adapters() -> Vec<AdapterInfo> {
    adapters::all().iter().map(|a| a.info()).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub description: String,
    pub output_files: Vec<String>,
}

pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    fn info(&self) -> AdapterInfo;
    fn render(&self, ir: &IrContext, output_base: &Path) -> Result<Vec<(PathBuf, String)>>;
}

pub(crate) fn render_template(template_str: &str, ctx: &tera::Context) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("t", template_str)
        .context("failed to load template")?;
    tera.render("t", ctx).context("failed to render template")
}

const START: &str = "<!-- first-plan:begin -->";
const END: &str = "<!-- first-plan:end -->";

fn merge_managed(existing: &str, generated: &str) -> Result<String> {
    anyhow::ensure!(
        !generated.contains(START) && !generated.contains(END),
        "input contains reserved managed markers"
    );
    let (frontmatter, body) = if let Some(rest) = generated.strip_prefix("---\n") {
        match rest.find("\n---\n") {
            Some(end) => generated.split_at(end + 9),
            None => ("", generated),
        }
    } else {
        ("", generated)
    };
    let block = format!("{}\n{}\n{}", START, body.trim(), END);
    match (existing.find(START), existing.find(END)) {
        (Some(start), Some(end)) => {
            anyhow::ensure!(
                start < end
                    && existing.matches(START).count() == 1
                    && existing.matches(END).count() == 1,
                "ambiguous managed markers; existing file preserved"
            );
            Ok(format!(
                "{}{}{}",
                &existing[..start],
                block,
                &existing[end + END.len()..]
            ))
        }
        (None, None) if existing.is_empty() => Ok(format!("{}{}\n", frontmatter, block)),
        (None, None) => Ok(format!("{}\n\n{}\n", existing, block)),
        _ => anyhow::bail!("incomplete managed markers; existing file preserved"),
    }
}

#[cfg(test)]
mod preservation_tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn accepts_root_alias_but_rejects_symlinked_instruction_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("actual");
        std::fs::create_dir(&root).unwrap();
        let alias = tmp.path().join("alias");
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        generate(&alias, "generic", None).unwrap();
        let target = tmp.path().join("user-rules.md");
        std::fs::write(&target, "user rules").unwrap();
        std::os::unix::fs::symlink(&target, root.join("AGENTS.md")).unwrap();
        assert!(generate(&alias, "codex", None).is_err());
        assert_eq!(std::fs::read_to_string(target).unwrap(), "user rules");
    }
    #[test]
    fn preserves_user_content_and_refreshes_idempotently() {
        let initial = "# Team rules\nDo not delete this.\n";
        let first = merge_managed(initial, "generated v1").unwrap();
        let second = merge_managed(&first, "generated v2").unwrap();
        assert!(second.starts_with(initial.trim_end()));
        assert!(!second.contains("generated v1"));
        assert_eq!(second, merge_managed(&second, "generated v2").unwrap());
        assert!(merge_managed("<!-- first-plan:begin -->", "new").is_err());
    }
    #[test]
    fn keeps_mdc_frontmatter_first() {
        let text = merge_managed("", "---\nalwaysApply: true\n---\nbody").unwrap();
        assert!(text.starts_with("---\nalwaysApply: true\n---\n"));
        assert!(text.contains("\nbody\n"));
    }
}
