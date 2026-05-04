use crate::utils::cli::ListArgs;
use crate::utils::config::DeniedFilesOnList;
use core::error::Error;
use core::fmt::Write as _;
use rusqlite::{Connection, params_from_iter};
use std::fs::FileType;
use std::io::{Write as _, stdout};
use std::sync::LazyLock;
use tracing::instrument;
use tracing::{info, warn};

use crate::utils;
use crate::utils::config;
use crate::utils::db;
use crate::utils::query;
use crate::utils::types::Frecency;
use crate::utils::types::NotedCount;
use crate::utils::types::UnixTimestamp;

#[derive(serde::Serialize)]
struct PathFrecency {
    path: String,
    frecency: Frecency,
    count: NotedCount,
    last_noted: String,
    #[serde(serialize_with = "crate::utils::serialize_file_type")]
    file_type: FileType,
}

/// Returns `Some(path)` if the missing entry should be deleted from the database,
/// or `None` if it should be retained (but skipped from output).
fn handle_missing_path(
    path: String,
    now: UnixTimestamp,
    last_noted_timestamp: UnixTimestamp,
) -> Option<String> {
    static MISSING_FILES_DELETE_AFTER_SECS: LazyLock<i64> =
        LazyLock::new(|| i64::from(config::get_missing_files_delete_from_db_after()) * 86400);

    if *MISSING_FILES_DELETE_AFTER_SECS < 0 {
        info!(
            "{path} no longer exists but get_missing_files_delete_from_db_after < 0, so it will not be deleted."
        );
        return None;
    }

    let last_noted_age_secs = now - last_noted_timestamp;

    if last_noted_age_secs > *MISSING_FILES_DELETE_AFTER_SECS {
        info!(
            "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; older than get_missing_files_delete_from_db_after, removed from database."
        );
        return Some(path);
    }

    info!(
        "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; within get_missing_files_delete_from_db_after, retained but skipped."
    );
    None
}

#[instrument(level = "trace")]
fn calculate(conn: &Connection, args: &ListArgs) -> Result<Vec<PathFrecency>, Box<dyn Error>> {
    let denylist_matcher = config::get_denylist_matcher();

    let newer_than_timestamp = if let Some(ref newer_than_str) = args.newer_than {
        Some(utils::parse_newer_than(newer_than_str)?)
    } else {
        None
    };

    // Paths to delete due to denylist policy; returned as FilterResult::Delete.
    let query::SortedMatches {
        now,
        matches,
        missing,
        filter_deletes,
    } = query::build_sorted_matches(conn, |row, meta| {
        if let ignore::Match::Ignore(_matched_pat) =
            denylist_matcher.matched_path_or_any_parents(&row.path, meta.is_dir())
        {
            match config::get_denied_files_on_list() {
                DeniedFilesOnList::Warn => {
                    warn!("Path {} is denied, remaining in database.", row.path);
                }
                DeniedFilesOnList::Delete => {
                    info!("Path {} is denied, deleted from database.", row.path);
                    return query::FilterResult::Delete;
                }
                DeniedFilesOnList::SkipSilently => {}
            }
            return query::FilterResult::Exclude;
        }

        if (args.files_only && !meta.is_file()) || (args.directories_only && !meta.is_dir()) {
            return query::FilterResult::Exclude;
        }

        if let Some(cutoff_timestamp) = newer_than_timestamp
            && row.last_noted_timestamp < cutoff_timestamp
        {
            return query::FilterResult::Exclude;
        }

        query::FilterResult::Include
    })?;

    let mut to_delete: Vec<String> = filter_deletes;

    for row in missing {
        if let Some(path) = handle_missing_path(row.path, now, row.last_noted_timestamp) {
            to_delete.push(path);
        }
    }

    if !to_delete.is_empty() {
        let placeholders = to_delete.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM paths WHERE path IN ({placeholders})");
        conn.execute(&sql, params_from_iter(&to_delete))
            .expect("Deleting paths from DB failed");
    }

    let to_output = matches
        .into_iter()
        .map(|m| PathFrecency {
            path: m.path,
            frecency: m.frecency,
            count: m.noted_count,
            last_noted: utils::timestamp_to_iso8601(m.last_noted_timestamp),
            file_type: m.metadata.file_type(),
        })
        .collect();

    Ok(to_output)
}

#[instrument(level = "trace", skip(results, args))]
fn format_results(results: &[PathFrecency], args: &ListArgs) -> Result<String, Box<dyn Error>> {
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&results)
                .expect("Failed to serialize results to JSON");
            Ok(format!("{}\n", &json_output))
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            for result in results {
                wtr.serialize(result)?;
            }
            wtr.flush()?;
            Ok(String::from_utf8(wtr.into_inner()?)?)
        }
        _ => {
            let mut output = String::with_capacity(results.len() * 256);
            for result in results {
                let _ = writeln!(
                    output,
                    "{}",
                    utils::format_path_colored(&result.path, result.file_type.is_dir())
                );
            }
            Ok(output)
        }
    }
}

#[instrument(level = "trace")]
pub fn command(args: &ListArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open()?;
    let results: Vec<PathFrecency> = calculate(&db_connection, args)?;
    db::close(db_connection)?;

    let output = format_results(&results, args)?;

    if args.output_filter {
        let output_filter_cmd = utils::output_filter::get_output_filter_command(args.output_filter_command.as_deref())
            .ok_or("No output filter command found. Set MEMY_OUTPUT_FILTER environment variable, memy_output_filter in config, or use --output-filter-command option.")?;

        let filtered = utils::output_filter::run_output_filter(&output, &output_filter_cmd)?;
        let mut stdout_handle = stdout().lock();
        stdout_handle.write_all(filtered.as_bytes())?;
    } else {
        let mut stdout_handle = stdout().lock();
        write!(stdout_handle, "{output}")?;
    }

    Ok(())
}
