use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub hash: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub text_val: Option<String>,
    pub image_path: Option<String>,
    pub thumb_path: Option<String>,
    pub file_list: Option<String>,
    pub source_app: Option<String>,
    pub byte_size: i64,
    pub fav: bool,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: i64,
    pub use_count: i64,
    pub title: Option<String>,
    pub tags: Option<String>,
    pub summary: Option<String>,
    pub note: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
    app_dir: std::path::PathBuf,
}

const ENTRY_COLUMNS_PLAIN: &str = "id, hash, type, text_val, image_path, thumb_path,
        file_list, source_app, byte_size, fav, pinned, created_at, updated_at,
        last_used_at, use_count, title, tags, summary, note";

fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<ClipboardEntry> {
    Ok(ClipboardEntry {
        id: row.get(0)?,
        hash: row.get(1)?,
        entry_type: row.get(2)?,
        text_val: row.get(3)?,
        image_path: row.get(4)?,
        thumb_path: row.get(5)?,
        file_list: row.get(6)?,
        source_app: row.get(7)?,
        byte_size: row.get(8)?,
        fav: row.get::<_, i32>(9)? != 0,
        pinned: row.get::<_, i32>(10)? != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        last_used_at: row.get(13)?,
        use_count: row.get(14)?,
        title: row.get(15)?,
        tags: row.get(16)?,
        summary: row.get(17)?,
        note: row.get(18)?,
    })
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                hash        TEXT NOT NULL UNIQUE,
                type        TEXT NOT NULL DEFAULT 'text',
                text_val    TEXT,
                image_path  TEXT,
                thumb_path  TEXT,
                file_list   TEXT,
                source_app  TEXT,
                byte_size   INTEGER DEFAULT 0,
                fav         INTEGER DEFAULT 0,
                pinned      INTEGER DEFAULT 0,
                created_at  INTEGER NOT NULL,
                updated_at  INTEGER NOT NULL,
                last_used_at INTEGER DEFAULT 0,
                use_count   INTEGER DEFAULT 0,
                title       TEXT,
                tags        TEXT,
                summary     TEXT,
                note        TEXT
            )",
            [],
        )?;

        Self::migrate(&conn)?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_created_at ON entries(created_at DESC)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_last_used_at ON entries(last_used_at DESC)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_type ON entries(type)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_fav ON entries(fav)", [])?;

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(text_val, content=entries, content_rowid=id, tokenize='unicode61 remove_diacritics 2')",
            [],
        )?;

        conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entries_fts(rowid, text_val) VALUES (new.id, new.text_val);
             END;
             CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, text_val) VALUES('delete', old.id, old.text_val);
             END;
             CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
                INSERT INTO entries_fts(entries_fts, rowid, text_val) VALUES('delete', old.id, old.text_val);
                INSERT INTO entries_fts(rowid, text_val) VALUES (new.id, new.text_val);
             END;"
        )?;

        let app_dir = path.parent().unwrap().to_path_buf();
        if std::fs::create_dir_all(app_dir.join("images")).is_err()
            || std::fs::create_dir_all(app_dir.join("thumbs")).is_err()
        {
            return Err(rusqlite::Error::InvalidQuery);
        }

        crate::core::settings::ensure_table(&conn)?;

        Ok(Database {
            conn: Mutex::new(conn),
            app_dir,
        })
    }

    pub fn load_settings(&self) -> crate::core::settings::AppSettings {
        let conn = self.conn.lock().unwrap();
        crate::core::settings::load_settings(&conn)
    }

    pub fn save_settings(&self, settings: &crate::core::settings::AppSettings) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        crate::core::settings::save_settings(&conn, settings)
    }

    fn migrate(conn: &Connection) -> Result<()> {
        let cols = Self::existing_columns(conn)?;
        let need = [
            ("last_used_at", "INTEGER DEFAULT 0"),
            ("use_count", "INTEGER DEFAULT 0"),
            ("title", "TEXT"),
            ("tags", "TEXT"),
            ("summary", "TEXT"),
            ("note", "TEXT"),
        ];
        for (col, decl) in need {
            if !cols.iter().any(|c| c == col) {
                conn.execute(&format!("ALTER TABLE entries ADD COLUMN {} {}", col, decl), [])?;
                log::info!("Migrated: added column {}", col);
            }
        }

        let fts_token: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='entries_fts'",
                [],
                |r| r.get(0),
            )
            .ok();
        let need_rebuild_fts = match fts_token {
            Some(sql) => !sql.contains("unicode61"),
            None => false,
        };
        if need_rebuild_fts {
            conn.execute("DROP TABLE IF EXISTS entries_fts", [])?;
            conn.execute(
                "CREATE VIRTUAL TABLE entries_fts USING fts5(text_val, content=entries, content_rowid=id, tokenize='unicode61 remove_diacritics 2')",
                [],
            )?;
            conn.execute("INSERT INTO entries_fts(entries_fts) VALUES('rebuild')", [])?;
            log::info!("FTS5 rebuilt with unicode61 tokenizer");
        }

        let idx_check: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_index_list('entries') WHERE name='idx_last_used_at'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if idx_check == 0 {
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_last_used_at ON entries(last_used_at DESC)",
                [],
            )?;
        }
        Ok(())
    }

    fn existing_columns(conn: &Connection) -> Result<Vec<String>> {
        let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn rebuild_fts(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO entries_fts(entries_fts) VALUES('rebuild')", [])?;
        Ok(())
    }

    pub fn has_entry(&self, hash: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE hash = ?1",
            params![hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn touch_entry(&self, hash: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET updated_at = ?1, last_used_at = ?1, use_count = use_count + 1 WHERE hash = ?2",
            params![now, hash],
        )?;
        Ok(())
    }

    pub fn insert_entry(
        &self,
        type_: &str,
        text: Option<&str>,
        image_data: Option<&[u8]>,
        file_list: Option<&str>,
        hash: &str,
        source_app: Option<&str>,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let (image_path, thumb_path, size) = match image_data {
            Some(data) if !data.is_empty() => {
                let ip = self.save_image(hash, data);
                let tp = self.save_thumbnail(hash, data);
                (ip, tp, data.len() as i64)
            }
            _ => {
                let size = text.map(|t| t.len() as i64).unwrap_or(0);
                (None, None, size)
            }
        };

        let truncated_text = text.map(|t| Self::truncate_text(t));

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO entries (hash, type, text_val, image_path, thumb_path, file_list, source_app, byte_size, created_at, updated_at, last_used_at, use_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, ?9, 1)",
            params![hash, type_, truncated_text, image_path, thumb_path, file_list, source_app, size, now],
        )?;

        Ok(())
    }

    fn save_image(&self, hash: &str, data: &[u8]) -> Option<String> {
        if data.len() > 50 * 1024 * 1024 {
            log::warn!("Image too large ({} bytes), skipping", data.len());
            return None;
        }
        let path = self.app_dir.join("images").join(format!("{}.png", hash));
        std::fs::write(&path, data).ok()?;
        Some(path.to_string_lossy().to_string())
    }

    fn save_thumbnail(&self, hash: &str, data: &[u8]) -> Option<String> {
        if data.len() > 50 * 1024 * 1024 {
            return None;
        }
        let img = image::load_from_memory(data).ok()?;
        const MAX_THUMB: u32 = 200;
        let thumb = img.resize(MAX_THUMB, MAX_THUMB, image::imageops::FilterType::Lanczos3);
        let path = self.app_dir.join("thumbs").join(format!("{}.png", hash));
        let mut buf = std::io::Cursor::new(Vec::new());
        thumb.write_to(&mut buf, image::ImageFormat::Png).ok()?;
        std::fs::write(&path, buf.into_inner()).ok()?;
        Some(path.to_string_lossy().to_string())
    }

    pub fn get_history(
        &self,
        query: &str,
        filter_type: Option<&str>,
    ) -> Result<Vec<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();

        let entries: Vec<ClipboardEntry> = if !query.is_empty() {
            let fts_query = Self::build_fts_query(query);
            let sql = if filter_type.is_some() {
                format!(
                    "SELECT {cols} FROM entries
                     WHERE id IN (SELECT rowid FROM entries_fts WHERE entries_fts MATCH ?1)
                       AND type = ?2
                     ORDER BY pinned DESC, last_used_at DESC LIMIT 500",
                    cols = ENTRY_COLUMNS_PLAIN
                )
            } else {
                format!(
                    "SELECT {cols} FROM entries
                     WHERE id IN (SELECT rowid FROM entries_fts WHERE entries_fts MATCH ?1)
                     ORDER BY pinned DESC, last_used_at DESC LIMIT 500",
                    cols = ENTRY_COLUMNS_PLAIN
                )
            };
            let mut stmt = conn.prepare(&sql)?;
            if let Some(ft) = filter_type {
                stmt.query_map(params![&fts_query, ft], row_to_entry)?
                    .collect::<Result<Vec<_>>>()?
            } else {
                stmt.query_map(params![&fts_query], row_to_entry)?
                    .collect::<Result<Vec<_>>>()?
            }
        } else if let Some(ft) = filter_type {
            let sql = format!(
                "SELECT {cols} FROM entries WHERE type = ?1
                 ORDER BY pinned DESC, last_used_at DESC LIMIT 500",
                cols = ENTRY_COLUMNS_PLAIN
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows: Vec<ClipboardEntry> = stmt
                .query_map(params![ft], row_to_entry)?
                .collect::<Result<Vec<_>>>()?;
            rows
        } else {
            let sql = format!(
                "SELECT {cols} FROM entries
                 ORDER BY pinned DESC, last_used_at DESC LIMIT 500",
                cols = ENTRY_COLUMNS_PLAIN
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows: Vec<ClipboardEntry> = stmt
                .query_map([], row_to_entry)?
                .collect::<Result<Vec<_>>>()?;
            rows
        };

        Ok(entries)
    }

    fn truncate_text(t: &str) -> String {
        const MAX_TEXT_BYTES: usize = 1024 * 1024;
        if t.len() <= MAX_TEXT_BYTES {
            return t.to_string();
        }
        let mut cut = MAX_TEXT_BYTES;
        while cut > 0 && !t.is_char_boundary(cut) {
            cut -= 1;
        }
        t[..cut].to_string()
    }

    fn build_fts_query(input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return "\"\"".to_string();
        }
        let needs_phrase = trimmed.chars().any(|c| c.is_whitespace());
        if needs_phrase {
            format!("\"{}\"", trimmed.replace('"', "\"\""))
        } else {
            let escaped = trimmed.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        }
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute("DELETE FROM entries", [])?;
        Ok(n)
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET fav = CASE WHEN fav = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn toggle_pin(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn update_entry_meta(
        &self,
        id: i64,
        tags: Option<&str>,
        note: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET tags = ?1, note = ?2 WHERE id = ?3",
            params![tags, note, id],
        )?;
        Ok(())
    }

    pub fn get_entry(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT {cols} FROM entries WHERE id = ?1",
            cols = ENTRY_COLUMNS_PLAIN
        );
        let mut stmt = conn.prepare(&sql)?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_entry(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_entries(&self, ids: &[i64]) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().unwrap();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM entries WHERE id IN ({})", placeholders);
        let params: Vec<&dyn rusqlite::ToSql> = ids
            .iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        let n = conn.execute(&sql, params.as_slice())?;
        Ok(n)
    }

    pub fn set_favorite_many(&self, ids: &[i64], fav: bool) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().unwrap();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("UPDATE entries SET fav = ? WHERE id IN ({})", placeholders);
        let fav_val = fav as i32;
        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&fav_val];
        for id in ids {
            params.push(id);
        }
        let n = conn.execute(&sql, rusqlite::params_from_iter(params))?;
        Ok(n)
    }

    pub fn set_pinned_many(&self, ids: &[i64], pinned: bool) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let conn = self.conn.lock().unwrap();
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("UPDATE entries SET pinned = ? WHERE id IN ({})", placeholders);
        let pinned_val = pinned as i32;
        let mut params: Vec<&dyn rusqlite::ToSql> = vec![&pinned_val];
        for id in ids {
            params.push(id);
        }
        let n = conn.execute(&sql, rusqlite::params_from_iter(params))?;
        Ok(n)
    }

    pub fn export_all(&self) -> Result<Vec<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "SELECT {cols} FROM entries ORDER BY pinned DESC, last_used_at DESC",
            cols = ENTRY_COLUMNS_PLAIN
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], row_to_entry)?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn import_entry(
        &self,
        type_: &str,
        text: Option<&str>,
        image_data: Option<&[u8]>,
        file_list: Option<&str>,
        hash: &str,
        source_app: Option<&str>,
        created_at: i64,
        last_used_at: i64,
        fav: bool,
        pinned: bool,
    ) -> Result<()> {
        let (image_path, thumb_path, size) = match image_data {
            Some(data) if !data.is_empty() => {
                let ip = self.save_image(hash, data);
                let tp = self.save_thumbnail(hash, data);
                (ip, tp, data.len() as i64)
            }
            _ => {
                let size = text.map(|t| t.len() as i64).unwrap_or(0);
                (None, None, size)
            }
        };

        let truncated_text = text.map(|t| Self::truncate_text(t));

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO entries (hash, type, text_val, image_path, thumb_path, file_list, source_app, byte_size, created_at, updated_at, last_used_at, use_count, fav, pinned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, ?10, 1, ?11, ?12)",
            params![hash, type_, truncated_text, image_path, thumb_path, file_list, source_app, size, created_at, last_used_at, fav as i32, pinned as i32],
        )?;
        Ok(())
    }

    pub fn cleanup_old(&self, max_age_days: Option<i64>, max_count: Option<i64>, max_bytes: Option<i64>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut deleted = 0usize;

        if let Some(days) = max_age_days {
            if days > 0 {
                let cutoff = chrono::Utc::now().timestamp() - days * 86400;
                let n = conn.execute(
                    "DELETE FROM entries WHERE fav = 0 AND pinned = 0 AND last_used_at < ?1",
                    params![cutoff],
                )?;
                deleted += n;
            }
        }

        if let Some(max) = max_count {
            if max > 0 {
                let n = conn.execute(
                    "DELETE FROM entries WHERE id NOT IN (
                        SELECT id FROM entries WHERE fav = 1 OR pinned = 1
                        UNION
                        SELECT id FROM entries ORDER BY last_used_at DESC LIMIT ?1
                    )",
                    params![max],
                )?;
                deleted += n;
            }
        }

        if let Some(max) = max_bytes {
            if max > 0 {
                let total: i64 = conn.query_row(
                    "SELECT COALESCE(SUM(byte_size), 0) FROM entries WHERE fav = 0 AND pinned = 0",
                    [],
                    |r| r.get(0),
                ).unwrap_or(0);
                if total > max {
                    let n = conn.execute(
                        "DELETE FROM entries WHERE fav = 0 AND pinned = 0 AND id IN (
                            SELECT id FROM entries WHERE fav = 0 AND pinned = 0
                            ORDER BY last_used_at ASC LIMIT 500
                        )",
                        [],
                    )?;
                    deleted += n;
                }
            }
        }

        Ok(deleted)
    }
}
