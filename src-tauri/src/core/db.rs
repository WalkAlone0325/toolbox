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
}

pub struct Database {
    conn: Mutex<Connection>,
    app_dir: std::path::PathBuf,
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
                updated_at  INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute("CREATE INDEX IF NOT EXISTS idx_created_at ON entries(created_at DESC)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_type ON entries(type)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_fav ON entries(fav)", [])?;

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(text_val, content=entries, content_rowid=id)",
            [],
        )?;

        let app_dir = path.parent().unwrap().to_path_buf();
        if std::fs::create_dir_all(app_dir.join("images")).is_err()
            || std::fs::create_dir_all(app_dir.join("thumbs")).is_err()
        {
            return Err(rusqlite::Error::InvalidQuery);
        }

        Ok(Database {
            conn: Mutex::new(conn),
            app_dir,
        })
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
            "UPDATE entries SET updated_at = ?1 WHERE hash = ?2",
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

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO entries (hash, type, text_val, image_path, thumb_path, file_list, byte_size, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![hash, type_, text, image_path, thumb_path, file_list, size, now],
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
        let path = self.app_dir.join("thumbs").join(format!("{}.png", hash));
        std::fs::write(&path, data).ok()?;
        Some(path.to_string_lossy().to_string())
    }

    pub fn get_history(
        &self,
        query: &str,
        filter_type: Option<&str>,
    ) -> Result<Vec<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT id, hash, type, text_val, image_path, thumb_path, file_list,
                    source_app, byte_size, fav, pinned, created_at, updated_at
             FROM entries",
        );

        if !query.is_empty() {
            let safe = query.replace('\'', "''");
            sql = format!(
                "SELECT e.id, e.hash, e.type, e.text_val, e.image_path, e.thumb_path,
                        e.file_list, e.source_app, e.byte_size, e.fav, e.pinned,
                        e.created_at, e.updated_at
                 FROM entries e
                 INNER JOIN entries_fts fts ON e.id = fts.rowid
                 WHERE entries_fts MATCH '{}'",
                safe
            );
        } else if let Some(ft) = filter_type {
            sql.push_str(&format!(" WHERE type = '{}'", ft));
        }

        sql.push_str(" ORDER BY pinned DESC, created_at DESC LIMIT 500");

        let mut stmt = conn.prepare(&sql)?;
        let entries = stmt
            .query_map([], |row| {
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
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
        Ok(())
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

    pub fn get_entry(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, hash, type, text_val, image_path, thumb_path, file_list,
                    source_app, byte_size, fav, pinned, created_at, updated_at
             FROM entries WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(ClipboardEntry {
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
            }))
        } else {
            Ok(None)
        }
    }
}
