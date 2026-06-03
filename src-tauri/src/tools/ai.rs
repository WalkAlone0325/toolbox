use crate::core::db::Database;
use crate::core::llm::{LLMConfig, LLMEvent, LLMMessage, LLMRequest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tauri::ipc::Channel;
use tokio::sync::Notify;

type DbState<'a> = State<'a, Arc<Mutex<Database>>>;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AIStreamEvent {
    Chunk { text: String },
    Done { text: String },
    Error { message: String },
}

#[derive(serde::Deserialize)]
pub struct AITransformInput {
    pub action: String,
    pub content: String,
    pub custom_prompt: Option<String>,
}

pub struct AIAbortRegistry {
    pub handles: Mutex<HashMap<String, Arc<Notify>>>,
}

impl AIAbortRegistry {
    pub fn new() -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, id: String, notify: Arc<Notify>) {
        let mut map = self.handles.lock().unwrap();
        map.insert(id, notify);
    }

    pub fn remove(&self, id: &str) -> Option<Arc<Notify>> {
        let mut map = self.handles.lock().unwrap();
        map.remove(id)
    }
}

fn build_messages(action: &str, content: &str, custom_prompt: Option<&str>) -> Vec<LLMMessage> {
    let system = match action {
        "translate" => "你是专业翻译助手。请将用户给出的文本翻译为英文，保留原文格式，仅输出译文，不要解释。",
        "translate_zh" => "你是专业翻译助手。请将用户给出的文本翻译为简体中文，保留原文格式，仅输出译文，不要解释。",
        "summarize" => "你是内容总结专家。请用简洁的中文要点列出用户给出文本的核心信息，不超过 5 条，使用 markdown 列表格式。",
        "polish" => "你是文本润色专家。请润色用户给出的中文文本，使其更通顺、专业，但保持原意不变。仅输出润色后的文本，不要解释。",
        "extract" => "你是信息提取专家。请从用户文本中提取关键信息（实体、数字、日期、要点），用 markdown 列表呈现。",
        "rewrite_formal" => "你是写作助手。请将用户文本改写为正式商务语气，仅输出改写后的文本，不要解释。",
        "rewrite_casual" => "你是写作助手。请将用户文本改写为轻松自然的口语化语气，仅输出改写后的文本，不要解释。",
        "explain" => "你是技术解释专家。请用通俗易懂的中文解释用户给出的内容，必要时用 markdown 列出要点。",
        "custom" => custom_prompt.unwrap_or("请按用户要求处理文本。"),
        _ => custom_prompt.unwrap_or("请处理以下文本。"),
    };
    vec![
        LLMMessage {
            role: "system".to_string(),
            content: system.to_string(),
        },
        LLMMessage {
            role: "user".to_string(),
            content: content.to_string(),
        },
    ]
}

#[tauri::command]
pub async fn ai_transform(
    app: AppHandle,
    db: DbState<'_>,
    input: AITransformInput,
    on_event: Channel<AIStreamEvent>,
) -> Result<String, String> {
    let config = {
        let guard = db.lock().map_err(|e| e.to_string())?;
        let s = guard.load_settings();
        let provider = s.llm_provider.clone().ok_or_else(|| {
            "未配置 LLM Provider，请在设置页「AI 助手」中配置".to_string()
        })?;
        let model = s.llm_model.clone().unwrap_or_else(|| default_model(&provider));
        LLMConfig {
            provider,
            api_key: s.llm_api_key.clone(),
            base_url: s.llm_base_url.clone(),
            model,
        }
    };

    let provider = config.provider()?;
    let messages = build_messages(
        &input.action,
        &input.content,
        input.custom_prompt.as_deref(),
    );
    let request = LLMRequest {
        model: config.model.clone(),
        messages,
        temperature: Some(0.7),
        max_tokens: None,
    };

    let task_id = uuid::Uuid::new_v4().to_string();
    let cancel = Arc::new(Notify::new());
    let cancel_for_stream = cancel.clone();
    let registry = app.state::<Arc<AIAbortRegistry>>();
    registry.insert(task_id.clone(), cancel);

    let mut rx = provider.chat_stream(request, cancel_for_stream).await?;

    let task_id_out = task_id.clone();
    tokio::spawn(async move {
        let mut full = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                LLMEvent::Chunk(text) => {
                    full.push_str(&text);
                    let _ = on_event.send(AIStreamEvent::Chunk { text });
                }
                LLMEvent::Done(text) => {
                    if !text.is_empty() {
                        full = text;
                    }
                    let _ = on_event.send(AIStreamEvent::Done { text: full });
                    break;
                }
                LLMEvent::Error(msg) => {
                    let _ = on_event.send(AIStreamEvent::Error { message: msg });
                    break;
                }
            }
        }
    });

    Ok(task_id_out)
}

#[tauri::command]
pub async fn ai_cancel(app: AppHandle, task_id: String) -> Result<(), String> {
    let registry = app.state::<Arc<AIAbortRegistry>>();
    if let Some(notify) = registry.remove(&task_id) {
        notify.notify_waiters();
    }
    Ok(())
}

#[tauri::command]
pub async fn test_llm_connection(db: DbState<'_>) -> Result<String, String> {
    let config = {
        let guard = db.lock().map_err(|e| e.to_string())?;
        let s = guard.load_settings();
        let provider = s.llm_provider.clone().ok_or("Provider 未配置")?;
        let model = s.llm_model.clone().unwrap_or_else(|| default_model(&provider));
        LLMConfig {
            provider,
            api_key: s.llm_api_key.clone(),
            base_url: s.llm_base_url.clone(),
            model,
        }
    };

    let provider = config.provider()?;
    let request = LLMRequest {
        model: config.model.clone(),
        messages: vec![LLMMessage {
            role: "user".to_string(),
            content: "请回复 ok".to_string(),
        }],
        temperature: Some(0.0),
        max_tokens: Some(16),
    };
    let cancel = Arc::new(Notify::new());
    let mut rx = provider.chat_stream(request, cancel).await?;
    let mut text = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            LLMEvent::Chunk(t) => text.push_str(&t),
            LLMEvent::Done(t) => {
                if !t.is_empty() {
                    text = t;
                }
                break;
            }
            LLMEvent::Error(msg) => return Err(msg),
        }
    }
    Ok(text.chars().take(80).collect())
}

fn default_model(provider: &str) -> String {
    match provider {
        "openai" => "gpt-4o-mini".to_string(),
        "anthropic" => "claude-haiku-4-5-20251001".to_string(),
        "ollama" => "qwen2.5:7b".to_string(),
        _ => "unknown".to_string(),
    }
}
