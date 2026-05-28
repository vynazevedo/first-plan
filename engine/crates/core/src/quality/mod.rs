//! Quality/Validation Layer (v0.8.0).
//!
//! Camada que captura o estado de validacao automatica do projeto:
//! - ci: workflows que rodam, jobs, steps, triggers
//! - coverage: % por arquivo, linhas nao cobertas, branches
//! - flaky: tests historicamente instaveis (via mining git log)
//!
//! Output: `.first-plan/11-quality/` com 00-pipeline.md + 01-coverage.md + 02-flaky.md
//!
//! Foundation para v0.9 (Schema/Contracts) que precisa saber quais endpoints tem teste,
//! v0.10 (Evolution) que valida migracoes completas, v0.11 (Production) que precisa de
//! CI history pra saber o que foi released.

pub mod ci;
pub mod coverage;
pub mod flaky;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub root: String,
    pub ci: ci::CiReport,
    pub coverage: coverage::CoverageReport,
    pub flaky: flaky::FlakyReport,
}

pub fn analyze(root: &std::path::Path) -> QualityReport {
    let start = std::time::Instant::now();
    let ci = ci::detect(root);
    let coverage = coverage::detect(root);
    let flaky = flaky::detect(root);
    QualityReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        root: root.to_string_lossy().into_owned(),
        ci,
        coverage,
        flaky,
    }
}
