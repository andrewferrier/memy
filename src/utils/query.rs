use core::error::Error;
use rayon::prelude::*;
use rusqlite::{Connection, params_from_iter};
use std::fs::{Metadata, metadata};
use tracing::instrument;
use tracing::{info, warn};

use super::config;
use super::config::DeniedFilesOnList;
use super::db;
use super::frecency;
use super::types::{NotedCount, UnixTimestamp};
use super::{get_unix_timestamp, timestamp_age_hours};
use crate::stats;

pub struct MatchEntry {
    pub path: String,
    pub metadata: Metadata,
    pub noted_count: NotedCount,
    pub last_noted_timestamp: UnixTimestamp,
    pub frecency: f64,
}

pub enum FilterResult {
    /// Include this entry in the output.
    Include,
    /// Exclude this entry silently.
    Exclude,
}

enum Outcome {
    Match(MatchEntry),
    Delete(String),
    Skip,
}

/// Builds a sorted list of frecency matches from the open database connection.
/// The `filter` closure receives `(&PathEntry, &Metadata)` and returns a [`FilterResult`].
#[instrument(level = "trace", skip(filter))]
pub fn build_sorted_matches<F>(
    conn: &Connection,
    filter: F,
) -> Result<Vec<MatchEntry>, Box<dyn Error>>
where
    F: Fn(&db::PathEntry, &Metadata) -> FilterResult + Send + Sync,
{
    let rows = db::get_rows(conn)?;
    let now = get_unix_timestamp();
    let stats = stats::get(conn)?;

    let (Some(oldest_note), Some(highest_count_entry)) = (stats.oldest_note, stats.highest_count)
    else {
        return Ok(vec![]);
    };

    let oldest_last_noted_timestamp_hours = timestamp_age_hours(now, oldest_note.timestamp);
    let highest_count = highest_count_entry.count;

    let denylist_matcher = config::get_denylist_matcher();
    let missing_files_delete_after_secs: i64 =
        i64::from(config::get_missing_files_delete_from_db_after()) * 86400;

    let outcomes: Vec<Outcome> = rows
        .into_par_iter()
        .map(|row| {
            let Ok(meta) = metadata(&row.path) else {
                let last_noted_age_secs = now - row.last_noted_timestamp;
                if missing_files_delete_after_secs >= 0
                    && last_noted_age_secs > missing_files_delete_after_secs
                {
                    info!(
                        "{} no longer exists; last noted {last_noted_age_secs} seconds ago; older than get_missing_files_delete_from_db_after, removed from database.",
                        row.path
                    );
                    return Outcome::Delete(row.path);
                }
                info!(
                    "{} no longer exists; last noted {last_noted_age_secs} seconds ago; {}.",
                    row.path,
                    if missing_files_delete_after_secs < 0 {
                        "get_missing_files_delete_from_db_after < 0, so it will not be deleted"
                    } else {
                        "within get_missing_files_delete_from_db_after, retained but skipped"
                    }
                );
                return Outcome::Skip;
            };

            if let ignore::Match::Ignore(_) =
                denylist_matcher.matched_path_or_any_parents(&row.path, meta.is_dir())
            {
                match config::get_denied_files_on_list() {
                    DeniedFilesOnList::Delete => {
                        info!("Path {} is denied, deleted from database.", row.path);
                        return Outcome::Delete(row.path);
                    }
                    DeniedFilesOnList::Warn => {
                        warn!("Path {} is denied, remaining in database.", row.path);
                    }
                    DeniedFilesOnList::SkipSilently => {}
                }
                return Outcome::Skip;
            }

            match filter(&row, &meta) {
                FilterResult::Exclude => return Outcome::Skip,
                FilterResult::Include => {}
            }

            let frecency = frecency::calculate(
                row.noted_count,
                timestamp_age_hours(now, row.last_noted_timestamp),
                highest_count,
                oldest_last_noted_timestamp_hours,
            );

            Outcome::Match(MatchEntry {
                path: row.path,
                metadata: meta,
                noted_count: row.noted_count,
                last_noted_timestamp: row.last_noted_timestamp,
                frecency,
            })
        })
        .collect();

    let mut matches = vec![];
    let mut to_delete = vec![];

    for outcome in outcomes {
        match outcome {
            Outcome::Match(entry) => matches.push(entry),
            Outcome::Delete(path) => to_delete.push(path),
            Outcome::Skip => {}
        }
    }

    if !to_delete.is_empty() {
        let placeholders = to_delete.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!("DELETE FROM paths WHERE path IN ({placeholders})");
        conn.execute(&sql, params_from_iter(&to_delete))
            .expect("Deleting paths from DB failed");
    }

    matches.par_sort_unstable_by_key(|e| e.frecency.to_bits());

    Ok(matches)
}
