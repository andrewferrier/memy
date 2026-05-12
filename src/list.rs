use crate::utils::cli::ListArgs;
use core::error::Error;
use core::fmt::Write as _;
use rusqlite::Connection;
use std::fs::FileType;
use std::io::{Write as _, stdout};
use tracing::instrument;

use crate::utils;
use crate::utils::db;
use crate::utils::query;
use crate::utils::types::Frecency;
use crate::utils::types::NotedCount;

#[derive(serde::Serialize)]
struct PathFrecency {
    path: String,
    frecency: Frecency,
    count: NotedCount,
    last_noted: String,
    #[serde(serialize_with = "crate::utils::serialize_file_type")]
    file_type: FileType,
}

#[instrument(level = "trace")]
fn calculate(conn: &Connection, args: &ListArgs) -> Result<Vec<PathFrecency>, Box<dyn Error>> {
    let newer_than_timestamp = if let Some(ref newer_than_str) = args.newer_than {
        Some(utils::parse_newer_than(newer_than_str)?)
    } else {
        None
    };

    let matches = query::build_sorted_matches(conn, |row, metadata| {
        if (args.files_only && !metadata.is_file()) || (args.directories_only && !metadata.is_dir())
        {
            return query::FilterResult::Exclude;
        }

        if let Some(cutoff_timestamp) = newer_than_timestamp
            && row.last_noted_timestamp < cutoff_timestamp
        {
            return query::FilterResult::Exclude;
        }

        query::FilterResult::Include
    })?;

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
        let filtered = utils::output_filter::run_output_filter(
            &output,
            args.output_filter_command.as_deref(),
        )?;
        let mut stdout_handle = stdout().lock();
        stdout_handle.write_all(filtered.as_bytes())?;
    } else {
        let mut stdout_handle = stdout().lock();
        write!(stdout_handle, "{output}")?;
    }

    Ok(())
}
