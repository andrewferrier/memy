use core::str::FromStr;
use rusqlite::Connection;
use std::fs;

use crate::types::UnixTimestamp;

pub type FasdScore = f64;

#[derive(Debug)]
pub struct FasdEntry {
    pub filename: String,
    pub score: FasdScore,
    pub timestamp: UnixTimestamp,
}

impl FromStr for FasdEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid entry: {s}"));
        }

        let filename = parts[0].to_owned();
        let score = parts[1]
            .parse::<FasdScore>()
            .map_err(|e| format!("Invalid score: {e}"))?;

        if score < 0.0 {
            return Err(format!("Score cannot be negative: {score}"));
        }

        let timestamp = parts[2]
            .parse::<UnixTimestamp>()
            .map_err(|e| format!("Invalid timestamp: {e}"))?;

        Ok(Self {
            filename,
            score,
            timestamp,
        })
    }
}

fn parse_fasd_state(contents: &str) -> Result<Vec<FasdEntry>, String> {
    contents.lines().map(str::parse::<FasdEntry>).collect()
}

pub fn process_fasd_file(file_path: &str, conn: &Connection) -> Result<(), String> {
    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file {file_path}: {e}"))?;
    let entries = parse_fasd_state(&contents)?;

    for entry in entries {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "We expect this score will always fit in a u64"
        )]
        #[allow(
            clippy::cast_sign_loss,
            reason = "We expect this score will always fit in a u64"
        )]
        let rounded_score = entry.score.round() as u64;
        conn.execute(
            "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, ?2, ?3)
             ON CONFLICT(path) DO UPDATE SET
             noted_count = noted_count + excluded.noted_count,
             last_noted_timestamp = excluded.last_noted_timestamp",
            [
                &entry.filename,
                &rounded_score.to_string(),
                &entry.timestamp.to_string(),
            ],
        )
        .map_err(|e| format!("Failed to insert or update entry into database: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp, reason = "Exact comparisons are desirable here")]
    fn test_parse_fasd_state_valid_input() {
        let input = "file1.txt|10.5|1633036800\nfile2.txt|20.0|1633123200";
        let result = parse_fasd_state(input).expect("Couldn't parse fasd state");

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].filename, "file1.txt");
        assert_eq!(result[0].score, 10.5);
        assert_eq!(result[0].timestamp, 1_633_036_800);
        assert_eq!(result[1].filename, "file2.txt");
        assert_eq!(result[1].score, 20.0);
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
        let result = parse_fasd_state(input);

        assert!(result.is_err());
    }
}
