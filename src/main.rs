mod cli;
mod config;
mod db;
mod hooks;
mod hooks_generated;
mod utils;

use atty::Stream;
use clap::CommandFactory;
use clap::Parser;
use cli::{Cli, Commands, ListArgs};
use config::DeniedFilesOnList;
use home::home_dir;
use log::{debug, error, info, warn};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::instrument;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter};

fn configure_logging_and_tracing(cli: &Cli) {
    LogTracer::init().expect("Failed to init LogTracer");

    let default_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // RUST_LOG overrides --verbose if set
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_ansi(atty::is(Stream::Stderr))
        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT)
        .event_format(fmt::format().compact())
        .with_writer(std::io::stderr)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
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

#[instrument(level = "trace")]
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

#[instrument(level = "trace")]
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

#[derive(serde::Serialize)]
struct PathFrecency {
    path: String,
    frecency: f64,
    count: u64,
    last_noted: String,
}

#[instrument(level = "trace")]
fn list_paths_calculate(conn: &Connection, args: &ListArgs) -> Vec<PathFrecency> {
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

    let mut results: Vec<PathFrecency> = vec![];

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
            warn!("Path {path} no longer exists, deleted from database.");
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
        results.push(PathFrecency {
            path,
            frecency,
            count,
            last_noted: utils::timestamp_to_iso8601(last_noted_timestamp),
        });
    }

    results.sort_by(|a, b| {
        a.frecency
            .partial_cmp(&b.frecency)
            .expect("Sort results failed")
    });

    results
}

#[instrument(level = "trace")]
fn list_paths(conn: &Connection, args: &ListArgs) {
    let results = list_paths_calculate(conn, args);

    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&results)
                .expect("Failed to serialize results to JSON");
            println!("{json_output}");
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            for result in results {
                wtr.serialize(result).expect("Cannot serialize CSV");
            }
            wtr.flush().expect("Cannot flush CSV");
        }
        _ => {
            for result in results {
                println!("{}", result.path);
            }
        }
    }
}

#[instrument(level = "trace")]
fn completions(shell: Option<clap_complete::Shell>) {
    let actual_shell = shell
        .or_else(utils::detect_shell)
        .expect("Could not determine shell. Specify one explicitly.");
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(actual_shell, &mut cmd, bin_name, &mut std::io::stdout());
}

#[instrument(level = "trace")]
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

    configure_logging_and_tracing(&cli);

    debug!("Memy version {}", env!("GIT_VERSION"));
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
            if let Err(err) = config::generate_config(filename.as_deref()) {
                error!("{err}");
                std::process::exit(1);
            }
        }
        Commands::Completions { shell } => {
            completions(shell);
        }
        Commands::Hook { hook_name } => {
            hook_show(hook_name);
        }
    }
}
