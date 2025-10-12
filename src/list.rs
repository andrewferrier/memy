use cli::ListArgs;
use colored::Colorize as _;
use config::DeniedFilesOnList;
use core::error::Error;
use is_terminal::IsTerminal as _;
use log::{error, info, warn};
use rusqlite::{params, Connection, OptionalExtension as _};
use std::fs::{metadata, FileType};
use std::io::{stdout, Write as _};
use tracing::instrument;

use crate::cli;
use crate::config;
use crate::db;
use crate::types::Frecency;
use crate::types::NotedCount;
use crate::types::UnixTimestamp;
use crate::types::UnixTimestampHours;
use crate::utils;

#[derive(serde::Serialize)]
struct PathFrecency {
    path: String,
    frecency: Frecency,
    count: NotedCount,
    last_noted: String,
    #[serde(serialize_with = "crate::utils::serialize_file_type")]
    file_type: FileType,
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

fn timestamp_age_hours(now: UnixTimestamp, timestamp: UnixTimestamp) -> UnixTimestampHours {
    let age_seconds = now - timestamp;
    age_seconds as f64 / 3600.0
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
fn calculate(conn: &Connection, args: &ListArgs) -> Result<Vec<PathFrecency>, Box<dyn Error>> {
    let rows = db::get_rows(conn)?;
    let now = utils::get_unix_timestamp();
    let denylist_matcher = config::get_denylist_matcher();

    let mut results: Vec<PathFrecency> = vec![];

    let (oldest_last_noted_timestamp, highest_count) =
        get_oldest_timestamp_and_highest_count(conn, now);
    let oldest_last_noted_timestamp_hours = timestamp_age_hours(now, oldest_last_noted_timestamp);

    for row in rows {
        let Ok(metadata) = metadata(&row.path) else {
            handle_missing_file(conn, &row.path, now, row.last_noted_timestamp);
            continue;
        };

        if let ignore::Match::Ignore(_matched_pat) =
            denylist_matcher.matched_path_or_any_parents(&row.path, metadata.is_dir())
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
            file_type: metadata.file_type(),
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
pub fn command(args: &ListArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open_db()?;
    let results = calculate(&db_connection, args)?;

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
                        if result.file_type.is_dir() {
                            writeln!(stdout_handle, "{}/{}", path_parts[1], path_parts[0].blue())?;
                        } else if result.file_type.is_file() {
                            writeln!(stdout_handle, "{}/{}", path_parts[1], path_parts[0].green())?;
                        }
                    } else if result.file_type.is_dir() {
                        writeln!(stdout_handle, "{}", processed_path.blue())?;
                    } else if result.file_type.is_file() {
                        writeln!(stdout_handle, "{}", processed_path.green())?;
                    }
                } else {
                    writeln!(stdout_handle, "{processed_path}")?;
                }
            }
        }
    }

    Ok(())
}
