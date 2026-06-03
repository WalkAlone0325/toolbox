use rusqlite::{params, Result};

#[derive(Debug, Clone, Default)]
pub struct AppSettings {
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

pub fn ensure_table(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    Ok(())
}

pub fn get(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

pub fn set(conn: &rusqlite::Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn load_settings(conn: &rusqlite::Connection) -> AppSettings {
    let auto_hide_on_blur = get(conn, "auto_hide_on_blur")
        .map(|v| v == "1")
        .unwrap_or(false);
    let max_age_days = get(conn, "max_age_days").and_then(|v| v.parse().ok());
    let max_count = get(conn, "max_count").and_then(|v| v.parse().ok());
    let max_bytes = get(conn, "max_bytes").and_then(|v| v.parse().ok());
    let cleanup_interval_secs = get(conn, "cleanup_interval_secs")
        .and_then(|v| v.parse().ok())
        .unwrap_or(3600);
    let ignore_apps = get(conn, "ignore_apps")
        .map(|v| {
            v.split('\n')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let llm_provider = get(conn, "llm_provider").filter(|s| !s.is_empty());
    let llm_api_key = get(conn, "llm_api_key").filter(|s| !s.is_empty());
    let llm_base_url = get(conn, "llm_base_url").filter(|s| !s.is_empty());
    let llm_model = get(conn, "llm_model").filter(|s| !s.is_empty());

    AppSettings {
        auto_hide_on_blur,
        max_age_days,
        max_count,
        max_bytes,
        ignore_apps,
        cleanup_interval_secs,
        llm_provider,
        llm_api_key,
        llm_base_url,
        llm_model,
    }
}

pub fn save_settings(conn: &rusqlite::Connection, settings: &AppSettings) -> Result<()> {
    set(conn, "auto_hide_on_blur", if settings.auto_hide_on_blur { "1" } else { "0" })?;
    set(
        conn,
        "max_age_days",
        &settings.max_age_days.map(|v| v.to_string()).unwrap_or_default(),
    )?;
    set(
        conn,
        "max_count",
        &settings.max_count.map(|v| v.to_string()).unwrap_or_default(),
    )?;
    set(
        conn,
        "max_bytes",
        &settings.max_bytes.map(|v| v.to_string()).unwrap_or_default(),
    )?;
    set(conn, "cleanup_interval_secs", &settings.cleanup_interval_secs.to_string())?;
    set(conn, "ignore_apps", &settings.ignore_apps.join("\n"))?;
    set(conn, "llm_provider", settings.llm_provider.as_deref().unwrap_or(""))?;
    set(conn, "llm_api_key", settings.llm_api_key.as_deref().unwrap_or(""))?;
    set(conn, "llm_base_url", settings.llm_base_url.as_deref().unwrap_or(""))?;
    set(conn, "llm_model", settings.llm_model.as_deref().unwrap_or(""))?;
    Ok(())
}
