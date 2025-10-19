use core::error::Error;
use rusqlite::Connection;
use rusqlite::{OptionalExtension as _, params};
use std::io::{Write as _, stdout};
use std::path::PathBuf;
use tracing::instrument;

use crate::db;
use crate::types::{NotedCount, UnixTimestamp};
use crate::{cli, utils};

#[derive(serde::Serialize)]
pub struct PathTimestamp {
    pub path: PathBuf,
    pub timestamp: UnixTimestamp,
}

#[derive(serde::Serialize)]
pub struct PathCount {
    pub path: PathBuf,
    pub count: NotedCount,
}

#[derive(serde::Serialize)]
pub struct StatsOutput {
    pub total_paths: usize,
    pub oldest_note: Option<PathTimestamp>,
    pub newest_note: Option<PathTimestamp>,
    pub highest_count: Option<PathCount>,
}

#[instrument(level = "trace")]
pub fn get(conn: &Connection) -> Result<StatsOutput, Box<dyn Error>> {
    let rows = db::get_rows(conn)?;

    let oldest = conn.query_row(
        "SELECT path, last_noted_timestamp FROM paths ORDER BY last_noted_timestamp ASC LIMIT 1",
        params![],
        |row| {
            Ok(PathTimestamp{
                path: PathBuf::from(row.get::<_, String>(0)?),
                timestamp: row.get::<_, UnixTimestamp>(1)?,
            })
        },
    )
    .optional()?;

    let newest = conn.query_row(
        "SELECT path, last_noted_timestamp FROM paths ORDER BY last_noted_timestamp DESC LIMIT 1",
        params![],
        |row| {
            Ok(PathTimestamp{
                path: PathBuf::from(row.get::<_, String>(0)?),
                timestamp: row.get::<_, UnixTimestamp>(1)?,
            })
        },
    )
    .optional()?;

    let highest_count = conn
        .query_row(
            "SELECT path, noted_count FROM paths ORDER BY noted_count DESC LIMIT 1",
            params![],
            |row| {
                Ok(PathCount {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    count: row.get::<_, NotedCount>(1)?,
                })
            },
        )
        .optional()?;

    Ok(StatsOutput {
        total_paths: rows.len(),
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
                utils::timestamp_to_iso8601(oldest_note.timestamp),
                oldest_note.path.to_string_lossy()
            )?;
        }

        if let Some(newest_note) = stats.newest_note {
            writeln!(
                stdout_handle,
                "Newest Note: {}, path={}",
                utils::timestamp_to_iso8601(newest_note.timestamp),
                newest_note.path.to_string_lossy()
            )?;
        }

        if let Some(highest_count) = stats.highest_count {
            writeln!(
                stdout_handle,
                "Highest Count: {}, path={}",
                highest_count.count,
                highest_count.path.to_string_lossy()
            )?;
        }
    }

    Ok(())
}
