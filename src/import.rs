use core::error::Error;
use rusqlite::Connection;
use std::fs;
use tracing::{debug, info};

use crate::utils;
use crate::utils::types::NotedCount;
use crate::utils::types::UnixTimestamp;

pub type FasdScore = f64;

#[derive(Debug)]
pub struct MemyEntry {
    pub filename: String,
    pub count: NotedCount,
    pub timestamp: UnixTimestamp,
}

fn from_fasd_str(s: &str) -> Result<MemyEntry, Box<dyn Error>> {
    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid entry: {s}").into());
    }

    let filename = parts[0].to_owned();
    let score = parts[1]
        .parse::<FasdScore>()
        .map_err(|e| format!("Invalid score: {e}"))?;

    if score < 0.0 {
        return Err(format!("Score cannot be negative: {score}").into());
    }

    let timestamp = parts[2]
        .parse::<UnixTimestamp>()
        .map_err(|e| format!("Invalid timestamp: {e}"))?;

    #[allow(clippy::cast_possible_truncation, reason = "Round is intentional")]
    #[allow(clippy::cast_sign_loss, reason = "Round is intentional")]
    Ok(MemyEntry {
        filename,
        count: score.round() as NotedCount,
        timestamp,
    })
}

fn from_whitespace_split_str(s: &str) -> Result<MemyEntry, Box<dyn Error>> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(format!("Invalid entry: {s}").into());
    }

    let count = parts[0]
        .parse::<f64>()
        .map_err(|e| format!("Invalid count: {e}"))?;
    let path = parts[1].to_owned();
    let timestamp = utils::get_unix_timestamp();

    if count < 0.0 {
        return Err(format!("Count cannot be negative: {count}").into());
    }

    Ok(MemyEntry {
        filename: path,
        #[allow(clippy::cast_possible_truncation, reason = "Round is intentional")]
        #[allow(clippy::cast_sign_loss, reason = "Round is intentional")]
        count: count.round() as u64,
        timestamp,
    })
}

fn parse_state_generic<F>(contents: &str, line_parser: F) -> Result<Vec<MemyEntry>, Box<dyn Error>>
where
    F: Fn(&str) -> Result<MemyEntry, Box<dyn Error>>,
{
    contents.lines().map(line_parser).collect()
}

fn parse_fasd_state(contents: &str) -> Result<Vec<MemyEntry>, Box<dyn Error>> {
    parse_state_generic(contents, from_fasd_str)
}

fn parse_zoxide_state(contents: &str) -> Result<Vec<MemyEntry>, Box<dyn Error>> {
    parse_state_generic(contents, from_whitespace_split_str)
}

fn parse_autojump_state(contents: &str) -> Result<Vec<MemyEntry>, Box<dyn Error>> {
    parse_state_generic(contents, from_whitespace_split_str)
}

fn insert_into_db(conn: &mut Connection, entries: Vec<MemyEntry>) -> Result<(), Box<dyn Error>> {
    let tx = conn.transaction().expect("Cannot start DB transaction");

    for entry in entries {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "We expect this score will always fit in a u64"
        )]
        #[allow(
            clippy::cast_sign_loss,
            reason = "We expect this score will always fit in a u64"
        )]
        tx.execute(
            "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET
             noted_count = noted_count + excluded.noted_count,
             last_noted_timestamp = excluded.last_noted_timestamp",
            rusqlite::params![entry.filename, entry.count, entry.timestamp],
        )
        .map_err(|e| format!("Failed to insert or update entry into database: {e}"))?;
        debug!("Imported entry for file {}", entry.filename);
    }

    tx.commit().expect("Cannot commit import transaction");

    Ok(())
}

pub fn process_file<F>(
    file_path: &str,
    conn: &mut Connection,
    parser: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&str) -> Result<Vec<MemyEntry>, Box<dyn Error>>,
{
    info!("Importing from database {file_path}...");

    let contents = fs::read_to_string(file_path)?;
    let entries = parser(&contents)?;

    insert_into_db(conn, entries)?;

    info!("Imported database {file_path}");

    Ok(())
}

pub fn process_fasd_file(file_path: &str, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    process_file(file_path, conn, parse_fasd_state)
}

pub fn process_autojump_file(file_path: &str, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    process_file(file_path, conn, parse_autojump_state)
}

pub fn process_zoxide_query(conn: &mut Connection) {
    let Ok(output) = std::process::Command::new("zoxide")
        .args(["query", "--list", "--all", "--score"])
        .output()
    else {
        return;
    };

    if !output.status.success() {
        return;
    }

    let Ok(stdout) = String::from_utf8(output.stdout) else {
        return;
    };

    let Ok(entries) = parse_zoxide_state(&stdout) else {
        return;
    };

    let Ok(()) = insert_into_db(conn, entries) else {
        return;
    };

    info!("Imported zoxide state");
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    #[allow(clippy::float_cmp, reason = "Exact comparisons are desirable here")]
    fn test_parse_fasd_state_valid_input() {
        let input = "file1.txt|10.5|1633036800\nfile2.txt|20.0|1633123200";
        let result = parse_fasd_state(input).expect("Couldn't parse fasd state");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].filename, "file1.txt");
        assert_eq!(result[0].count, 11);
        assert_eq!(result[0].timestamp, 1_633_036_800);
        assert_eq!(result[1].filename, "file2.txt");
        assert_eq!(result[1].count, 20);
        assert_eq!(result[1].timestamp, 1_633_123_200);
    }

    #[test]
    fn test_parse_fasd_state_missing_fields() {
        let input = "file1.txt|10.5";
        let result = parse_fasd_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fasd_state_invalid_score() {
        let input = "file1.txt|invalid_score|1633036800";
        let result = parse_fasd_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fasd_state_negative_score() {
        let input = "file1.txt|-5|1633036800";
        let result = parse_fasd_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fasd_state_invalid_timestamp() {
        let input = "file1.txt|10.5|invalid_timestamp";
        let result = parse_fasd_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_fasd_state_negative_timestamp() {
        let input = "file1.txt|10.5|-5";
        let result = parse_fasd_state(input).expect("Couldn't parse fasd state");

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].filename, "file1.txt");
        assert_eq!(result[0].count, 11);
        assert_eq!(result[0].timestamp, -5);
    }

    #[test]
    #[allow(clippy::float_cmp, reason = "Exact comparisons are desirable here")]
    fn test_parse_zoxide_state_valid_input() {
        let input = "   12.0    /home/user/docs\n2.0 /tmp\n0.5 /home/user/.local/share";
        let result = parse_zoxide_state(input).expect("Couldn't parse zoxide state");

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].filename, "/home/user/docs");
        assert_eq!(result[0].count, 12);
        assert!(result[0].timestamp > 0);
        assert_eq!(result[1].filename, "/tmp");
        assert_eq!(result[1].count, 2);
        assert!(result[1].timestamp > 0);
        assert_eq!(result[2].filename, "/home/user/.local/share");
        assert_eq!(result[2].count, 1);
        assert!(result[2].timestamp > 0);
    }

    #[test]
    fn test_parse_zoxide_state_missing_fields() {
        let input = "12.0";
        let result = parse_zoxide_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zoxide_state_invalid_count() {
        let input = "invalid_count /home/docs/paperwork";
        let result = parse_zoxide_state(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_zoxide_state_negative_count() {
        let input = "-12.0 /home/docs";
        let result = parse_zoxide_state(input);

        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn prop_parse_fasd_str_valid_entry(
            filename in "[^|\\n\\r]{1,50}",
            score in 0u64..=10_000u64,
            timestamp in 0i64..=i64::MAX,
        ) {
            let input = format!("{filename}|{score}.0|{timestamp}");
            let result = from_fasd_str(&input).expect("valid fasd entry should parse");
            prop_assert_eq!(&result.filename, &filename);
            prop_assert_eq!(result.count, score, "count should equal rounded score");
            prop_assert_eq!(result.timestamp, timestamp);
        }

        #[test]
        fn prop_parse_whitespace_str_valid_entry(
            count in 0u64..=10_000u64,
            path in "/[a-z/]{1,50}",
        ) {
            let input = format!("{count}.0 {path}");
            let result = from_whitespace_split_str(&input).expect("valid whitespace entry should parse");
            prop_assert_eq!(&result.filename, &path);
            prop_assert_eq!(result.count, count, "count should equal rounded score");
            prop_assert!(result.timestamp > 0, "timestamp should be positive");
        }
    }
}
