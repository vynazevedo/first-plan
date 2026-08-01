//! Trait comum a todos os LLM providers.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("erro HTTP: {0}")]
    Http(String),
    #[error("resposta inválida: {0}")]
    InvalidResponse(String),
    #[error("configuração ausente: {0}")]
    MissingConfig(String),
    #[error("provider não suportado: {0}")]
    Unsupported(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        LlmError::Http(err.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Openai,
    Anthropic,
    Ollama,
}

impl FromStr for ProviderKind {
    type Err = LlmError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "openai" => Ok(Self::Openai),
            "anthropic" => Ok(Self::Anthropic),
            "ollama" => Ok(Self::Ollama),
            other => Err(LlmError::Unsupported(other.to_string())),
        }
    }
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Nome canônico do provider (para logs e JSON output).
    fn name(&self) -> &'static str;

    /// Modelo em uso.
    fn model(&self) -> &str;

    /// Executa uma completion. `max_tokens` é apenas hint.
    async fn chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: Option<u32>,
    ) -> Result<String, LlmError>;
}
