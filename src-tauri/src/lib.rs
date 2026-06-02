mod core;
mod tools;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("toolbox.db");
            let db = std::sync::Arc::new(std::sync::Mutex::new(core::db::Database::new(&db_path)?));
            app.manage(db.clone());

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                core::clipboard::start_monitoring(handle, db);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tools::clipboard::get_history,
            tools::clipboard::delete_entry,
            tools::clipboard::toggle_favorite,
            tools::clipboard::toggle_pin,
            tools::clipboard::paste_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
