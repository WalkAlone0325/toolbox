use crate::core::db::{self, Database};
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use tauri::State;

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
