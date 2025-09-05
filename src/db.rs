use super::config;
use super::import;

use log::{debug, error, info};
use rusqlite::Connection;
use std::env;
use std::path::PathBuf;
use tracing::instrument;
use xdg::BaseDirectories;

const DB_VERSION: i32 = 1;

#[instrument(level = "trace")]
fn check_db_version(conn: &Connection) {
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .expect("Failed to read database version");
    if version != DB_VERSION {
        error!("Database version mismatch: expected {DB_VERSION}, found {version}. Please delete your database.");
        std::process::exit(1);
    }
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
fn handle_post_init_checks(conn: &Connection) {
    let fasd_state_path = BaseDirectories::new()
        .find_cache_file("fasd")
        .expect("Cannot find cache file");
    let fasd_state_path_str = fasd_state_path
        .to_str()
        .expect("Cannot convert PathBuf to str");

    if fasd_state_path.exists() {
        import::process_fasd_file(fasd_state_path_str, conn)
            .expect("Failed to process fasd state file");
        info!("Imported {}", &fasd_state_path_str);
    }
}

#[instrument(level = "trace")]
pub fn open_db() -> Result<Connection, String> {
    let db_path = get_db_path();

    if db_path.exists() {
        let db_file = db_path.join("memy.sqlite3");
        let db_path_exists = db_file.exists();
        let conn = Connection::open(&db_file).expect("Failed to open memy database");

        if db_path_exists {
            debug!("Database at {} does exist", db_file.to_string_lossy());
            check_db_version(&conn);
        } else {
            debug!("Database at {} does not exist", db_file.to_string_lossy());
            init_db(&conn);

            if config::get_import_on_first_use() {
                handle_post_init_checks(&conn);
            }
        }

        debug!("Database opened");
        Ok(conn)
    } else {
        Err(format!(
            "Database path {} doesn't exist.",
            db_path.to_string_lossy()
        ))
    }
}
