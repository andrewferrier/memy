#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use serde_json::Value;

#[test]
fn test_memy_stats_plain_format() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file = create_test_file(&working_path, "test_note.txt", "Test content");
    note_path(&db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(stdout.contains("Total Paths: 1"));
    assert!(stdout.contains("Newest Note"));
    assert!(stdout.contains("Oldest Note"));
    assert!(stdout.contains("Highest Count"));
}

#[test]
fn test_memy_stats_json_format() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file = create_test_file(&working_path, "test_note.txt", "Test content");
    note_path(&db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(Some(&db_path), None, &["stats", "--format", "json"], vec![]);
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    let parsed_json: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(parsed_json.is_ok(), "Output is not valid JSON");
}
