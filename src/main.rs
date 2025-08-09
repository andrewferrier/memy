use clap::CommandFactory;
use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, error, info, warn, LevelFilter};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

mod config;
use config::DeniedFilesOnList;
mod cli;
mod db;
mod hooks;
mod hooks_generated;
use home::home_dir;
mod utils;
use crate::cli::{Cli, Commands, ListArgs};

fn set_logging_level(cli: &Cli) {
    let level;

    if cli.debug {
        level = LevelFilter::Debug;
    } else if cli.verbose {
        level = LevelFilter::Info;
    } else {
        level = LevelFilter::Warn;
    }

    Builder::from_env(Env::default().default_filter_or(level.to_string()))
        .target(env_logger::Target::Stderr)
        .init();
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    } else if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(path)
}

fn normalize_path_if_needed(path: &Path) -> String {
    let normalize = config::get_normalize_symlinks_on_note();

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
    let pathbuf = expand_tilde(raw_path);
    let path: &Path = pathbuf.as_path();

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }

        return;
    }

    let clean_path = normalize_path_if_needed(path);

    let matcher = config::get_denylist_matcher();
    if let ignore::Match::Ignore(_matched_pat) = matcher.matched(&clean_path, false) {
        if config::get_denied_files_warn_on_note() {
            warn!("Path denied by denylist pattern.");
        }
        return;
    }

    let now = utils::get_secs_since_epoch();
    conn.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![clean_path, now],
    )
    .expect("Insert failed");
    info!("Path {clean_path} noted");
}

fn timestamp_to_age_hours(now: u64, timestamp: u64) -> f64 {
    let age_seconds = now - timestamp;
    age_seconds as f64 / 3600.0
}

fn calculate_frecency(
    count: u64,
    last_noted_timestamp_hours: f64,
    highest_count: u64,
    oldest_last_noted_timestamp_hours: f64,
) -> f64 {
    let freq_score = if highest_count > 0 {
        count as f64 / highest_count as f64
    } else {
        0.0
    };

    let recency_score = if last_noted_timestamp_hours < oldest_last_noted_timestamp_hours {
        1.0 - (last_noted_timestamp_hours / oldest_last_noted_timestamp_hours)
    } else {
        0.0
    };

    let lambda = config::get_recency_bias();
    (1.0 - lambda).mul_add(freq_score, lambda * recency_score)
}

fn get_oldest_timestamp_and_highest_count(conn: &Connection, now: u64) -> (u64, u64) {
    let oldest_last_noted_timestamp: u64 = conn
        .query_row(
            "SELECT last_noted_timestamp FROM paths ORDER BY last_noted_timestamp ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .expect("Cannot get oldest timestamp")
        .unwrap_or(now);

    let highest_count: u64 = conn
        .query_row(
            "SELECT noted_count FROM paths ORDER BY noted_count DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .expect("Cannot get highest count")
        .unwrap_or(0);

    (oldest_last_noted_timestamp, highest_count)
}

fn list_paths(conn: &Connection, args: &ListArgs) {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted_timestamp FROM paths")
        .expect("Select failed");

    let rows = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let count: u64 = row.get(1)?;
            let last_noted_timestamp: u64 = row.get(2)?;
            Ok((path, count, last_noted_timestamp))
        })
        .expect("query_map failed");

    let now = utils::get_secs_since_epoch();

    let mut results: Vec<(String, f64)> = vec![];

    let list_denied_action = config::get_denied_files_on_list();
    let matcher = config::get_denylist_matcher();

    let (oldest_last_noted_timestamp, highest_count) =
        get_oldest_timestamp_and_highest_count(conn, now);
    let oldest_last_noted_timestamp_hours =
        timestamp_to_age_hours(now, oldest_last_noted_timestamp);

    for (path, count, last_noted_timestamp) in rows.into_iter().flatten() {
        if let ignore::Match::Ignore(_matched_pat) = matcher.matched(&path, false) {
            match list_denied_action {
                DeniedFilesOnList::SkipSilently => {
                    continue;
                }
                DeniedFilesOnList::Warn => {
                    warn!("Path {path} is denied, remaining in database.");
                    continue;
                }
                DeniedFilesOnList::Delete => {
                    conn.execute("DELETE FROM paths WHERE path = ?", params![path])
                        .expect("Delete failed");
                    info!("Path {path} is denied, deleted from database.");
                    continue;
                }
            }
        }

        let Ok(metadata) = fs::metadata(&path) else {
            conn.execute("DELETE FROM paths WHERE path = ?", params![path])
                .expect("Delete failed");
            info!("Path {path} no longer exists, deleted from database.");
            continue;
        };

        if args.files_only && !metadata.is_file() {
            continue;
        }

        if args.directories_only && !metadata.is_dir() {
            continue;
        }

        let frecency = calculate_frecency(
            count,
            timestamp_to_age_hours(now, last_noted_timestamp),
            highest_count,
            oldest_last_noted_timestamp_hours,
        );
        results.push((path, frecency));
    }

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("Sort results failed"));
    for (path, _) in results {
        println!("{path}");
    }
}

fn completions(shell: Option<clap_complete::Shell>) {
    let actual_shell = shell
        .or_else(utils::detect_shell)
        .expect("Could not determine shell. Specify one explicitly.");
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(actual_shell, &mut cmd, bin_name, &mut std::io::stdout());
}

fn hook_show(hook_name: Option<String>) {
    if let Some(actual_hook_name) = hook_name {
        if let Some(content) = hooks::get_hook_content(&actual_hook_name) {
            print!("{content}");
        } else {
            eprintln!("Hook not found: {actual_hook_name}");
            std::process::exit(1);
        }
    } else {
        println!("Available hooks:");
        for hook in hooks::get_hook_list() {
            println!("{hook}");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    config::set_config_overrides(cli.config.clone());

    set_logging_level(&cli);

    debug!("CLI params parsed: {cli:?}");

    match cli.command {
        Commands::Note(note_args) => {
            let db_connection = db::open_db();

            if note_args.paths.is_empty() {
                error!("You must specify some paths to note");
                std::process::exit(1);
            }
            for path in note_args.paths {
                note_path(&db_connection, &path);
            }
        }
        Commands::List(list_args) => {
            let db_connection = db::open_db();
            list_paths(&db_connection, &list_args);
        }
        Commands::GenerateConfig { filename } => {
            config::generate_config(filename.as_deref());
        }
        Commands::Completions { shell } => {
            completions(shell);
        }
        Commands::Hook { hook_name } => {
            hook_show(hook_name);
        }
    }
}
