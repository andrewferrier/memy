use cli::ListArgs;
use colored::Colorize as _;
use config::DeniedFilesOnList;
use core::error::Error;
use log::{info, warn};
use rusqlite::{Connection, params};
use std::fs::{FileType, metadata};
use std::io::{Write as _, stdout};
use std::sync::LazyLock;
use tracing::instrument;

use crate::cli;
use crate::config;
use crate::db;
use crate::frecency;
use crate::stats;
use crate::types::Frecency;
use crate::types::NotedCount;
use crate::types::UnixTimestamp;
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
    static MISSING_FILES_DELETE_AFTER_SECS: LazyLock<i64> =
        LazyLock::new(|| i64::from(config::get_missing_files_delete_from_db_after()) * 86400);

    if *MISSING_FILES_DELETE_AFTER_SECS < 0 {
        info!(
            "{path} no longer exists but get_missing_files_delete_from_db_after < 0, so it will not be deleted."
        );
    } else {
        let last_noted_age_secs = now - last_noted_timestamp;

        if last_noted_age_secs > *MISSING_FILES_DELETE_AFTER_SECS {
            conn.execute("DELETE FROM paths WHERE path = ?", params![path])
                .expect("Delete failed");
            info!(
                "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; older than get_missing_files_delete_from_db_after, removed from database."
            );
        } else {
            info!(
                "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; within get_missing_files_delete_from_db_after, retained but skipped."
            );
        }
    }
}

#[instrument(level = "trace")]
fn calculate(conn: &Connection, args: &ListArgs) -> Result<Vec<PathFrecency>, Box<dyn Error>> {
    let rows = db::get_rows(conn)?;
    let now = utils::get_unix_timestamp();
    let denylist_matcher = config::get_denylist_matcher();

    // Parse the newer_than filter if provided
    let newer_than_timestamp = if let Some(ref newer_than_str) = args.newer_than {
        Some(utils::parse_newer_than(newer_than_str)?)
    } else {
        None
    };

    let mut results: Vec<PathFrecency> = vec![];
    let stats = stats::get(conn)?;

    let Some(oldest_note) = stats.oldest_note else {
        return Ok(results);
    };
    let oldest_last_noted_timestamp_hours = utils::timestamp_age_hours(now, oldest_note.timestamp);

    let Some(highest_count) = stats.highest_count else {
        return Ok(results);
    };

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

        if let Some(cutoff_timestamp) = newer_than_timestamp
            && row.last_noted_timestamp < cutoff_timestamp
        {
            continue;
        }

        let frecency = frecency::calculate(
            row.noted_count,
            utils::timestamp_age_hours(now, row.last_noted_timestamp),
            highest_count.count,
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
    let db_connection = db::open()?;
    let results = calculate(&db_connection, args)?;
    db::close(db_connection)?;

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
            for result in results {
                let processed_path = if config::get_use_tilde_on_list() {
                    utils::collapse_to_tilde(result.path)
                } else {
                    result.path
                };

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
            }
        }
    }

    Ok(())
}
