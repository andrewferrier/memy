use chrono::Utc;
use clap::Parser;
use rusqlite::{params, Connection};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "memy")]
#[command(version = "0.1")]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
struct Args {
    /// Note usage of one or more paths
    #[arg(short, long, value_name = "PATHS", conflicts_with = "list")]
    note: Option<Vec<String>>,

    /// List paths by frecency score (default action)
    #[arg(short, long, conflicts_with = "note")]
    list: bool,

    /// Controls recency weighting (higher means favor recency more)
    #[arg(long, default_value = "3600.0")]
    recency_bias: f64,

    /// Disable symlink normalization when noting paths
    #[arg(long)]
    no_normalize_symlinks: bool,
}

fn get_db_path() -> PathBuf {
    let cache_dir = dirs::cache_dir().expect("Cannot determine cache directory");
    let memy_dir = cache_dir.join("memy");
    if !memy_dir.exists() {
        fs::create_dir_all(&memy_dir).expect("Failed to create memy cache directory");
    }
    memy_dir.join("memy.sqlite3")
}

fn init_db(conn: &Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS paths (
            path TEXT PRIMARY KEY,
            noted_count INTEGER NOT NULL,
            last_noted TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to initialize database");
}

fn normalize_path(path: &str) -> Option<String> {
    let p = Path::new(path);
    fs::canonicalize(p)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
        .or_else(|| Some(p.to_string_lossy().into_owned()))
}

fn note_path(conn: &Connection, raw_path: &str, normalize: bool) {
    if !Path::new(raw_path).exists() {
        eprintln!("Path {} does not exist.", raw_path);
        std::process::exit(1);
    }

    let path = if normalize {
        normalize_path(raw_path)
    } else {
        Some(Path::new(raw_path).to_string_lossy().into_owned())
    };

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO paths (path, noted_count, last_noted) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted = excluded.last_noted",
        params![path, now],
    )
    .unwrap();
}

fn list_paths(conn: &Connection, recent_bias: f64) {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted FROM paths")
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let last_noted: String = row.get(2)?;
            Ok((path, count, last_noted))
        })
        .unwrap();

    let now = Utc::now();

    let mut results: Vec<(String, f64)> = vec![];

    for row in rows {
        if let Ok((path, count, last_noted)) = row {
            if !Path::new(&path).exists() {
                conn.execute("DELETE FROM paths WHERE path = ?", params![path])
                    .unwrap();
                continue;
            }

            if let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(&last_noted) {
                let age_secs = now
                    .signed_duration_since(last_dt.with_timezone(&Utc))
                    .num_seconds() as f64;
                let frecency = count as f64 * (1.0 / (1.0 + age_secs / recent_bias));
                results.push((path, frecency));
            }
        }
    }

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (path, score) in results {
        println!("{}\t{}", path, score);
    }
}

fn main() {
    let args = Args::parse();

    let db_path = get_db_path();
    let conn = Connection::open(db_path).expect("Failed to open memy database");
    init_db(&conn);

    let normalize = !args.no_normalize_symlinks;

    if let Some(paths) = args.note {
        for path in paths {
            note_path(&conn, &path, normalize);
        }
    } else {
        list_paths(&conn, args.recency_bias);
    }
}
