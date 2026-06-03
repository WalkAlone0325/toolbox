use crate::core::db::{self, Database};
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

type DbState<'a> = State<'a, Arc<Mutex<Database>>>;

#[tauri::command]
pub fn get_history(
    db: DbState<'_>,
    query: Option<String>,
    filter_type: Option<String>,
) -> Result<Vec<db::ClipboardEntry>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_history(&query.unwrap_or_default(), filter_type.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_entry(db: DbState<'_>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_all(db: DbState<'_>) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.clear_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_favorite(db: DbState<'_>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.toggle_favorite(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_pin(db: DbState<'_>, id: i64) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.toggle_pin(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_entry_meta(
    db: DbState<'_>,
    id: i64,
    tags: Option<String>,
    note: Option<String>,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.update_entry_meta(id, tags.as_deref(), note.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn paste_entry(db: DbState<'_>, id: i64) -> Result<(), String> {
    let entry = {
        let db = db.lock().map_err(|e| e.to_string())?;
        db.get_entry(id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Entry not found".to_string())?
    };

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

    match entry.entry_type.as_str() {
        "text" => {
            if let Some(text) = &entry.text_val {
                clipboard.set_text(text).map_err(|e| e.to_string())?;
            }
        }
        "image" => {
            if let Some(path) = &entry.image_path {
                let data = std::fs::read(path).map_err(|e| e.to_string())?;
                let img = image::load_from_memory(&data)
                    .map_err(|e| e.to_string())?
                    .to_rgba8();
                let (w, h) = img.dimensions();
                let img_data = arboard::ImageData {
                    width: w as usize,
                    height: h as usize,
                    bytes: std::borrow::Cow::Owned(img.into_raw()),
                };
                clipboard.set_image(img_data).map_err(|e| e.to_string())?;
            }
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.unminimize().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        window.emit("sparkbox-window-shown", ()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_entries(db: DbState<'_>, ids: Vec<i64>) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.delete_entries(&ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_favorite_many(db: DbState<'_>, ids: Vec<i64>, fav: bool) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_favorite_many(&ids, fav).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_pinned_many(db: DbState<'_>, ids: Vec<i64>, pinned: bool) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.set_pinned_many(&ids, pinned).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_history(db: DbState<'_>) -> Result<Vec<db::ClipboardEntry>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.export_all().map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct ImportEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub text_val: Option<String>,
    pub image_path: Option<String>,
    pub file_list: Option<String>,
    pub source_app: Option<String>,
    pub created_at: Option<i64>,
    pub last_used_at: Option<i64>,
    pub fav: Option<bool>,
    pub pinned: Option<bool>,
}

#[tauri::command]
pub fn import_history(db: DbState<'_>, entries: Vec<ImportEntry>) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for e in entries {
        let text_for_hash = e.text_val.as_deref();
        let file_for_hash = e.file_list.as_deref();
        let hash = crate::core::clipboard::compute_hash_public(
            &e.entry_type,
            text_for_hash,
            file_for_hash,
        );
        let created_at = e.created_at.unwrap_or_else(|| chrono::Utc::now().timestamp());
        let last_used_at = e.last_used_at.unwrap_or(created_at);
        let fav = e.fav.unwrap_or(false);
        let pinned = e.pinned.unwrap_or(false);

        let image_data: Option<Vec<u8>> = match &e.image_path {
            Some(p) if !p.is_empty() => std::fs::read(p).ok(),
            _ => None,
        };

        db.import_entry(
            &e.entry_type,
            text_for_hash,
            image_data.as_deref(),
            file_for_hash,
            &hash,
            e.source_app.as_deref(),
            created_at,
            last_used_at,
            fav,
            pinned,
        )
        .map_err(|err| err.to_string())?;
        count += 1;
    }
    Ok(count)
}

#[tauri::command]
pub fn cleanup_now(
    db: DbState<'_>,
    max_age_days: Option<i64>,
    max_count: Option<i64>,
    max_bytes: Option<i64>,
) -> Result<usize, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.cleanup_old(max_age_days, max_count, max_bytes)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct AppSettingsOut {
    pub auto_hide_on_blur: bool,
    pub max_age_days: Option<i64>,
    pub max_count: Option<i64>,
    pub max_bytes: Option<i64>,
    pub ignore_apps: Vec<String>,
    pub cleanup_interval_secs: i64,
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_base_url: Option<String>,
    pub llm_model: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct AppSettingsIn {
    pub auto_hide_on_blur: Option<bool>,
    pub max_age_days: Option<Option<i64>>,
    pub max_count: Option<Option<i64>>,
    pub max_bytes: Option<Option<i64>>,
    pub ignore_apps: Option<Vec<String>>,
    pub cleanup_interval_secs: Option<i64>,
    pub llm_provider: Option<Option<String>>,
    pub llm_api_key: Option<Option<String>>,
    pub llm_base_url: Option<Option<String>>,
    pub llm_model: Option<Option<String>>,
}

#[tauri::command]
pub fn get_settings(db: DbState<'_>) -> Result<AppSettingsOut, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let s = db.load_settings();
    Ok(AppSettingsOut {
        auto_hide_on_blur: s.auto_hide_on_blur,
        max_age_days: s.max_age_days,
        max_count: s.max_count,
        max_bytes: s.max_bytes,
        ignore_apps: s.ignore_apps,
        cleanup_interval_secs: s.cleanup_interval_secs,
        llm_provider: s.llm_provider,
        llm_api_key: s.llm_api_key,
        llm_base_url: s.llm_base_url,
        llm_model: s.llm_model,
    })
}

#[tauri::command]
pub fn save_settings(db: DbState<'_>, settings: AppSettingsIn) -> Result<(), String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let mut current = db.load_settings();
    if let Some(v) = settings.auto_hide_on_blur {
        current.auto_hide_on_blur = v;
    }
    if let Some(v) = settings.max_age_days {
        current.max_age_days = v;
    }
    if let Some(v) = settings.max_count {
        current.max_count = v;
    }
    if let Some(v) = settings.max_bytes {
        current.max_bytes = v;
    }
    if let Some(v) = settings.ignore_apps {
        current.ignore_apps = v;
    }
    if let Some(v) = settings.cleanup_interval_secs {
        current.cleanup_interval_secs = v;
    }
    if let Some(v) = settings.llm_provider {
        current.llm_provider = v.filter(|s| !s.is_empty());
    }
    if let Some(v) = settings.llm_api_key {
        current.llm_api_key = v.filter(|s| !s.is_empty());
    }
    if let Some(v) = settings.llm_base_url {
        current.llm_base_url = v.filter(|s| !s.is_empty());
    }
    if let Some(v) = settings.llm_model {
        current.llm_model = v.filter(|s| !s.is_empty());
    }
    db.save_settings(&current).map_err(|e| e.to_string())
}
