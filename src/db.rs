use core::error::Error;
use log::debug;
use rusqlite::{Connection, OptionalExtension as _};
use std::env;
use std::fs;
use std::path::PathBuf;
use tracing::instrument;
use xdg::BaseDirectories;

use crate::config;
use crate::import;
use crate::types::{NotedCount, UnixTimestamp};

const DB_VERSION: i32 = 1;

pub struct PathEntry {
    pub path: String,
    pub noted_count: NotedCount,
    pub last_noted_timestamp: UnixTimestamp,
}

#[instrument(level = "trace")]
fn check_db_version(conn: &Connection) -> Result<(), Box<dyn Error>> {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .expect("Failed to read database version");

    if version != DB_VERSION {
        return Err(
            format!("Database version mismatch: expected {DB_VERSION}, found {version}.").into(),
        );
    }

    Ok(())
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
    conn.execute(&format!("PRAGMA user_version = {DB_VERSION};"), [])
        .expect("Failed to set database version");
}

#[instrument(level = "trace")]
fn handle_post_init_checks(conn: &mut Connection) {
    if let Some(fasd_state_path) = BaseDirectories::new().find_cache_file("fasd")
        && fasd_state_path.exists()
    {
        let fasd_state_path_str = fasd_state_path
            .to_str()
            .expect("Cannot convert PathBuf to str");

        import::process_fasd_file(fasd_state_path_str, conn)
            .expect("Failed to process fasd state file");
    }

    if let Some(autojump_share_path) =
        BaseDirectories::with_prefix("autojump").find_data_file("autojump.txt")
        && autojump_share_path.exists()
    {
        let autojump_share_path_str = autojump_share_path
            .to_str()
            .expect("Cannot convert PathBuf to str");

        import::process_autojump_file(autojump_share_path_str, conn)
            .expect("Failed to process autojump state file");
    }

    import::process_zoxide_query(conn);
}

#[instrument(level = "trace")]
pub fn open() -> Result<Connection, Box<dyn Error>> {
    let db_path = get_db_path();

    if !db_path.exists() {
        fs::create_dir_all(&db_path)?;
    }

    let db_file = db_path.join("memy.sqlite3");
    let db_path_exists = db_file.exists();
    let mut conn = Connection::open(&db_file).expect("Failed to open memy database");

    if db_path_exists {
        debug!("Database at {} does exist", db_file.to_string_lossy());
        check_db_version(&conn)?;
    } else {
        debug!("Database at {} does not exist", db_file.to_string_lossy());
        init_db(&conn);

        if config::get_import_on_first_use() {
            handle_post_init_checks(&mut conn);
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

pub fn get_rows(conn: &Connection) -> Result<Vec<PathEntry>, rusqlite::Error> {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted_timestamp FROM paths")
        .expect("Select failed");

    stmt.query_map([], |row| {
        Ok(PathEntry {
            path: row.get(0)?,
            noted_count: row.get(1)?,
            last_noted_timestamp: row.get(2)?,
        })
    })
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
        check_db_version(&conn).expect("Couldn't check DB version");
        close(conn).expect("Cannot close connection");
    }
}
