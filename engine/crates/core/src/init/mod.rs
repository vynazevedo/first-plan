//! Init LLM-agnostic (v1.1.0): gera `.first-plan/` sem depender de Claude Code.
//!
//! Pipeline:
//! 1. `signals::collect` captura sinais estáveis do projeto (README, manifests, tree, git log)
//! 2. Para cada `LayerSpec`, monta prompt combinando sinais + instrução específica
//! 3. Chama `LlmProvider::chat` e escreve markdown resultante em `.first-plan/<path>`
//! 4. Adiciona frontmatter YAML padrão (section, confidence, generated_at, provider)
//!
//! Uso programático via `run_init`. Interface CLI: subcommand `init --llm`.

pub mod layers;
pub mod signals;

use crate::llm::{ChatMessage, LlmProvider};
use anyhow::{Context, Result};
use chrono::Utc;
use layers::LayerSpec;
use serde::Serialize;
use signals::ProjectSignals;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct InitReport {
    pub root: PathBuf,
    pub provider: String,
    pub model: String,
    pub layers_generated: Vec<LayerResult>,
    pub layers_skipped: Vec<LayerSkip>,
    pub elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
pub struct LayerResult {
    pub name: String,
    pub output_path: PathBuf,
    pub bytes_written: usize,
    pub elapsed_ms: u128,
}

#[derive(Debug, Serialize)]
pub struct LayerSkip {
    pub name: String,
    pub reason: String,
}

pub struct InitOptions {
    pub root: PathBuf,
    pub layer_filter: Option<Vec<String>>,
    pub overwrite: bool,
    pub max_tokens: Option<u32>,
}

/// Orquestra a geração assíncrona de layers.
pub async fn run_init(provider: &dyn LlmProvider, opts: InitOptions) -> Result<InitReport> {
    let start = Instant::now();
    let signals = signals::collect(&opts.root).with_context(|| "coletando sinais do projeto")?;

    let signals_summary = serialize_signals(&signals)?;
    let first_plan_dir = opts.root.join(".first-plan");
    fs::create_dir_all(&first_plan_dir)?;

    let selected: Vec<&LayerSpec> = match &opts.layer_filter {
        Some(filter) => layers::all_layers()
            .iter()
            .filter(|l| filter.iter().any(|f| l.name == f))
            .collect(),
        None => layers::all_layers().iter().collect(),
    };

    let mut generated = Vec::new();
    let mut skipped = Vec::new();

    for spec in selected {
        let output_path = first_plan_dir.join(spec.output_path);
        if output_path.exists() && !opts.overwrite {
            skipped.push(LayerSkip {
                name: spec.name.to_string(),
                reason: "arquivo existe (use --overwrite)".to_string(),
            });
            continue;
        }

        let layer_start = Instant::now();
        let content = generate_layer(provider, spec, &signals_summary, opts.max_tokens).await?;
        let full_content = wrap_with_frontmatter(spec, provider, &content);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, &full_content)?;

        generated.push(LayerResult {
            name: spec.name.to_string(),
            output_path: output_path.clone(),
            bytes_written: full_content.len(),
            elapsed_ms: layer_start.elapsed().as_millis(),
        });
    }

    Ok(InitReport {
        root: opts.root.clone(),
        provider: provider.name().to_string(),
        model: provider.model().to_string(),
        layers_generated: generated,
        layers_skipped: skipped,
        elapsed_ms: start.elapsed().as_millis(),
    })
}

async fn generate_layer(
    provider: &dyn LlmProvider,
    spec: &LayerSpec,
    signals_json: &str,
    max_tokens: Option<u32>,
) -> Result<String> {
    let system = format!(
        "Você é o gerador de layers do first-plan (compiled context layer). \
Layer alvo: {}. Regras: markdown puro sem frontmatter, sem emojis, sem \
menções a IA/Claude/LLM. Baseie-se apenas nos sinais fornecidos. Marque \
`TBD` quando faltar evidência.",
        spec.name
    );
    let user = format!(
        "{}\n\n---\n\nSinais do projeto (JSON):\n\n```json\n{}\n```",
        spec.prompt, signals_json
    );
    let messages = vec![ChatMessage::system(system), ChatMessage::user(user)];
    provider
        .chat(&messages, max_tokens)
        .await
        .map_err(|e| anyhow::anyhow!("LLM chat falhou para layer {}: {}", spec.name, e))
}

fn wrap_with_frontmatter(spec: &LayerSpec, provider: &dyn LlmProvider, content: &str) -> String {
    let ts = Utc::now().to_rfc3339();
    let fm = format!(
        "---\nsection: {}\nconfidence: 0.6\ngenerated_at: {}\ngenerated_by: first-plan-engine {}\nprovider: {}\nmodel: {}\n---\n\n",
        spec.name,
        ts,
        crate::ENGINE_VERSION,
        provider.name(),
        provider.model(),
    );
    format!("{}{}", fm, content.trim_end())
}

fn serialize_signals(signals: &ProjectSignals) -> Result<String> {
    Ok(serde_json::to_string_pretty(signals)?)
}

/// Retorna o caminho absoluto para uma layer relativa a `.first-plan/`.
pub fn layer_output_path(root: &Path, layer_rel: &str) -> PathBuf {
    root.join(".first-plan").join(layer_rel)
}
