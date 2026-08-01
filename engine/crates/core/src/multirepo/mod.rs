//! Cross-repo awareness (v1.2.0): registro, escaneamento e agregação de sibling repos.
//!
//! Persiste config em `.first-plan/multi.yaml`. Cada entry aponta para outro repo
//! (path relativo ao project_root ou absoluto). O comando `aggregate` produz
//! `.first-plan/multi/OVERVIEW.md` com um sumário cross-repo (tabela de repos +
//! excerptos de mission/stacks de cada um, quando disponível).
//!
//! Escopo v1.2.0: registro manual, autodetecção via `scan`, agregação simples.
//! Futuro (v1.3+): resolução de referências cross-repo, diff de contratos, alertas
//! quando um repo consome API que outro repo mudou.

pub mod aggregate;
pub mod config;
pub mod scan;

pub use aggregate::{aggregate, AggregateReport, RepoStatus};
pub use config::{config_path, load, resolved_path, save, MultiRepoConfig, RepoEntry};
pub use scan::{scan, DetectedRepo};
