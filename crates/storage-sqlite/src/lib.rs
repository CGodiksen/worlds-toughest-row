use std::sync::Mutex;

use api_types::Entry;
use core::Storage;
use rusqlite::Connection;

const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meters INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);";

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl Storage for SqliteStorage {
    fn add_entry(&self, meters: i64) -> anyhow::Result<Entry> {
        let conn = self.conn.lock().unwrap();

        Ok(conn.query_row(
            "INSERT INTO entries (meters) VALUES (?1)
             RETURNING id, meters, created_at",
            [meters],
            |r| {
                Ok(Entry {
                    id: r.get(0)?,
                    meters: r.get(1)?,
                    created_at: r.get(2)?,
                })
            },
        )?)
    }

    fn list_entries(&self) -> anyhow::Result<Vec<Entry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, meters, created_at FROM entries ORDER BY id DESC")?;

        let rows = stmt.query_map([], |r| {
            Ok(Entry {
                id: r.get(0)?,
                meters: r.get(1)?,
                created_at: r.get(2)?,
            })
        })?;

        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn total_meters(&self) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();

        Ok(
            conn.query_row("SELECT COALESCE(SUM(meters), 0) FROM entries", [], |r| {
                r.get(0)
            })?,
        )
    }
}
