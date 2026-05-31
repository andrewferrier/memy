use crate::utils::db::FromRow as _;
use chrono::{Datelike as _, Local, TimeZone as _, Timelike as _};
use core::error::Error;
use rusqlite::Connection;
use rusqlite::{OptionalExtension as _, params};
use std::collections::BTreeMap;
use std::io::{Write as _, stdout};
use tracing::instrument;

use crate::utils;
use crate::utils::cli;
use crate::utils::db;
use crate::utils::db::TablePathsEntry;
use crate::utils::graphs::{COL_WIDTH, get_terminal_width, render_bar_chart, render_column_chart};
use crate::utils::types::{NotedCount, UnixTimestamp};

#[derive(serde::Serialize)]
pub struct StatsOutput {
    pub total_paths: usize,
    pub files_count: usize,
    pub dirs_count: usize,
    pub missing_count: usize,
    pub oldest_note: Option<TablePathsEntry>,
    pub newest_note: Option<TablePathsEntry>,
    pub highest_count: Option<TablePathsEntry>,
    #[serde(skip)]
    pub all_timestamps: Vec<UnixTimestamp>,
    #[serde(skip)]
    pub all_noted_counts: Vec<NotedCount>,
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

    let rows = db::get_rows(conn)?;
    let mut files_count = 0_usize;
    let mut dirs_count = 0_usize;
    let mut missing_count = 0_usize;
    let mut all_timestamps = Vec::with_capacity(rows.len());
    let mut all_noted_counts = Vec::with_capacity(rows.len());

    for row in &rows {
        all_timestamps.push(row.last_noted_timestamp);
        all_noted_counts.push(row.noted_count);
        match std::fs::metadata(&row.path) {
            Ok(file_meta) if file_meta.is_file() => files_count += 1,
            Ok(dir_meta) if dir_meta.is_dir() => dirs_count += 1,
            _ => missing_count += 1,
        }
    }

    Ok(StatsOutput {
        total_paths: row_count,
        files_count,
        dirs_count,
        missing_count,
        oldest_note: oldest,
        newest_note: newest,
        highest_count,
        all_timestamps,
        all_noted_counts,
    })
}

fn build_histogram(counts: &[NotedCount]) -> Vec<(String, usize)> {
    if counts.is_empty() {
        return vec![];
    }
    let max_count = *counts.iter().max().expect("non-empty");
    // Number of log-2 buckets needed: floor(log2(max_count)) + 1, capped at 63.
    // Bucket 0 = [1, 1]; bucket n >= 1 = [2^n, 2^(n+1)-1].
    let n_buckets = ((u64::BITS - max_count.leading_zeros()) as usize).min(63);
    let mut bucket_counts = vec![0_usize; n_buckets];
    for &count in counts {
        if count == 0 {
            continue;
        }
        let idx = ((u64::BITS - count.leading_zeros() - 1) as usize).min(n_buckets - 1);
        bucket_counts[idx] += 1;
    }
    bucket_counts
        .into_iter()
        .enumerate()
        .filter(|(_, c)| *c > 0)
        .map(|(idx, c)| {
            let label = if idx == 0 {
                "1".to_owned()
            } else {
                let low = 1u64 << idx;
                let high = (1u64 << (idx + 1)) - 1;
                format!("{low}-{high}")
            };
            (label, c)
        })
        .collect()
}

#[derive(Clone, Copy)]
enum TimeGranularity {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

/// Pick the finest granularity whose bucket count for `total_span_secs` fits in `max_cols`.
fn choose_granularity_for_width(total_span_secs: i64, max_cols: usize) -> TimeGranularity {
    let candidates = [
        (TimeGranularity::Hour, 3_600_i64),
        (TimeGranularity::Day, 86_400_i64),
        (TimeGranularity::Week, 7 * 86_400_i64),
        (TimeGranularity::Month, 30 * 86_400_i64),
        (TimeGranularity::Year, 365 * 86_400_i64),
    ];
    for (granularity, interval_secs) in candidates {
        // +1 so a span of exactly one interval produces 2 buckets (start + end).
        let buckets = usize::try_from(total_span_secs / interval_secs + 1).unwrap_or(usize::MAX);
        if buckets <= max_cols {
            return granularity;
        }
    }
    TimeGranularity::Year
}

/// Advance one bucket forward from `floor_ts`.
fn next_bucket_ts(floor_ts: i64, granularity: TimeGranularity) -> i64 {
    match granularity {
        TimeGranularity::Hour => floor_ts + 3_600,
        TimeGranularity::Day => floor_ts + 86_400,
        TimeGranularity::Week => floor_ts + 7 * 86_400,
        TimeGranularity::Month => {
            let dt = Local
                .timestamp_opt(floor_ts, 0)
                .single()
                .expect("valid timestamp");
            let (year, month) = if dt.month() == 12 {
                (dt.year() + 1, 1_u32)
            } else {
                (dt.year(), dt.month() + 1)
            };
            Local
                .with_ymd_and_hms(year, month, 1, 0, 0, 0)
                .single()
                .expect("valid date")
                .timestamp()
        }
        TimeGranularity::Year => {
            let dt = Local
                .timestamp_opt(floor_ts, 0)
                .single()
                .expect("valid timestamp");
            Local
                .with_ymd_and_hms(dt.year() + 1, 1, 1, 0, 0, 0)
                .single()
                .expect("valid date")
                .timestamp()
        }
    }
}

const fn granularity_name(granularity: TimeGranularity) -> &'static str {
    match granularity {
        TimeGranularity::Hour => "hour",
        TimeGranularity::Day => "day",
        TimeGranularity::Week => "week",
        TimeGranularity::Month => "month",
        TimeGranularity::Year => "year",
    }
}

fn bucket_floor_ts(ts: UnixTimestamp, granularity: TimeGranularity) -> i64 {
    let dt = Local
        .timestamp_opt(ts, 0)
        .single()
        .expect("valid timestamp");
    match granularity {
        TimeGranularity::Hour => Local
            .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), 0, 0)
            .single()
            .expect("valid date")
            .timestamp(),
        TimeGranularity::Day => Local
            .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
            .single()
            .expect("valid date")
            .timestamp(),
        TimeGranularity::Week => {
            let weekday_offset = i64::from(dt.weekday().num_days_from_monday());
            let day_start = Local
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                .single()
                .expect("valid date")
                .timestamp();
            day_start - weekday_offset * 86_400
        }
        TimeGranularity::Month => Local
            .with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
            .single()
            .expect("valid date")
            .timestamp(),
        TimeGranularity::Year => Local
            .with_ymd_and_hms(dt.year(), 1, 1, 0, 0, 0)
            .single()
            .expect("valid date")
            .timestamp(),
    }
}

fn bucket_display_label(floor_ts: i64, granularity: TimeGranularity) -> String {
    let dt = Local
        .timestamp_opt(floor_ts, 0)
        .single()
        .expect("valid timestamp");
    match granularity {
        TimeGranularity::Hour => dt.format("%d %H:00").to_string(),
        TimeGranularity::Day => dt.format("%m-%d").to_string(),
        TimeGranularity::Week => {
            let iso = dt.iso_week();
            format!("{}-W{:02}", iso.year(), iso.week())
        }
        TimeGranularity::Month => dt.format("%Y-%m").to_string(),
        TimeGranularity::Year => dt.format("%Y").to_string(),
    }
}

fn build_time_chart(
    timestamps: &[UnixTimestamp],
    now: UnixTimestamp,
    terminal_width: usize,
) -> (String, Vec<(String, usize)>) {
    const ESTIMATED_Y_AXIS_WIDTH: usize = 7;

    if timestamps.len() < 2 {
        return (String::new(), vec![]);
    }

    let min_ts = *timestamps.iter().min().expect("non-empty, checked above");
    let total_span = now.saturating_sub(min_ts).max(1);

    let available_cols = (terminal_width.saturating_sub(ESTIMATED_Y_AXIS_WIDTH) / COL_WIDTH).max(1);
    let granularity = choose_granularity_for_width(total_span, available_cols);

    let mut bucket_counts: BTreeMap<i64, usize> = BTreeMap::new();
    for &ts in timestamps {
        *bucket_counts
            .entry(bucket_floor_ts(ts, granularity))
            .or_insert(0) += 1;
    }

    let first_bucket = bucket_floor_ts(min_ts, granularity);
    let now_bucket = bucket_floor_ts(now, granularity);
    let mut entries = Vec::new();
    let mut current = first_bucket;
    loop {
        let count = *bucket_counts.get(&current).unwrap_or(&0);
        entries.push((bucket_display_label(current, granularity), count));
        if current >= now_bucket {
            break;
        }
        current = next_bucket_ts(current, granularity);
    }

    let title = format!("Time Distribution (by {})", granularity_name(granularity));
    (title, entries)
}

#[instrument(level = "trace")]
pub fn command(args: &cli::StatsArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open()?;
    let stats = get(&db_connection)?;
    db::close(db_connection)?;

    let mut stdout_handle = stdout().lock();

    if args.format.as_str() == "json" {
        let json_str =
            serde_json::to_string_pretty(&stats).expect("Failed to serialize stats to JSON");
        writeln!(stdout_handle, "{json_str}")?;
    } else {
        writeln!(stdout_handle, "Total Paths: {}", stats.total_paths)?;
        writeln!(stdout_handle, "  Files: {}", stats.files_count)?;
        writeln!(stdout_handle, "  Directories: {}", stats.dirs_count)?;
        if stats.missing_count > 0 {
            writeln!(stdout_handle, "  Missing: {}", stats.missing_count)?;
        }

        if let Some(oldest_note) = &stats.oldest_note {
            writeln!(
                stdout_handle,
                "Oldest Note: {}, path={}",
                utils::time::timestamp_to_iso8601(oldest_note.last_noted_timestamp),
                oldest_note.path
            )?;
        }

        if let Some(newest_note) = &stats.newest_note {
            writeln!(
                stdout_handle,
                "Newest Note: {}, path={}",
                utils::time::timestamp_to_iso8601(newest_note.last_noted_timestamp),
                newest_note.path
            )?;
        }

        if let Some(highest_count) = &stats.highest_count {
            writeln!(
                stdout_handle,
                "Highest Count: {}, path={}",
                highest_count.noted_count, highest_count.path,
            )?;
        }

        let terminal_width = get_terminal_width();

        if !stats.all_noted_counts.is_empty() {
            let histogram = build_histogram(&stats.all_noted_counts);
            if !histogram.is_empty() {
                writeln!(stdout_handle)?;
                let chart = render_bar_chart("Count Distribution", &histogram, terminal_width);
                write!(stdout_handle, "{chart}")?;
            }
        }

        if stats.all_timestamps.len() >= 2 {
            let (time_title, time_entries) = build_time_chart(
                &stats.all_timestamps,
                Local::now().timestamp(),
                terminal_width,
            );
            if !time_entries.is_empty() {
                writeln!(stdout_handle)?;
                let chart = render_column_chart(&time_title, &time_entries, terminal_width);
                write!(stdout_handle, "{chart}")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_single_bucket() {
        let counts = vec![1_u64, 1, 1];
        let result = build_histogram(&counts);
        assert_eq!(result.len(), 1, "Expected one bucket");
        assert_eq!(result[0].0, "1");
        assert_eq!(result[0].1, 3);
    }

    #[test]
    fn test_histogram_multiple_buckets() {
        let counts = vec![1_u64, 2, 5, 10, 20, 50, 100];
        let result = build_histogram(&counts);
        assert_eq!(result.len(), 7, "Expected 7 buckets populated");
        assert_eq!(result[0].0, "1");
        assert_eq!(result[6].0, "64-127");
    }

    #[test]
    fn test_histogram_dynamic_stops_at_max() {
        let counts = vec![1_u64, 2, 3];
        let result = build_histogram(&counts);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "1");
        assert_eq!(result[1].0, "2-3");
    }

    #[test]
    fn test_histogram_skips_empty_buckets() {
        let counts = vec![1_u64, 4, 5, 6];
        let result = build_histogram(&counts);
        for (_, c) in &result {
            assert!(*c > 0, "No empty bucket should appear");
        }
        assert_eq!(result[0].0, "1");
        assert_eq!(result[1].0, "4-7");
    }

    #[test]
    fn test_histogram_empty() {
        let result = build_histogram(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_time_chart_too_few_timestamps() {
        let base: i64 = 1_700_000_000;

        let (title, entries) = build_time_chart(&[], base, 80);
        assert!(title.is_empty());
        assert!(entries.is_empty());

        let (title2, entries2) = build_time_chart(&[1_000_000], base, 80);
        assert!(title2.is_empty());
        assert!(entries2.is_empty());
    }

    #[test]
    fn test_time_chart_hourly_granularity() {
        // Terminal=80 → available_cols=(80-7)/9=8.
        // Hourly fits when span/3600+1 ≤ 8 → span ≤ 7 hours.
        let base: i64 = 1_700_000_000;
        let span = 3_600_i64; // 1 hour
        let timestamps = vec![base, base + span];
        let (title, entries) = build_time_chart(&timestamps, base + span, 80);
        assert!(title.contains("hour"), "Expected hourly, got: {title}");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_time_chart_daily_granularity() {
        // span=3 days: hourly would need 72+1=73 cols (>8); daily needs 3+1=4 ≤ 8 ✓
        let base: i64 = 1_700_000_000;
        let span = 3 * 86_400_i64;
        let (title, entries) = build_time_chart(&[base, base + span], base + span, 80);
        assert!(title.contains("day"), "Expected daily, got: {title}");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_time_chart_weekly_granularity() {
        // span=30 days: hourly=721, daily=31 (both >8); weekly=30/7+1=5 ≤ 8 ✓
        let base: i64 = 1_700_000_000;
        let span = 30 * 86_400_i64;
        let (title, _) = build_time_chart(&[base, base + span], base + span, 80);
        assert!(title.contains("week"), "Expected weekly, got: {title}");
    }

    #[test]
    fn test_time_chart_monthly_granularity() {
        // span=200 days: weekly=200/7+1=29 (>8); monthly=200/30+1=7 ≤ 8 ✓
        let base: i64 = 1_700_000_000;
        let span = 200 * 86_400_i64;
        let (title, _) = build_time_chart(&[base, base + span], base + span, 80);
        assert!(title.contains("month"), "Expected monthly, got: {title}");
    }

    #[test]
    fn test_time_chart_yearly_granularity() {
        // span=3 years: monthly=3*365/30+1=37 (>8); yearly=3+1=4 ≤ 8 ✓
        let base: i64 = 1_000_000_000;
        let span = 3 * 365 * 86_400_i64;
        let (title, entries) = build_time_chart(&[base, base + span], base + span, 80);
        assert!(title.contains("year"), "Expected yearly, got: {title}");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_time_chart_all_same_timestamp() {
        let ts: i64 = 1_700_000_000;
        let timestamps = vec![ts, ts, ts];
        let (title, entries) = build_time_chart(&timestamps, ts, 80);
        assert!(!title.is_empty());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].1, 3);
    }

    #[test]
    fn test_time_chart_includes_empty_buckets_between_notes() {
        let base: i64 = 1_700_000_000;
        let day_floor = base - (base % 86_400);
        let span = 4 * 86_400_i64;
        let timestamps = vec![day_floor, day_floor + span];
        let (title, entries) = build_time_chart(&timestamps, day_floor + span, 80);
        assert!(title.contains("day"), "Expected daily, got: {title}");
        assert!(
            entries.len() >= 5,
            "Expected at least 5 daily buckets, got {}",
            entries.len()
        );
        let total: usize = entries.iter().map(|(_, c)| c).sum();
        assert_eq!(total, 2, "All timestamps must be covered");
    }

    #[test]
    fn test_time_chart_extends_to_now() {
        let base: i64 = 1_700_000_000;
        let now = base + 6 * 86_400; // 6 days later → span/86400+1=7 ≤ 8 → daily
        let timestamps = vec![base, base + 86_400]; // only 2 days of notes
        let (title, entries) = build_time_chart(&timestamps, now, 80);
        assert!(title.contains("day"), "Expected daily, got: {title}");
        assert!(
            entries.len() >= 6,
            "Expected chart to extend to now (≥6 buckets), got {}",
            entries.len()
        );
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(proptest::test_runner::Config::with_cases(200))]

            #[allow(clippy::unwrap_used, reason = "proptest macros use unwrap internally")]
            #[test]
            fn proptest_histogram_covers_all_counts(
                counts in prop::collection::vec(1u64..=1_000_000u64, 0..100)
            ) {
                let histogram = build_histogram(&counts);
                let total: usize = histogram.iter().map(|(_, c)| c).sum();
                prop_assert_eq!(total, counts.len());
            }

            #[allow(clippy::unwrap_used, reason = "proptest macros use unwrap internally")]
            #[test]
            fn proptest_histogram_no_empty_buckets(
                counts in prop::collection::vec(1u64..=1_000_000u64, 1..100)
            ) {
                let histogram = build_histogram(&counts);
                for (_, count) in &histogram {
                    prop_assert!(*count > 0);
                }
            }

            #[allow(clippy::unwrap_used, reason = "proptest macros use unwrap internally")]
            #[test]
            fn proptest_time_chart_covers_all_timestamps(
                base in 1_000_000i64..1_800_000_000i64,
                offsets in prop::collection::vec(0i64..=3_650i64 * 86_400, 2..50)
            ) {
                let timestamps: Vec<i64> = offsets.iter().map(|&o| base + o).collect();
                let max_ts = *timestamps.iter().max().unwrap();
                let now = max_ts + 86_400;
                let (_, entries) = build_time_chart(&timestamps, now, 80);
                let total: usize = entries.iter().map(|(_, c)| c).sum();
                prop_assert_eq!(total, timestamps.len());
            }
        }
    }
}
