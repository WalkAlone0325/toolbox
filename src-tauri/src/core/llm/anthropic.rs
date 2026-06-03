use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

use super::{LLMConfig, LLMEvent, LLMProvider, LLMRequest, read_sse_stream};

pub struct AnthropicProvider {
    config: LLMConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, client }
    }

    fn endpoint(&self) -> String {
        let base = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
        let trimmed = base.trim_end_matches('/');
        format!("{}/v1/messages", trimmed)
    }
}

#[derive(Serialize)]
struct AnthropicPayload {
    model: String,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat_stream(
        &self,
        request: LLMRequest,
        cancel: std::sync::Arc<tokio::sync::Notify>,
    ) -> Result<tokio::sync::mpsc::Receiver<LLMEvent>, String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| "Anthropic API key not configured".to_string())?
            .clone();

        let mut system_prompt: Option<String> = None;
        let mut user_messages = request.messages.clone();
        if !user_messages.is_empty() && user_messages[0].role == "system" {
            system_prompt = Some(user_messages.remove(0).content);
        }
        let messages: Vec<AnthropicMessage> = user_messages
            .into_iter()
            .map(|m| AnthropicMessage {
                role: if m.role == "assistant" {
                    "assistant".to_string()
                } else {
                    "user".to_string()
                },
                content: m.content,
            })
            .collect();

        let payload = AnthropicPayload {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(2048),
            stream: true,
            temperature: request.temperature,
            system: system_prompt,
            messages,
        };

        let endpoint = self.endpoint();
        let response = self
            .client
            .post(&endpoint)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Accept", "text/event-stream")
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, body));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let cancel_handle = cancel;
        tokio::spawn(async move {
            read_sse_stream(response, tx, cancel_handle, |v| {
                let event = v.get("type").and_then(|s| s.as_str());
                if event == Some("content_block_delta") {
                    v.get("delta")
                        .and_then(|d| d.get("text"))
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .await;
        });
        Ok(rx)
    }
}
