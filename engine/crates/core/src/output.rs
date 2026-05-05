//! JSON output schemas and writers.
//!
//! All outputs include `$schema` and `engine_version` so consumers (skills) can
//! detect format changes safely.

use crate::cochange::{Cluster, Pair};
use crate::hash::FileHash;
use crate::index::IndexStats;
use crate::search::SearchHit;
use crate::ENGINE_VERSION;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoChangeOutput {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub engine_version: String,
    pub generated_at: DateTime<Utc>,
    pub repo_root: String,
    pub window_days: u32,
    pub total_commits_analyzed: u32,
    pub total_files_analyzed: u32,
    pub pairs: Vec<Pair>,
    pub clusters: Vec<Cluster>,
    pub elapsed_ms: u64,
}

impl CoChangeOutput {
    pub fn new(
        repo_root: impl Into<String>,
        window_days: u32,
        total_commits: u32,
        total_files: u32,
        pairs: Vec<Pair>,
        clusters: Vec<Cluster>,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            schema: "first-plan-cochange-v1".into(),
            engine_version: ENGINE_VERSION.into(),
            generated_at: Utc::now(),
            repo_root: repo_root.into(),
            window_days,
            total_commits_analyzed: total_commits,
            total_files_analyzed: total_files,
            pairs,
            clusters,
            elapsed_ms,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashOutput {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub engine_version: String,
    pub generated_at: DateTime<Utc>,
    pub algorithm: String,
    pub files: BTreeMap<String, FileHash>,
    pub elapsed_ms: u64,
}

impl HashOutput {
    pub fn new(files: BTreeMap<String, FileHash>, elapsed_ms: u64) -> Self {
        Self {
            schema: "first-plan-hash-v1".into(),
            engine_version: ENGINE_VERSION.into(),
            generated_at: Utc::now(),
            algorithm: "xxh3_64".into(),
            files,
            elapsed_ms,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexOutput {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub engine_version: String,
    pub generated_at: DateTime<Utc>,
    pub repo_root: String,
    pub db_path: String,
    pub total_symbols: u32,
    pub total_doc_length: u32,
    pub avg_doc_length: f64,
    pub elapsed_ms: u64,
}

impl IndexOutput {
    pub fn new(
        repo_root: impl Into<String>,
        db_path: impl Into<String>,
        stats: IndexStats,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            schema: "first-plan-index-v1".into(),
            engine_version: ENGINE_VERSION.into(),
            generated_at: Utc::now(),
            repo_root: repo_root.into(),
            db_path: db_path.into(),
            total_symbols: stats.total_symbols,
            total_doc_length: stats.total_doc_length,
            avg_doc_length: stats.avg_doc_length,
            elapsed_ms,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOutput {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub engine_version: String,
    pub generated_at: DateTime<Utc>,
    pub query: String,
    pub limit: u32,
    pub hits: Vec<SearchHit>,
    pub elapsed_ms: u64,
}

impl SearchOutput {
    pub fn new(
        query: impl Into<String>,
        limit: u32,
        hits: Vec<SearchHit>,
        elapsed_ms: u64,
    ) -> Self {
        Self {
            schema: "first-plan-search-v1".into(),
            engine_version: ENGINE_VERSION.into(),
            generated_at: Utc::now(),
            query: query.into(),
            limit,
            hits,
            elapsed_ms,
        }
    }
}

/// Write a serializable value to disk as pretty JSON, or to stdout if path is `-`.
pub fn write_json<T: Serialize>(value: &T, path: &str) -> Result<()> {
    let s = serde_json::to_string_pretty(value).context("serialize JSON")?;
    if path == "-" {
        println!("{}", s);
        return Ok(());
    }
    std::fs::write(Path::new(path), s).with_context(|| format!("write {}", path))?;
    Ok(())
}
