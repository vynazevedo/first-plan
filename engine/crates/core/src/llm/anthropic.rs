//! Provider Anthropic (Messages API).

use super::provider::{ChatMessage, LlmError, LlmProvider, Role};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
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
struct AntMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct AntRequest<'a> {
    model: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AntMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct AntResponse {
    content: Vec<AntBlock>,
}

#[derive(Deserialize)]
struct AntBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: Option<u32>,
    ) -> Result<String, LlmError> {
        let system = messages
            .iter()
            .filter(|m| matches!(m.role, Role::System))
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");
        let chat_msgs: Vec<AntMessage> = messages
            .iter()
            .filter(|m| !matches!(m.role, Role::System))
            .map(|m| AntMessage {
                role: m.role.as_str(),
                content: &m.content,
            })
            .collect();

        let body = AntRequest {
            model: &self.model,
            system: if system.is_empty() {
                None
            } else {
                Some(system)
            },
            messages: chat_msgs,
            max_tokens: max_tokens.unwrap_or(4096),
            temperature: 0.2,
        };

        let url = format!("{}/messages", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(LlmError::Http(format!("{}: {}", status, text)));
        }

        let parsed: AntResponse = serde_json::from_str(&text)
            .map_err(|e| LlmError::InvalidResponse(format!("{} - body: {}", e, text)))?;
        let content = parsed
            .content
            .into_iter()
            .filter(|b| b.kind == "text")
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        if content.is_empty() {
            return Err(LlmError::InvalidResponse("bloco text ausente".to_string()));
        }
        Ok(content)
    }
}
