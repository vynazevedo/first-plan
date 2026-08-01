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

    let files = adapter.render(&ir, &output_base)?;

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
