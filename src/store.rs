//! SQLite-backed storage for Lwow (history, bookmarks, settings, sessions).

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
}

impl Store {
    /// Opens (or creates) the database with WAL mode for fast, safe writes.
    pub fn open(data_dir: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(data_dir.join("lwow.db"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let store = Self { conn: Mutex::new(conn) };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS history (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                url       TEXT NOT NULL,
                title     TEXT,
                visited_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS bookmarks (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id INTEGER REFERENCES bookmarks(id),
                title     TEXT NOT NULL,
                url       TEXT,
                folder    INTEGER NOT NULL DEFAULT 0,
                position  INTEGER NOT NULL DEFAULT 0
            );",
        )
    }

    fn get(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
            .ok()
    }

    fn set(&self, key: &str, value: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [key, value],
        );
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)
    }

    pub fn set_string(&self, key: &str, value: &str) {
        self.set(key, value);
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).map(|v| v == "1" || v == "true")
    }

    pub fn set_bool(&self, key: &str, value: bool) {
        self.set(key, if value { "1" } else { "0" });
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.parse().ok())
    }

    pub fn set_f64(&self, key: &str, value: f64) {
        self.set(key, &value.to_string());
    }
}
