use log::{debug, error};
use rusqlite::Connection;
use std::env;
use std::path::PathBuf;
use xdg::BaseDirectories;

const DB_VERSION: i32 = 1;

fn check_db_version(conn: &Connection) {
    debug!("Checking database version...");
    let version: i32 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .expect("Failed to read database version");
    if version != DB_VERSION {
        error!("Database version mismatch: expected {DB_VERSION}, found {version}. Please delete your database.");
        std::process::exit(1);
    }
}

fn get_db_path() -> PathBuf {
    env::var("MEMY_DB_DIR").map_or_else(
        |_| {
            let xdg_dirs = BaseDirectories::with_prefix("memy");
            xdg_dirs
                .place_state_file("memy.sqlite3")
                .expect("Cannot determine state file path")
        },
        |env_path| PathBuf::from(env_path).join("memy.sqlite3"),
    )
}

fn init_db(conn: &Connection) {
    debug!("Initializing database...");
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
    debug!("Database initialized");
}

pub fn open_db() -> Connection {
    let db_path = get_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    let db_path_exists = db_path.exists();
    let conn = Connection::open(&db_path).expect("Failed to open memy database");

    if db_path_exists {
        debug!("Database at {db_path_str} does exist");
        check_db_version(&conn);
    } else {
        debug!("Database at {db_path_str} does not exist");
        init_db(&conn);
    }

    debug!("Database opened");
    conn
}
