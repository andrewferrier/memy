use core::error::Error;
use rusqlite::{Connection, OptionalExtension as _};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::debug;
use tracing::instrument;
use xdg::BaseDirectories;

use super::config;
use super::types::{NotedCount, UnixTimestamp};
use crate::import;

const DB_VERSION: i32 = 2;
const DB_FILENAME: &str = "memy.sqlite3";

#[derive(serde::Serialize)]
pub struct TablePathsEntry {
    pub path: String,
    pub noted_count: NotedCount,
    pub last_noted_timestamp: UnixTimestamp,
}

pub trait FromRow: Sized {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self>;
}

impl FromRow for TablePathsEntry {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            path: row.get("path")?,
            noted_count: row.get("noted_count")?,
            last_noted_timestamp: row.get("last_noted_timestamp")?,
        })
    }
}

#[instrument(level = "trace")]
fn get_db_version(conn: &Connection) -> i32 {
    conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
        .expect("Failed to read database version")
}

#[instrument(level = "trace")]
fn get_db_path() -> PathBuf {
    env::var("MEMY_DB_DIR").map_or_else(
        |_| {
            let xdg_dirs = BaseDirectories::with_prefix("memy");
            xdg_dirs.get_state_home().expect("Cannot get XDG home")
        },
        PathBuf::from,
    )
}

fn create_state_table(conn: &Connection, breaking_change_sort_warning_count_remaining: i32) {
    conn.execute(
        "CREATE TABLE state (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )
    .expect("Failed to create state table");

    conn.execute(
        &format!("INSERT INTO state (key, value) VALUES ('breaking_change_sort_warning_count_remaining', '{breaking_change_sort_warning_count_remaining}')"),
        []
    ).expect("Cannot insert into state table");
}

#[instrument(level = "trace")]
fn init_db(conn: &Connection) {
    conn.execute(
        "CREATE TABLE paths (
            path TEXT PRIMARY KEY,
            noted_count INTEGER NOT NULL,
            last_noted_timestamp INTEGER NOT NULL
        )",
        [],
    )
    .expect("Failed to initialize database");

    create_state_table(conn, 0);

    conn.execute(&format!("PRAGMA user_version = {DB_VERSION};"), [])
        .expect("Failed to set database version");
}

#[instrument(level = "trace")]
fn migrate_v1_to_v2(conn: &Connection) {
    debug!("Migrating database from version 1 to version 2");

    create_state_table(conn, 10);

    conn.execute("PRAGMA user_version = 2;", [])
        .expect("Failed to set database version to 2");

    debug!("Migration from v1 to v2 complete");
}

fn get_warning_count_left(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT value FROM state WHERE key = 'breaking_change_sort_warning_count_remaining'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<i64>().ok())
    .unwrap_or(0)
}

fn decrement_warning_count(conn: &Connection) {
    conn.execute(
        "UPDATE state SET value = CAST(CAST(value AS INTEGER) - 1 AS TEXT) \
         WHERE key = 'breaking_change_sort_warning_count_remaining' AND CAST(value AS INTEGER) > 0",
        [],
    )
    .expect("Failed to decrement warning count");
}

pub fn should_show_breaking_change_sort_warning() -> bool {
    let db_file = get_db_path().join(DB_FILENAME);
    if !db_file.exists() {
        return false;
    }

    let Ok(conn) = open() else {
        return false;
    };

    let should_show = get_warning_count_left(&conn) > 0;
    if should_show {
        decrement_warning_count(&conn);
    }

    let _: core::result::Result<_, _> = close(conn);
    should_show
}

#[instrument(level = "trace")]
pub fn open() -> Result<Connection, Box<dyn Error>> {
    let db_path = get_db_path();

    if !db_path.exists() {
        fs::create_dir_all(&db_path)?;
    }

    let db_file = db_path.join(DB_FILENAME);
    let db_path_exists = db_file.exists();
    let mut conn = Connection::open(&db_file).expect("Failed to open memy database");

    if db_path_exists {
        debug!("Database at {} does exist", db_file.to_string_lossy());
        let version = get_db_version(&conn);

        if version == 1 {
            migrate_v1_to_v2(&conn);
        } else if !(1..=2).contains(&version) {
            return Err(format!(
                "Database version mismatch: expected {DB_VERSION}, found {version}."
            )
            .into());
        }
    } else {
        debug!("Database at {} does not exist", db_file.to_string_lossy());
        init_db(&conn);

        if config::get_import_on_first_use() {
            import::run_importers(&mut conn);
        }
    }

    debug!("Database opened");
    Ok(conn)
}

#[instrument(level = "trace")]
pub fn close(conn: Connection) -> Result<(), Box<dyn Error>> {
    conn.execute("PRAGMA optimize;", []).optional()?;
    conn.close().map_err(|(_, err)| err.into())
}

pub fn get_rows(conn: &Connection) -> Result<Vec<TablePathsEntry>, rusqlite::Error> {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted_timestamp FROM paths")
        .expect("Select failed");

    stmt.query_map([], TablePathsEntry::from_row)
        .expect("Query mapping failed")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_check_db() {
        let conn = Connection::open_in_memory().expect("Could not open connection");
        init_db(&conn);
        assert_eq!(get_db_version(&conn), DB_VERSION, "DB Version incorrect");
        close(conn).expect("Cannot close connection");
    }
}
