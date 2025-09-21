mod cli;
mod config;
mod db;
mod denylist_default;
mod hooks;
mod hooks_generated;
mod import;
mod logging;
mod types;
mod utils;

use clap::CommandFactory as _;
use clap::Parser as _;
use cli::{Cli, Commands, ListArgs};
use config::DeniedFilesOnList;
use core::error::Error;
use is_terminal::IsTerminal as _;
use log::{debug, error, info, warn};
use rusqlite::{params, Connection, OptionalExtension as _, Transaction};
use std::borrow::Cow;
use std::fs;
use std::io::{self, stdout, Write as _};
use std::path::Path;
use tracing::instrument;

use types::Frecency;
use types::NotedCount;
use types::UnixTimestamp;
use types::UnixTimestampHours;

fn normalize_path_if_needed(path: &Path) -> std::io::Result<Cow<'_, Path>> {
    let normalize = config::get_normalize_symlinks_on_note();

    if normalize {
        Ok(Cow::Owned(fs::canonicalize(path)?))
    } else {
        Ok(Cow::Borrowed(path))
    }
}

fn note_path(tx: &Transaction, raw_path: &str) -> Result<(), Box<(dyn Error + 'static)>> {
    let pathbuf = utils::expand_tilde(raw_path);
    let path = pathbuf.as_path();

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }

        return Ok(());
    }

    let clean_path = normalize_path_if_needed(path)?;

    let matcher = config::get_denylist_matcher();
    if let ignore::Match::Ignore(_matched_pat) =
        matcher.matched_path_or_any_parents(&clean_path, false)
    {
        if config::get_denied_files_warn_on_note() {
            warn!("Path {} denied by denylist pattern.", clean_path.display());
        }

        return Ok(());
    }

    let now = utils::get_unix_timestamp();
    tx.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![clean_path.to_string_lossy(), now],
    )
    .expect("Insert failed");

    info!("Path {} noted", clean_path.display());
    Ok(())
}

#[instrument(level = "trace")]
fn note_paths(note_args: cli::NoteArgs) -> Result<(), Box<dyn Error>> {
    if note_args.paths.is_empty() {
        return Err("You must specify some paths to note".into());
    }

    let mut db_connection = db::open_db().expect("Could not open memy database");
    let tx = db_connection
        .transaction()
        .expect("Cannot start DB transaction");

    for path in note_args.paths {
        note_path(&tx, &path)?;
    }

    tx.commit().expect("Cannot commit transaction");

    Ok(())
}

fn timestamp_age_hours(now: UnixTimestamp, timestamp: UnixTimestamp) -> UnixTimestampHours {
    let age_seconds = now - timestamp;
    age_seconds as f64 / 3600.0
}

fn calculate_frecency(
    count: NotedCount,
    last_noted_timestamp_hours: UnixTimestampHours,
    highest_count: NotedCount,
    oldest_last_noted_timestamp_hours: UnixTimestampHours,
) -> Frecency {
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
fn get_oldest_timestamp_and_highest_count(
    conn: &Connection,
    now: UnixTimestamp,
) -> (UnixTimestamp, NotedCount) {
    let oldest_last_noted_timestamp: UnixTimestamp = conn
        .query_row(
            "SELECT last_noted_timestamp FROM paths ORDER BY last_noted_timestamp ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .expect("Cannot get oldest timestamp")
        .unwrap_or(now);

    let highest_count: NotedCount = conn
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
    frecency: Frecency,
    count: NotedCount,
    last_noted: String,
}

fn should_use_color(color: &String) -> bool {
    match color.as_str() {
        "always" => true,
        "never" => false,
        "automatic" => stdout().is_terminal(),
        _ => {
            error!("Invalid value for color: {color}");
            std::process::exit(1);
        }
    }
}

fn handle_denied_file(conn: &Connection, path: &String) {
    let list_denied_action = config::get_denied_files_on_list();

    match list_denied_action {
        DeniedFilesOnList::Warn => {
            warn!("Path {} is denied, remaining in database.", &path);
        }
        DeniedFilesOnList::Delete => {
            conn.execute("DELETE FROM paths WHERE path = ?", params![&path])
                .expect("Delete failed");
            info!("Path {} is denied, deleted from database.", &path);
        }
        DeniedFilesOnList::SkipSilently => {}
    }
}

fn handle_missing_file(
    conn: &Connection,
    path: &String,
    now: UnixTimestamp,
    last_noted_timestamp: UnixTimestamp,
) {
    let missing_files_delete_after_days = config::get_missing_files_delete_from_db_after();

    let last_noted_age_days = (now - last_noted_timestamp) / 86_400; // Convert seconds to days

    if last_noted_age_days > missing_files_delete_after_days {
        conn.execute("DELETE FROM paths WHERE path = ?", params![path])
            .expect("Delete failed");
        warn!("{path} no longer exists; last noted {last_noted_age_days} days ago; older than get_missing_files_delete_from_db_after, removed from database.");
    } else {
        info!("{path} no longer exists; last noted {last_noted_age_days} days ago; within get_missing_files_delete_from_db_after, retained but skipped.");
    }
}

#[instrument(level = "trace")]
fn list_paths_calculate(
    conn: &Connection,
    args: &ListArgs,
) -> Result<Vec<PathFrecency>, Box<dyn Error>> {
    let rows = db::get_rows(conn)?;
    let now = utils::get_unix_timestamp();

    let mut results: Vec<PathFrecency> = vec![];

    let (oldest_last_noted_timestamp, highest_count) =
        get_oldest_timestamp_and_highest_count(conn, now);
    let oldest_last_noted_timestamp_hours = timestamp_age_hours(now, oldest_last_noted_timestamp);

    for row in rows {
        let Ok(metadata) = fs::metadata(&row.path) else {
            handle_missing_file(conn, &row.path, now, row.last_noted_timestamp);
            continue;
        };

        if let ignore::Match::Ignore(_matched_pat) =
            config::get_denylist_matcher().matched_path_or_any_parents(&row.path, metadata.is_dir())
        {
            handle_denied_file(conn, &row.path);
            continue;
        }

        if (args.files_only && !metadata.is_file()) || (args.directories_only && !metadata.is_dir())
        {
            continue;
        }

        let frecency = calculate_frecency(
            row.noted_count,
            timestamp_age_hours(now, row.last_noted_timestamp),
            highest_count,
            oldest_last_noted_timestamp_hours,
        );

        results.push(PathFrecency {
            path: row.path,
            frecency,
            count: row.noted_count,
            last_noted: utils::timestamp_to_iso8601(row.last_noted_timestamp),
        });
    }

    results.sort_unstable_by(|a, b| {
        a.frecency
            .partial_cmp(&b.frecency)
            .expect("Sort results failed")
    });

    Ok(results)
}

#[instrument(level = "trace")]
fn list_paths(args: &ListArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open_db()?;
    let results = list_paths_calculate(&db_connection, args)?;

    let mut stdout_handle = stdout().lock();

    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&results)
                .expect("Failed to serialize results to JSON");
            writeln!(stdout_handle, "{json_output}")?;
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(stdout());
            for result in results {
                wtr.serialize(result)?;
            }
            wtr.flush()?;
        }
        _ => {
            let use_color = should_use_color(&args.color);
            for result in results {
                let processed_path = if config::get_use_tilde_on_list() {
                    utils::collapse_to_tilde(result.path)
                } else {
                    result.path
                };

                if use_color {
                    let path_parts: Vec<&str> = processed_path.rsplitn(2, '/').collect();
                    if path_parts.len() == 2 {
                        writeln!(
                            stdout_handle,
                            "{}/\x1b[34m{}\x1b[0m",
                            path_parts[1], path_parts[0]
                        )?;
                    } else {
                        writeln!(stdout_handle, "\x1b[34m{processed_path}\x1b[0m")?;
                    }
                } else {
                    writeln!(stdout_handle, "{processed_path}")?;
                }
            }
        }
    }

    Ok(())
}

#[instrument(level = "trace")]
fn completions(shell: Option<clap_complete::Shell>) {
    let actual_shell = shell
        .or_else(utils::detect_shell)
        .expect("Could not determine shell. Specify one explicitly.");
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_owned();
    clap_complete::generate(actual_shell, &mut cmd, bin_name, &mut stdout());
}

#[instrument(level = "trace")]
fn hook_show(
    hook_name: Option<String>,
) -> core::result::Result<(), std::boxed::Box<(dyn Error + 'static)>> {
    let mut stdout_handle = io::stdout().lock();

    if let Some(actual_hook_name) = hook_name {
        if let Some(content) = hooks::get_hook_content(&actual_hook_name) {
            write!(stdout_handle, "{content}")?;
        } else {
            return Err(format!("Hook not found: {actual_hook_name}").into());
        }
    } else {
        writeln!(stdout_handle, "Available hooks:")?;
        for hook in hooks::get_hook_list() {
            writeln!(stdout_handle, "{hook}")?;
        }
    }

    Ok(())
}

fn handle_cli_command(
    command: Commands,
) -> core::result::Result<(), std::boxed::Box<(dyn Error + 'static)>> {
    match command {
        Commands::Note(note_args) => Ok(note_paths(note_args)?),
        Commands::List(list_args) => Ok(list_paths(&list_args)?),
        Commands::GenerateConfig {} => Ok(config::output_template_config()?),
        Commands::Completions { shell } => {
            completions(shell);
            Ok(())
        }
        Commands::Hook { hook_name } => Ok(hook_show(hook_name)?),
    }
}

fn main() {
    let cli = Cli::parse();

    config::set_config_overrides(cli.config.clone());
    logging::configure_logging_and_tracing(cli.verbose);

    debug!("Memy version {}", env!("GIT_VERSION"));
    debug!("CLI params parsed: {cli:?}");

    match handle_cli_command(cli.command) {
        Ok(()) => {}
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    }
}
