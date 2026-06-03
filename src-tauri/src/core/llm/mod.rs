pub mod ollama;
pub mod openai;
pub mod anthropic;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub model: String,
    pub messages: Vec<LLMMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LLMEvent {
    Chunk(String),
    Done(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
}

impl LLMConfig {
    pub fn provider(&self) -> Result<Box<dyn LLMProvider + Send>, String> {
        match self.provider.as_str() {
            "openai" => Ok(Box::new(openai::OpenAIProvider::new(self.clone()))),
            "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(self.clone()))),
            "ollama" => Ok(Box::new(ollama::OllamaProvider::new(self.clone()))),
            other => Err(format!("unknown provider: {}", other)),
        }
    }
}

#[async_trait]
pub trait LLMProvider {
    async fn chat_stream(
        &self,
        request: LLMRequest,
        cancel: std::sync::Arc<tokio::sync::Notify>,
    ) -> Result<tokio::sync::mpsc::Receiver<LLMEvent>, String>;
}

pub(crate) async fn decode_sse_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(':') {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("data: ") {
        if rest == "[DONE]" {
            return None;
        }
        return Some(rest.to_string());
    }
    None
}

pub(crate) async fn read_sse_stream(
    response: reqwest::Response,
    sender: tokio::sync::mpsc::Sender<LLMEvent>,
    cancel: std::sync::Arc<tokio::sync::Notify>,
    extract_delta: impl Fn(serde_json::Value) -> Option<String> + Send + 'static,
) {
    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    let mut full = String::new();
    loop {
        tokio::select! {
            _ = cancel.notified() => {
                let _ = sender.send(LLMEvent::Done(full.clone())).await;
                return;
            }
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        buf.push_str(&String::from_utf8_lossy(&Bytes::from(bytes)));
                        loop {
                            let nl = buf.find('\n');
                            if nl.is_none() { break; }
                            let idx = nl.unwrap();
                            let line: String = buf.drain(..=idx).collect::<String>();
                            if let Some(json_str) = decode_sse_line(&line).await {
                                match serde_json::from_str::<serde_json::Value>(&json_str) {
                                    Ok(v) => {
                                        if let Some(delta) = extract_delta(v) {
                                            if !delta.is_empty() {
                                                full.push_str(&delta);
                                                if sender.send(LLMEvent::Chunk(delta)).await.is_err() {
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("parse llm chunk failed: {} | {}", e, json_str);
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        let _ = sender.send(LLMEvent::Error(e.to_string())).await;
                        return;
                    }
                    None => {
                        let _ = sender.send(LLMEvent::Done(full)).await;
                        return;
                    }
                }
            }
        }
    }
}
