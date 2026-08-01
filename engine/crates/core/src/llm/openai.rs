//! Provider OpenAI-compatível (também serve Ollama, LM Studio, vLLM, etc).

use super::provider::{ChatMessage, LlmError, LlmProvider};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OaiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct OaiRequest<'a> {
    model: &'a str,
    messages: Vec<OaiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    temperature: f32,
}

#[derive(Deserialize)]
struct OaiResponse {
    choices: Vec<OaiChoice>,
}

#[derive(Deserialize)]
struct OaiChoice {
    message: OaiChoiceMessage,
}

#[derive(Deserialize)]
struct OaiChoiceMessage {
    content: String,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: Option<u32>,
    ) -> Result<String, LlmError> {
        let body = OaiRequest {
            model: &self.model,
            messages: messages
                .iter()
                .map(|m| OaiMessage {
                    role: m.role.as_str(),
                    content: &m.content,
                })
                .collect(),
            max_tokens,
            temperature: 0.2,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let mut req = self.client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(LlmError::Http(format!("{}: {}", status, text)));
        }

        let parsed: OaiResponse = serde_json::from_str(&text)
            .map_err(|e| LlmError::InvalidResponse(format!("{} - body: {}", e, text)))?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::InvalidResponse("choices vazio".to_string()))?
            .message
            .content;
        Ok(content)
    }
}
