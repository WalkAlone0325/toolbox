use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

use super::{LLMConfig, LLMEvent, LLMProvider, LLMRequest, read_sse_stream};

pub struct OpenAIProvider {
    config: LLMConfig,
    client: reqwest::Client,
}

impl OpenAIProvider {
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
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let trimmed = base.trim_end_matches('/');
        format!("{}/chat/completions", trimmed)
    }
}

#[derive(Serialize)]
struct OpenAIStreamPayload<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat_stream(
        &self,
        request: LLMRequest,
        cancel: std::sync::Arc<tokio::sync::Notify>,
    ) -> Result<tokio::sync::mpsc::Receiver<LLMEvent>, String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| "OpenAI API key not configured".to_string())?
            .clone();

        let messages: Vec<OpenAIMessage> = request
            .messages
            .iter()
            .map(|m| OpenAIMessage {
                role: m.role.as_str(),
                content: m.content.as_str(),
            })
            .collect();

        let payload = OpenAIStreamPayload {
            model: &request.model,
            messages,
            stream: true,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let endpoint = self.endpoint();
        let req = self
            .client
            .post(&endpoint)
            .bearer_auth(&api_key)
            .json(&payload)
            .header("Accept", "text/event-stream");

        let response = req.send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, body));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let cancel_handle = cancel;
        tokio::spawn(async move {
            read_sse_stream(response, tx, cancel_handle, |v| {
                v.get("choices")
                    .and_then(|c| c.get(0))
                    .and_then(|c| c.get("delta"))
                    .and_then(|d| d.get("content"))
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            })
            .await;
        });
        Ok(rx)
    }
}
