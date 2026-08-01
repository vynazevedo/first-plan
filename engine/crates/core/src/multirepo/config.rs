//! Schema e IO do arquivo `.first-plan/multi.yaml`.
//!
//! Estrutura versionada. Adicionamos `version: 1` para permitir migrações
//! futuras sem quebrar arquivos existentes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_REL_PATH: &str = ".first-plan/multi.yaml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiRepoConfig {
    pub version: u32,
    #[serde(default)]
    pub repos: Vec<RepoEntry>,
}

impl Default for MultiRepoConfig {
    fn default() -> Self {
        Self {
            version: 1,
            repos: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoEntry {
    pub name: String,
    /// Path relativo (preferido) ou absoluto. Resolvido a partir da raiz de projeto.
    pub path: PathBuf,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

pub fn config_path(project_root: &Path) -> PathBuf {
    project_root.join(CONFIG_REL_PATH)
}

pub fn load(project_root: &Path) -> Result<MultiRepoConfig> {
    let path = config_path(project_root);
    if !path.exists() {
        return Ok(MultiRepoConfig::default());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("lendo {}", path.display()))?;
    let cfg: MultiRepoConfig =
        serde_yaml::from_str(&text).with_context(|| format!("parseando {}", path.display()))?;
    Ok(cfg)
}

pub fn save(project_root: &Path, config: &MultiRepoConfig) -> Result<PathBuf> {
    let path = config_path(project_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_yaml::to_string(config)?;
    fs::write(&path, text).with_context(|| format!("gravando {}", path.display()))?;
    Ok(path)
}

/// Resolve o path de um RepoEntry para absoluto tendo o project_root como base.
pub fn resolved_path(project_root: &Path, entry: &RepoEntry) -> PathBuf {
    if entry.path.is_absolute() {
        entry.path.clone()
    } else {
        project_root.join(&entry.path)
    }
}
