pub mod cli;
pub mod config;
pub mod db;
pub mod denylist_default;
pub mod frecency;
pub mod logging;
pub mod output;
pub mod path;
pub mod query;
pub mod types;

use chrono::{DateTime, Local, TimeZone as _};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use self::types::UnixTimestamp;
use self::types::UnixTimestampHours;

#[allow(
    clippy::cast_possible_wrap,
    reason = "Value is never going to be large enough in practice that it can't be cast"
)]
pub fn get_unix_timestamp() -> UnixTimestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as UnixTimestamp
}

pub fn timestamp_to_iso8601(timestamp: UnixTimestamp) -> String {
    let datetime: DateTime<Local> = Local
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(|| panic!("Can't convert timestamp {timestamp}"));

    datetime.to_rfc3339()
}

pub fn timestamp_age_hours(now: UnixTimestamp, timestamp: UnixTimestamp) -> UnixTimestampHours {
    let age_seconds = now - timestamp;
    age_seconds as f64 / 3600.0
}

pub fn parse_newer_than(input: &str) -> Result<UnixTimestamp, Box<dyn core::error::Error>> {
    // First try parsing as a duration using humantime
    if let Ok(duration) = humantime::parse_duration(input) {
        let now = get_unix_timestamp();
        let cutoff = now - duration.as_secs().cast_signed() as UnixTimestamp;
        return Ok(cutoff);
    }

    // Try parsing as ISO-8601 datetime
    if let Ok(datetime) = DateTime::parse_from_rfc3339(input) {
        return Ok(datetime.timestamp());
    }

    // Try parsing as a date-only string (partial ISO-8601)
    // Handle formats like "2025-01-01"
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let datetime = naive_date
            .and_hms_opt(0, 0, 0)
            .ok_or("Failed to create datetime")?;
        let local_datetime = Local
            .from_local_datetime(&datetime)
            .single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    // Try parsing as datetime without timezone
    if let Ok(naive_datetime) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        let local_datetime = Local
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    Err(format!("Unable to parse '{input}' as a duration or date/time").into())
}

pub fn is_command_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "Reference required for Serialize"
)]
pub fn serialize_file_type<S>(ft: &std::fs::FileType, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(match (ft.is_dir(), ft.is_file(), ft.is_symlink()) {
        (true, _, _) => "dir",
        (_, true, _) => "file",
        (_, _, true) => "symlink",
        _ => "other",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use proptest::prelude::*;

    #[test]
    fn test_parse_newer_than_humantime_hour() {
        let now = get_unix_timestamp();
        let cutoff = parse_newer_than("1h").expect("'1h' should parse");
        let diff = now - cutoff;
        assert!(
            (3598..=3602).contains(&diff),
            "expected diff ~3600, got {diff}"
        );
    }

    #[test]
    fn test_parse_newer_than_humantime_days() {
        let now = get_unix_timestamp();
        let cutoff = parse_newer_than("2days").expect("'2days' should parse");
        let expected = 2 * 86400_i64;
        let diff = now - cutoff;
        assert!(
            diff >= expected - 2 && diff <= expected + 2,
            "expected diff ~{expected}, got {diff}"
        );
    }

    #[test]
    fn test_parse_newer_than_iso8601_datetime() {
        let cutoff =
            parse_newer_than("2025-01-01T00:00:00Z").expect("ISO 8601 datetime should parse");
        assert_eq!(
            cutoff, 1_735_689_600,
            "unexpected epoch for 2025-01-01T00:00:00Z"
        );
    }

    #[test]
    fn test_parse_newer_than_date_only() {
        let result = parse_newer_than("2025-06-15");
        assert!(
            result.is_ok(),
            "date-only '2025-06-15' should parse, got {result:?}"
        );
    }

    #[test]
    fn test_parse_newer_than_datetime_no_timezone() {
        let result = parse_newer_than("2025-06-15T10:30:00");
        assert!(
            result.is_ok(),
            "datetime without timezone should parse, got {result:?}"
        );
    }

    #[test]
    fn test_parse_newer_than_invalid_returns_error() {
        assert!(parse_newer_than("not-a-date").is_err());
        assert!(parse_newer_than("yesterday").is_err());
        assert!(parse_newer_than("").is_err());
    }

    proptest! {
        #[test]
        fn round_trip_timestamp_serialization(timestamp in 0..=DateTime::parse_from_rfc3339("9999-12-31T23:59:59+00:00").expect("Cannot parse").timestamp()) {
            let iso8601 = timestamp_to_iso8601(timestamp);
            let parsed_datetime = DateTime::parse_from_rfc3339(&iso8601).unwrap_or_else(|_| panic!("Failed to parse ISO8601 string {iso8601}"));
            let round_trip_timestamp = parsed_datetime.timestamp();

            prop_assert_eq!(timestamp, round_trip_timestamp);
        }
    }
}
