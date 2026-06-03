mod core;
mod tools;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        log::info!("Global shortcut triggered: {:?}", shortcut);
                        toggle_main_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("sparkbox.db");
            let db = std::sync::Arc::new(std::sync::Mutex::new(core::db::Database::new(&db_path)?));
            if let Err(e) = db.lock().unwrap().rebuild_fts() {
                log::warn!("FTS rebuild failed: {}", e);
            }
            app.manage(db.clone());
            app.manage(std::sync::Arc::new(tools::ai::AIAbortRegistry::new()));

            setup_tray(app.handle())?;
            setup_global_shortcut(app.handle())?;

            let handle = app.handle().clone();
            let db_for_monitor = db.clone();
            std::thread::spawn(move || {
                core::clipboard::start_monitoring(handle, db_for_monitor);
            });

            let handle2 = app.handle().clone();
            std::thread::spawn(move || {
                core::clipboard::start_cleanup_task(handle2, db);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tools::clipboard::get_history,
            tools::clipboard::delete_entry,
            tools::clipboard::delete_entries,
            tools::clipboard::clear_all,
            tools::clipboard::toggle_favorite,
            tools::clipboard::toggle_pin,
            tools::clipboard::update_entry_meta,
            tools::clipboard::set_favorite_many,
            tools::clipboard::set_pinned_many,
            tools::clipboard::paste_entry,
            tools::clipboard::show_main_window,
            tools::clipboard::hide_main_window,
            tools::clipboard::export_history,
            tools::clipboard::import_history,
            tools::clipboard::cleanup_now,
            tools::clipboard::get_settings,
            tools::clipboard::save_settings,
            tools::ai::ai_transform,
            tools::ai::ai_cancel,
            tools::ai::test_llm_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "tray_show", "显示窗口", true, None::<&str>)?;
    let clear = MenuItem::with_id(app, "tray_clear", "清空记录", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "tray_quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &clear, &quit])?;

    let app_handle = app.clone();
    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().cloned().expect("no window icon"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "tray_show" => {
                show_main_window_cmd(app);
            }
            "tray_clear" => {
                if let Some(db) = app.try_state::<std::sync::Arc<std::sync::Mutex<core::db::Database>>>() {
                    if let Ok(guard) = db.lock() {
                        let _ = guard.clear_all();
                    }
                }
                let _ = app.emit("clipboard-update", ());
            }
            "tray_quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window_cmd(tray.app_handle());
            }
            let _ = app_handle;
        })
        .build(app)?;
    Ok(())
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    app.global_shortcut().register(shortcut)?;
    log::info!("Registered global shortcut: Cmd+Shift+V");
    Ok(())
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                show_main_window_cmd(app);
            }
        }
    }
}

fn show_main_window_cmd(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = window.emit("sparkbox-window-shown", ());
    }
}
