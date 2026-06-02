use arboard::Clipboard;
use log::error;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::core::db::Database;

static RUNNING: AtomicBool = AtomicBool::new(true);

pub fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
}

pub fn start_monitoring(app: AppHandle, db: Arc<std::sync::Mutex<Database>>) {
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to initialize clipboard: {}", e);
            return;
        }
    };

    let mut last_change: Option<u64> = None;

    while RUNNING.load(Ordering::SeqCst) {
        if detect_change(&mut last_change) {
            if let Ok(Some((type_, text, image_data, file_list))) = read_clipboard(&mut clipboard)
            {
                let hash = compute_hash(
                    &type_,
                    text.as_deref(),
                    image_data.as_deref(),
                    file_list.as_deref(),
                );
                if let Ok(guard) = db.lock() {
                    if !guard.has_entry(&hash).unwrap_or(false) {
                        if let Err(e) = guard.insert_entry(
                            &type_,
                            text.as_deref(),
                            image_data.as_deref(),
                            file_list.as_deref(),
                            &hash,
                        ) {
                            error!("Failed to save clipboard: {}", e);
                        } else {
                            let _ = app.emit("clipboard-update", ());
                        }
                    } else {
                        let _ = guard.touch_entry(&hash);
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn detect_change(last: &mut Option<u64>) -> bool {
    #[cfg(target_os = "macos")]
    {
        let count = macos_change_count();
        let prev = last.replace(count);
        prev.map_or(true, |p| p != count)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = last;
        true
    }
}

#[cfg(target_os = "macos")]
fn macos_change_count() -> u64 {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let cls = Class::get("NSPasteboard").unwrap();
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        msg_send![pb, changeCount]
    }
}

fn read_clipboard(
    clipboard: &mut Clipboard,
) -> Result<Option<(String, Option<String>, Option<Vec<u8>>, Option<String>)>, arboard::Error> {
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            return Ok(Some(("text".into(), Some(text), None, None)));
        }
    }
    if let Ok(img) = clipboard.get_image() {
        if let Some(png) = encode_png(&img) {
            return Ok(Some(("image".into(), None, Some(png), None)));
        }
    }
    Ok(None)
}

fn encode_png(img: &arboard::ImageData) -> Option<Vec<u8>> {
    let w = img.width as u32;
    let h = img.height as u32;
    let rgba: Vec<u8> = img.bytes.iter().copied().collect();
    let image_buffer: image::RgbaImage = image::ImageBuffer::from_raw(w, h, rgba)?;
    let mut png_buf = Vec::new();
    image::DynamicImage::ImageRgba8(image_buffer)
        .write_to(&mut std::io::Cursor::new(&mut png_buf), image::ImageFormat::Png)
        .ok()?;
    Some(png_buf)
}

fn compute_hash(type_: &str, text: Option<&str>, image: Option<&[u8]>, files: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    match type_ {
        "text" => hasher.update(text.unwrap_or("").as_bytes()),
        "image" => hasher.update(image.unwrap_or(b"")),
        "files" => hasher.update(files.unwrap_or("").as_bytes()),
        _ => hasher.update(b"unknown"),
    }
    hex::encode(hasher.finalize())
}
