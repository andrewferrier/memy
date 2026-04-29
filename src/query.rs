use core::error::Error;
use log::info;
use rayon::prelude::*;
use rusqlite::Connection;
use std::fs::{Metadata, metadata};
use tracing::instrument;

use crate::db;
use crate::frecency;
use crate::stats;
use crate::types::{NotedCount, UnixTimestamp};
use crate::utils;

pub struct MatchEntry {
    pub path: String,
    pub metadata: Metadata,
    pub noted_count: NotedCount,
    pub last_noted_timestamp: UnixTimestamp,
    pub frecency: f64,
}

pub struct SortedMatches {
    /// Unix timestamp captured at the start of the query (shared for consistency).
    pub now: UnixTimestamp,
    /// Entries that exist on disk and passed the filter, sorted ascending by frecency
    /// (highest frecency is last).
    pub matches: Vec<MatchEntry>,
    /// Entries whose paths no longer exist on disk.
    pub missing: Vec<db::PathEntry>,
    /// Paths flagged for immediate DB deletion by the filter returning [`FilterResult::Delete`].
    pub filter_deletes: Vec<String>,
}

pub enum FilterResult {
    /// Include this entry in the output.
    Include,
    /// Exclude this entry silently.
    Exclude,
    /// Exclude this entry and flag its path for deletion from the database.
    Delete,
}

enum Outcome {
    Match(MatchEntry),
    Missing(db::PathEntry),
    Delete(String),
    Skip,
}

/// Builds a sorted list of frecency matches from the open database connection.
/// The `filter` closure receives `(&PathEntry, &Metadata)` and returns a [`FilterResult`].
#[instrument(level = "trace", skip(filter))]
pub fn build_sorted_matches<F>(
    conn: &Connection,
    filter: F,
) -> Result<SortedMatches, Box<dyn Error>>
where
    F: Fn(&db::PathEntry, &Metadata) -> FilterResult + Send + Sync,
{
    let rows = db::get_rows(conn)?;
    let now = utils::get_unix_timestamp();
    let stats = stats::get(conn)?;

    let (Some(oldest_note), Some(highest_count_entry)) = (stats.oldest_note, stats.highest_count)
    else {
        return Ok(SortedMatches {
            now,
            matches: vec![],
            missing: vec![],
            filter_deletes: vec![],
        });
    };

    let oldest_last_noted_timestamp_hours = utils::timestamp_age_hours(now, oldest_note.timestamp);
    let highest_count = highest_count_entry.count;

    let outcomes: Vec<Outcome> = rows
        .into_par_iter()
        .map(|row| {
            let Ok(meta) = metadata(&row.path) else {
                info!("{} no longer exists, skipping", row.path);
                return Outcome::Missing(row);
            };

            match filter(&row, &meta) {
                FilterResult::Exclude => return Outcome::Skip,
                FilterResult::Delete => return Outcome::Delete(row.path),
                FilterResult::Include => {}
            }

            let frecency = frecency::calculate(
                row.noted_count,
                utils::timestamp_age_hours(now, row.last_noted_timestamp),
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
    let mut missing = vec![];
    let mut filter_deletes = vec![];

    for outcome in outcomes {
        match outcome {
            Outcome::Match(entry) => matches.push(entry),
            Outcome::Missing(row) => missing.push(row),
            Outcome::Delete(path) => filter_deletes.push(path),
            Outcome::Skip => {}
        }
    }

    matches.par_sort_unstable_by_key(|e| e.frecency.to_bits());

    Ok(SortedMatches {
        now,
        matches,
        missing,
        filter_deletes,
    })
}
