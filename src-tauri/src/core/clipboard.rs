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
                let source_app = current_source_app();
                let ignore = {
                    if let Some(name) = &source_app {
                        if let Ok(guard) = db.lock() {
                            let s = guard.load_settings();
                            s.ignore_apps.iter().any(|x| x == name)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
                if ignore {
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }

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
                            source_app.as_deref(),
                        ) {
                            error!("Failed to save clipboard: {}", e);
                        } else {
                            let _ = app.emit("clipboard-update", ());
                        }
                    } else {
                        let _ = guard.touch_entry(&hash);
                        let _ = app.emit("clipboard-update", ());
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

pub fn start_cleanup_task(app: AppHandle, db: Arc<std::sync::Mutex<Database>>) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(60));
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            let settings = {
                let Ok(guard) = db.lock() else { continue };
                guard.load_settings()
            };
            let interval = settings.cleanup_interval_secs.max(300);
            let max_age = settings.max_age_days.filter(|&v| v > 0);
            let max_count = settings.max_count.filter(|&v| v > 0);
            let max_bytes = settings.max_bytes.filter(|&v| v > 0);
            if max_age.is_none() && max_count.is_none() && max_bytes.is_none() {
                continue;
            }
            if let Ok(guard) = db.lock() {
                if let Err(e) = guard.cleanup_old(max_age, max_count, max_bytes) {
                    log::warn!("Auto cleanup failed: {}", e);
                } else {
                    let _ = app.emit("clipboard-update", ());
                }
            }
            let _ = interval;
        }
    });
}

fn current_source_app() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        macos_frontmost_app()
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn macos_frontmost_app() -> Option<String> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let ws_cls = Class::get("NSWorkspace")?;
        let ws: *mut Object = msg_send![ws_cls, sharedWorkspace];
        let app: *mut Object = msg_send![ws, frontmostApplication];
        if app.is_null() {
            return None;
        }
        let name: *mut Object = msg_send![app, localizedName];
        if name.is_null() {
            return None;
        }
        let c_str: *const i8 = msg_send![name, UTF8String];
        if c_str.is_null() {
            return None;
        }
        let s = std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned();
        Some(s)
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
    #[cfg(target_os = "macos")]
    {
        if let Some(files) = macos_read_files() {
            if !files.is_empty() {
                let json = serde_json::to_string(&files).unwrap_or_default();
                return Ok(Some(("files".into(), None, None, Some(json))));
            }
        }
    }

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

#[cfg(target_os = "macos")]
fn macos_read_files() -> Option<Vec<String>> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let cls = Class::get("NSPasteboard")?;
        let pb: *mut Object = msg_send![cls, generalPasteboard];

        let ns_string_cls = Class::get("NSString")?;
        let key_filenames: *mut Object =
            msg_send![ns_string_cls, stringWithUTF8String: "NSFilenamesPboardType"];

        let types: *mut Object = msg_send![pb, types];
        let count: usize = msg_send![types, count];
        let mut has_filenames = false;
        let mut has_file_url = false;
        for i in 0..count {
            let t: *mut Object = msg_send![types, objectAtIndex: i];
            let c_str: *const i8 = msg_send![t, UTF8String];
            if c_str.is_null() {
                continue;
            }
            let s = std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned();
            if s == "NSFilenamesPboardType" {
                has_filenames = true;
            } else if s == "public.file-url" {
                has_file_url = true;
            }
        }

        if has_filenames {
            let plist: *mut Object = msg_send![pb, propertyListForType: key_filenames];
            if !plist.is_null() {
                let is_array: bool = msg_send![plist, isKindOfClass: Class::get("NSArray")?];
                if is_array {
                    let n: usize = msg_send![plist, count];
                    let mut paths = Vec::with_capacity(n);
                    for i in 0..n {
                        let p: *mut Object = msg_send![plist, objectAtIndex: i];
                        let c_str: *const i8 = msg_send![p, UTF8String];
                        if !c_str.is_null() {
                            let s = std::ffi::CStr::from_ptr(c_str)
                                .to_string_lossy()
                                .into_owned();
                            paths.push(s);
                        }
                    }
                    if !paths.is_empty() {
                        return Some(paths);
                    }
                }
            }
        }

        if has_file_url {
            let ns_array_cls = Class::get("NSArray")?;
            let url_cls = Class::get("NSURL")?;
            let classes: *mut Object = msg_send![ns_array_cls, arrayWithObject: url_cls];
            let objects: *mut Object = msg_send![pb, readObjectsForClasses:classes options: std::ptr::null::<Object>()];
            if !objects.is_null() {
                let n: usize = msg_send![objects, count];
                let mut paths = Vec::with_capacity(n);
                for i in 0..n {
                    let url: *mut Object = msg_send![objects, objectAtIndex: i];
                    let path: *mut Object = msg_send![url, path];
                    if !path.is_null() {
                        let c_str: *const i8 = msg_send![path, UTF8String];
                        if !c_str.is_null() {
                            let s = std::ffi::CStr::from_ptr(c_str)
                                .to_string_lossy()
                                .into_owned();
                            paths.push(s);
                        }
                    }
                }
                if !paths.is_empty() {
                    return Some(paths);
                }
            }
        }

        None
    }
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

pub fn compute_hash_public(type_: &str, text: Option<&str>, files: Option<&str>) -> String {
    compute_hash(type_, text, None, files)
}
