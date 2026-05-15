use crate::utils::db::FromRow as _;
use core::error::Error;
use rusqlite::Connection;
use rusqlite::{OptionalExtension as _, params};
use std::io::{Write as _, stdout};
use tracing::instrument;

use crate::utils;
use crate::utils::cli;
use crate::utils::db;
use crate::utils::db::TablePathsEntry;

#[derive(serde::Serialize)]
pub struct StatsOutput {
    pub total_paths: usize,
    pub oldest_note: Option<TablePathsEntry>,
    pub newest_note: Option<TablePathsEntry>,
    pub highest_count: Option<TablePathsEntry>,
}

fn query_path_limit_timestamp(
    conn: &Connection,
    order: &str,
) -> Result<Option<TablePathsEntry>, Box<dyn Error>> {
    let result = conn
        .query_row(
            &format!(
                "SELECT path, noted_count, last_noted_timestamp FROM paths ORDER BY last_noted_timestamp {order} LIMIT 1"
            ),
            params![],
            TablePathsEntry::from_row,
        )
        .optional()?;
    Ok(result)
}

#[instrument(level = "trace")]
pub fn get(conn: &Connection) -> Result<StatsOutput, Box<dyn Error>> {
    let row_count = conn.query_row("SELECT COUNT(*) FROM paths", [], |row| row.get(0))?;

    let oldest = query_path_limit_timestamp(conn, "ASC")?;
    let newest = query_path_limit_timestamp(conn, "DESC")?;

    let highest_count = conn
        .query_row(
            "SELECT path, noted_count, last_noted_timestamp FROM paths ORDER BY noted_count DESC LIMIT 1",
            params![],
            TablePathsEntry::from_row,
        )
        .optional()?;

    Ok(StatsOutput {
        total_paths: row_count,
        oldest_note: oldest,
        newest_note: newest,
        highest_count,
    })
}

#[instrument(level = "trace")]
pub fn command(args: &cli::StatsArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open()?;
    let stats = get(&db_connection)?;
    db::close(db_connection)?;

    let mut stdout_handle = stdout().lock();

    if args.format.as_str() == "json" {
        let json_output =
            serde_json::to_string_pretty(&stats).expect("Failed to serialize stats to JSON");
        writeln!(stdout_handle, "{json_output}")?;
    } else {
        writeln!(stdout_handle, "Total Paths: {}", stats.total_paths)?;

        if let Some(oldest_note) = stats.oldest_note {
            writeln!(
                stdout_handle,
                "Oldest Note: {}, path={}",
                utils::time::timestamp_to_iso8601(oldest_note.last_noted_timestamp),
                oldest_note.path
            )?;
        }

        if let Some(newest_note) = stats.newest_note {
            writeln!(
                stdout_handle,
                "Newest Note: {}, path={}",
                utils::time::timestamp_to_iso8601(newest_note.last_noted_timestamp),
                newest_note.path
            )?;
        }

        if let Some(highest_count) = stats.highest_count {
            writeln!(
                stdout_handle,
                "Highest Count: {}, path={}",
                highest_count.noted_count, highest_count.path,
            )?;
        }
    }

    Ok(())
}
