use chrono::{DateTime, Utc};
use clap::CommandFactory;
use clap::{Args, Parser, Subcommand};
use env_logger::{Builder, Env};
use log::{debug, error, info, warn, LevelFilter};
use rusqlite::{params, Connection};
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use xdg::BaseDirectories;

mod config;
mod utils;

#[derive(Parser)]
#[command(name = "memy")]
#[command(version = env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Enable verbose (info) logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug (very detailed) logging
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Note usage of one or more paths
    Note(NoteArgs),
    /// List paths by frecency score
    List(ListArgs),
    /// Generate a template memy.toml config file
    GenerateConfig {
        /// Optional output filename for the generated config
        #[arg(value_name = "FILENAME")]
        filename: Option<String>,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for (e.g. bash, zsh)
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,
    },
}

#[derive(Args)]
struct NoteArgs {
    /// One or more paths to note
    #[arg(value_name = "PATHS")]
    paths: Vec<String>,
}

#[derive(Args)]
struct ListArgs {
    /// Show only files in the list
    #[arg(short, long)]
    files_only: bool,

    /// Show only directories in the list
    #[arg(short, long)]
    directories_only: bool,

    /// Include frecency score in output
    #[arg(long)]
    include_frecency_score: bool,
}

const RECENCY_BIAS: f64 = 3600.0;
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

fn normalize_path_if_needed(path: &Path, normalize: bool) -> String {
    if normalize {
        std::fs::canonicalize(path).ok().map_or_else(
            || path.to_string_lossy().into_owned(),
            |p| p.to_string_lossy().into_owned(),
        )
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn note_path(conn: &Connection, raw_path: &str) {
    let path = Path::new(raw_path);

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }

        return;
    }

    let config_normalize = config::get_normalize_symlinks_on_note();
    let clean_path = normalize_path_if_needed(path, config_normalize);

    let matcher = config::get_denylist_matcher();
    if let ignore::Match::Ignore(_matched_pat) = matcher.matched(&clean_path, false) {
        if config::get_denied_files_warn_on_note() {
            warn!("Path denied by denylist pattern.");
        }
        return;
    }

    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![clean_path, now],
    )
    .unwrap();
    info!("Path {raw_path} noted");
}

fn list_paths(conn: &Connection, args: &ListArgs) {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted_timestamp FROM paths")
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let last_noted_timestamp: i64 = row.get(2)?;
            Ok((path, count, last_noted_timestamp))
        })
        .unwrap();

    let now = Utc::now();

    let mut results: Vec<(String, f64)> = vec![];

    for (path, count, last_noted_timestamp) in rows.into_iter().flatten() {
        if !Path::new(&path).exists() {
            conn.execute("DELETE FROM paths WHERE path = ?", params![path])
                .unwrap();
            info!("Path {path} no longer exists, deleted from database.");
            continue;
        }

        let metadata = fs::metadata(&path).unwrap();

        if args.files_only && !metadata.is_file() {
            continue;
        }

        if args.directories_only && !metadata.is_dir() {
            continue;
        }

        let last_dt = DateTime::from_timestamp(last_noted_timestamp, 0).expect("invalid timestamp");
        let age_secs = now.signed_duration_since(last_dt).num_seconds() as f64;
        let frecency = count as f64 * (1.0 / (1.0 + age_secs / RECENCY_BIAS));
        results.push((path, frecency));
    }

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (path, score) in results {
        if args.include_frecency_score {
            println!("{path}\t{score}");
        } else {
            println!("{path}");
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let level = match &cli.command {
        Commands::GenerateConfig { .. } => {
            if cli.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        }
        _ => {
            if cli.debug {
                LevelFilter::Debug
            } else if cli.verbose {
                LevelFilter::Info
            } else {
                LevelFilter::Warn
            }
        }
    };

    Builder::from_env(Env::default().default_filter_or(level.to_string()))
        .target(env_logger::Target::Stderr)
        .init();

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

    match cli.command {
        Commands::Note(note_args) => {
            for path in note_args.paths {
                note_path(&conn, &path);
            }
        }
        Commands::List(list_args) => {
            list_paths(&conn, &list_args);
        }
        Commands::GenerateConfig { filename } => {
            config::generate_config(filename.as_deref());
        }
        Commands::Completions { shell } => {
            let shell = shell
                .or_else(utils::detect_shell)
                .expect("Could not determine shell. Specify one explicitly.");
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}
