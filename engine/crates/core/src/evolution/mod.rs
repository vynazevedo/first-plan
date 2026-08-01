//! Evolution Layer (v0.10.0).
//!
//! Deprecation e migration ledger. Captura como o codebase mudou ao longo do tempo:
//! - deprecations: itens marcados como depreciados em codigo ou CHANGELOG
//! - migrations: mudancas breaking em progresso via git history
//! - replacements: X foi substituido por Y no commit Z (rastreado)
//!
//! Serve para AI evitar sugerir padroes que o time ja substituiu. AI training data
//! fica desatualizada rapido, evolution layer traz constraints frescas por projeto.
//!
//! Foundation para v0.11 Runtime (validar que replacements foram deployed) e
//! v0.12 Cross-repo (mudanca de contrato em lib compartilhada afeta downstream).

pub mod deprecations;
pub mod migrations;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub root: String,
    pub deprecations: deprecations::DeprecationsReport,
    pub migrations: migrations::MigrationsReport,
}

pub fn analyze(root: &std::path::Path) -> EvolutionReport {
    let start = std::time::Instant::now();
    let deprecations = deprecations::detect(root);
    let migrations = migrations::detect(root);
    EvolutionReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        root: root.to_string_lossy().into_owned(),
        deprecations,
        migrations,
    }
}
