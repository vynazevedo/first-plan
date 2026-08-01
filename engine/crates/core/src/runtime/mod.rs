//! Runtime Layer (v0.11.0).
//!
//! Link entre IR e o que esta em producao. Responde:
//! - Qual release cada arquivo mora?
//! - O que esta em main mas nao foi released ainda?
//! - Qual commit range compoe cada release?
//! - Esse bug esta na versao deployed ou so em main?
//!
//! Foundation para v0.12 Cross-repo (multi-service release awareness)
//! e futura integracao opcional com Sentry/Datadog webhooks.
//!
//! Sem dep externa - usa git CLI + CHANGELOG parsing local.

pub mod file_releases;
pub mod releases;
pub mod unreleased;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReport {
    pub generated_at: String,
    pub elapsed_ms: u64,
    pub root: String,
    pub releases: releases::ReleasesReport,
    pub unreleased: unreleased::UnreleasedReport,
    pub file_releases: file_releases::FileReleasesReport,
}

pub fn analyze(root: &std::path::Path) -> RuntimeReport {
    let start = std::time::Instant::now();
    let releases = releases::detect(root);
    let unreleased = unreleased::detect(root, &releases);
    let file_releases = file_releases::detect(root, &releases);
    RuntimeReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        root: root.to_string_lossy().into_owned(),
        releases,
        unreleased,
        file_releases,
    }
}
