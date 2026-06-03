use async_trait::async_trait;
use serde::Serialize;
use std::time::Duration;

use super::{LLMConfig, LLMEvent, LLMProvider, LLMRequest, read_sse_stream};

pub struct OllamaProvider {
    config: LLMConfig,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(config: LLMConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { config, client }
    }

    fn endpoint(&self) -> String {
        let base = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        let trimmed = base.trim_end_matches('/');
        format!("{}/api/chat", trimmed)
    }
}

#[derive(Serialize)]
struct OllamaPayload {
    model: String,
    stream: bool,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn chat_stream(
        &self,
        request: LLMRequest,
        cancel: std::sync::Arc<tokio::sync::Notify>,
    ) -> Result<tokio::sync::mpsc::Receiver<LLMEvent>, String> {
        let options = if request.temperature.is_some() || request.max_tokens.is_some() {
            Some(OllamaOptions {
                temperature: request.temperature.unwrap_or(0.7),
                num_predict: request.max_tokens.unwrap_or(2048),
            })
        } else {
            None
        };

        let messages: Vec<OllamaMessage> = request
            .messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let payload = OllamaPayload {
            model: request.model.clone(),
            stream: true,
            messages,
            options,
        };

        let endpoint = self.endpoint();
        let response = self
            .client
            .post(&endpoint)
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
                v.get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            })
            .await;
        });
        Ok(rx)
    }
}
