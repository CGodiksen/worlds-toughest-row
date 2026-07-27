//! SQLite-backed implementation of the [`Storage`] trait, persisting rowing entries in a local
//! database file.

use std::sync::Mutex;

use api_types::Entry;
use route::Storage;
use rusqlite::Connection;

/// SQL that creates the `entries` table if it does not already exist.
const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meters INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);";

/// A [`Storage`] backed by a single SQLite connection.
pub struct SqliteStorage {
    /// The database connection, guarded by a mutex for shared access.
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    /// Open (or create) the database at `path` and ensure the schema exists.
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl Storage for SqliteStorage {
    /// Insert a new entry and return it as stored (with id, meters, and timestamp).
    fn add_entry(&self, meters: i32) -> anyhow::Result<Entry> {
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

    /// Return all entries ordered newest-first.
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

    /// Return the summed meters across all entries (0 when the table is empty).
    fn total_meters(&self) -> anyhow::Result<i32> {
        let conn = self.conn.lock().unwrap();

        Ok(
            conn.query_row("SELECT COALESCE(SUM(meters), 0) FROM entries", [], |r| {
                r.get(0)
            })?,
        )
    }

    /// Delete every entry from the table.
    fn reset(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute("DELETE FROM entries", [])?;

        Ok(())
    }
}
