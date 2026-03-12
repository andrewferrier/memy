use cli::ListArgs;
use colored::Colorize as _;
use config::DeniedFilesOnList;
use core::error::Error;
use core::fmt::Write as _;
use log::{info, warn};
use rayon::prelude::*;
use rusqlite::{Connection, params_from_iter};
use std::env;
use std::fs::{FileType, metadata};
use std::io::{Write as _, stdout};
use std::process::{Command, Stdio};
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

enum EntryOutcome {
    Result(PathFrecency),
    DeleteFromDb(String),
    Skip,
}

fn handle_missing_path(
    path: String,
    now: UnixTimestamp,
    last_noted_timestamp: UnixTimestamp,
) -> EntryOutcome {
    static MISSING_FILES_DELETE_AFTER_SECS: LazyLock<i64> =
        LazyLock::new(|| i64::from(config::get_missing_files_delete_from_db_after()) * 86400);

    if *MISSING_FILES_DELETE_AFTER_SECS < 0 {
        info!(
            "{path} no longer exists but get_missing_files_delete_from_db_after < 0, so it will not be deleted."
        );
        return EntryOutcome::Skip;
    }

    let last_noted_age_secs = now - last_noted_timestamp;

    if last_noted_age_secs > *MISSING_FILES_DELETE_AFTER_SECS {
        info!(
            "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; older than get_missing_files_delete_from_db_after, removed from database."
        );
        return EntryOutcome::DeleteFromDb(path);
    }

    info!(
        "{path} no longer exists; last noted {last_noted_age_secs} seconds ago; within get_missing_files_delete_from_db_after, retained but skipped."
    );
    EntryOutcome::Skip
}

fn handle_denied_path(path: String) -> EntryOutcome {
    match config::get_denied_files_on_list() {
        DeniedFilesOnList::Warn => {
            warn!("Path {path} is denied, remaining in database.");
            EntryOutcome::Skip
        }
        DeniedFilesOnList::Delete => {
            info!("Path {path} is denied, deleted from database.");
            EntryOutcome::DeleteFromDb(path)
        }
        DeniedFilesOnList::SkipSilently => EntryOutcome::Skip,
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

    let stats = stats::get(conn)?;

    let Some(oldest_note) = stats.oldest_note else {
        return Ok(vec![]);
    };

    let oldest_last_noted_timestamp_hours = utils::timestamp_age_hours(now, oldest_note.timestamp);

    let Some(highest_count) = stats.highest_count else {
        return Ok(vec![]);
    };

    let outcomes: Vec<EntryOutcome> = rows
        .into_par_iter()
        .map(|row| {
            let Ok(metadata) = metadata(&row.path) else {
                return handle_missing_path(row.path, now, row.last_noted_timestamp);
            };

            if let ignore::Match::Ignore(_matched_pat) =
                denylist_matcher.matched_path_or_any_parents(&row.path, metadata.is_dir())
            {
                return handle_denied_path(row.path);
            }

            if (args.files_only && !metadata.is_file())
                || (args.directories_only && !metadata.is_dir())
            {
                return EntryOutcome::Skip;
            }

            if let Some(cutoff_timestamp) = newer_than_timestamp
                && row.last_noted_timestamp < cutoff_timestamp
            {
                return EntryOutcome::Skip;
            }

            let frecency = frecency::calculate(
                row.noted_count,
                utils::timestamp_age_hours(now, row.last_noted_timestamp),
                highest_count.count,
                oldest_last_noted_timestamp_hours,
            );

            EntryOutcome::Result(PathFrecency {
                path: row.path,
                frecency,
                count: row.noted_count,
                last_noted: utils::timestamp_to_iso8601(row.last_noted_timestamp),
                file_type: metadata.file_type(),
            })
        })
        .collect();

    let mut to_output: Vec<PathFrecency> = vec![];
    let mut to_delete: Vec<String> = vec![];

    for outcome in outcomes {
        match outcome {
            EntryOutcome::Result(pf) => to_output.push(pf),
            EntryOutcome::DeleteFromDb(path) => to_delete.push(path),
            EntryOutcome::Skip => {}
        }
    }

    if !to_delete.is_empty() {
        let placeholders = to_delete.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM paths WHERE path IN ({placeholders})");
        conn.execute(&sql, params_from_iter(&to_delete))
            .expect("Deleting paths from DB failed");
    }

    to_output.par_sort_unstable_by(|a, b| {
        a.frecency
            .partial_cmp(&b.frecency)
            .expect("Sort results failed")
    });

    Ok(to_output)
}

fn get_output_filter_command(args: &ListArgs) -> Option<String> {
    if let Some(cmd) = &args.output_filter_command {
        return Some(cmd.clone());
    }

    if let Ok(cmd) = env::var("MEMY_OUTPUT_FILTER") {
        return Some(cmd);
    }

    if let Some(cmd) = config::get_memy_output_filter() {
        return Some(cmd);
    }

    if utils::is_command_available("fzf") {
        return Some("fzf --ansi".to_owned());
    }

    if utils::is_command_available("sk") {
        return Some("sk --ansi".to_owned());
    }

    if utils::is_command_available("fzy") {
        return Some("fzy".to_owned());
    }

    None
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
            let mut output = String::new();

            // plain format
            for result in results {
                let processed_path = if config::get_use_tilde_on_list() {
                    utils::collapse_to_tilde(result.path.clone())
                } else {
                    result.path.clone()
                };

                let path_parts: Vec<&str> = processed_path.rsplitn(2, '/').collect();

                if path_parts.len() == 2 {
                    if result.file_type.is_dir() {
                        let _ = writeln!(output, "{}/{}", path_parts[1], path_parts[0].blue());
                    } else if result.file_type.is_file() {
                        let _ = writeln!(output, "{}/{}", path_parts[1], path_parts[0].green());
                    }
                } else if result.file_type.is_dir() {
                    let _ = writeln!(output, "{}", processed_path.blue());
                } else if result.file_type.is_file() {
                    let _ = writeln!(output, "{}", processed_path.green());
                }
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
        let output_filter_cmd = get_output_filter_command(args).ok_or("No output filter command found. Set MEMY_OUTPUT_FILTER environment variable, memy_output_filter in config, or use --output-filter-command option.")?;

        let shell = env::var("SHELL")
            .ok()
            .filter(|shell| !shell.is_empty())
            .unwrap_or_else(|| "sh".to_owned());

        let mut cmd = Command::new(&shell)
            .arg("-c")
            .arg(&output_filter_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| {
                format!(
                    "Failed to execute output filter command via shell {shell:?} (command: {output_filter_cmd:?}): {err}"
                )
            })?;

        {
            let stdin = cmd.stdin.as_mut().ok_or("Failed to open stdin")?;
            stdin.write_all(output.as_bytes())?;
        };

        let output_data = cmd.wait_with_output()?;
        if !output_data.status.success() {
            let stderr = String::from_utf8_lossy(&output_data.stderr);
            return Err(format!(
                "Output filter command failed via shell {shell:?} with status {}: {}",
                output_data.status,
                stderr.trim()
            )
            .into());
        }

        let output_filter_output = String::from_utf8(output_data.stdout)
            .map_err(|_| "Output filter output is not valid UTF-8")?;
        let expanded_output_filter_output =
            utils::expand_tildes_in_multiline_string(&output_filter_output);
        let mut stdout_handle = stdout().lock();
        stdout_handle.write_all(expanded_output_filter_output.as_bytes())?;
    } else {
        let mut stdout_handle = stdout().lock();
        write!(stdout_handle, "{output}")?;
    }

    Ok(())
}
