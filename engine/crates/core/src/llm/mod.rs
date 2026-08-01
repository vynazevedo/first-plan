//! LLM provider abstraction (v1.1.0).
//!
//! Unified interface para invocar diferentes provedores de LLM (OpenAI,
//! Anthropic, Ollama e qualquer endpoint OpenAI-compatível). Serve como
//! motor de geração para o subcommand `init --llm`, que produz layers
//! do `.first-plan/` sem depender de Claude Code.
//!
//! Config via variáveis de ambiente:
//! - `FIRST_PLAN_LLM_PROVIDER` = openai | anthropic | ollama
//! - `FIRST_PLAN_LLM_MODEL` = model id (default varia por provider)
//! - `FIRST_PLAN_LLM_BASE_URL` = override do endpoint HTTP
//! - `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` = chaves de auth

pub mod anthropic;
pub mod openai;
pub mod provider;

pub use provider::{ChatMessage, LlmError, LlmProvider, ProviderKind, Role};

use std::env;

/// Constrói um provider a partir de env vars ou dos argumentos passados.
pub fn build(
    kind: Option<ProviderKind>,
    model: Option<String>,
    base_url: Option<String>,
) -> Result<Box<dyn LlmProvider>, LlmError> {
    let kind = kind
        .or_else(|| {
            env::var("FIRST_PLAN_LLM_PROVIDER")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(ProviderKind::Openai);

    let model = model.or_else(|| env::var("FIRST_PLAN_LLM_MODEL").ok());
    let base_url = base_url.or_else(|| env::var("FIRST_PLAN_LLM_BASE_URL").ok());

    match kind {
        ProviderKind::Openai | ProviderKind::Ollama => {
            let default_base = if matches!(kind, ProviderKind::Ollama) {
                "http://localhost:11434/v1".to_string()
            } else {
                "https://api.openai.com/v1".to_string()
            };
            let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
            let default_model = if matches!(kind, ProviderKind::Ollama) {
                "qwen2.5-coder:latest".to_string()
            } else {
                "gpt-4o-mini".to_string()
            };
            Ok(Box::new(openai::OpenAiProvider::new(
                base_url.unwrap_or(default_base),
                api_key,
                model.unwrap_or(default_model),
            )))
        }
        ProviderKind::Anthropic => {
            let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| {
                LlmError::MissingConfig("ANTHROPIC_API_KEY não definida".to_string())
            })?;
            Ok(Box::new(anthropic::AnthropicProvider::new(
                base_url.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
                api_key,
                model.unwrap_or_else(|| "claude-sonnet-5".to_string()),
            )))
        }
    }
}
