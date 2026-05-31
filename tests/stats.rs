#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use serde_json::Value;

#[test]
fn test_memy_stats_plain_format() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
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
fn test_memy_stats_plain_shows_file_and_dir_counts() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    let test_dir = create_test_directory(&ctx.working_path, "test_dir");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, test_dir.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        stdout.contains("Files: 1"),
        "Expected 'Files: 1' in output:\n{stdout}"
    );
    assert!(
        stdout.contains("Directories: 1"),
        "Expected 'Directories: 1' in output:\n{stdout}"
    );
}

#[test]
fn test_memy_stats_plain_shows_histogram() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        stdout.contains("Count Distribution"),
        "Expected histogram header in output:\n{stdout}"
    );
    assert!(
        stdout.contains('│'),
        "Expected bar chart separator in output:\n{stdout}"
    );
}

#[test]
fn test_memy_stats_plain_shows_time_chart_with_multiple_paths() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "a.txt", "a");
    let file_b = create_test_file(&ctx.working_path, "b.txt", "b");
    note_path(&ctx.db_path, None, file_a.to_str().unwrap(), 1, &[], &[]);
    sleep(200);
    note_path(&ctx.db_path, None, file_b.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        stdout.contains("Time Distribution"),
        "Expected time chart header in output:\n{stdout}"
    );
    // Column chart uses ┤ for its Y-axis
    assert!(
        stdout.contains('┤'),
        "Expected column chart Y-axis separator in output:\n{stdout}"
    );
}

#[test]
fn test_memy_stats_plain_missing_label() {
    let ctx = TestContext::new();

    // Create the file so it can be noted, then remove it to make it "missing"
    let missing_path = ctx.working_path.join("will_be_deleted.txt");
    create_test_file(&ctx.working_path, "will_be_deleted.txt", "temp");
    note_path(
        &ctx.db_path,
        None,
        missing_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );
    std::fs::remove_file(&missing_path).expect("Failed to remove test file");

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        stdout.contains("Missing:"),
        "Expected 'Missing:' label in output:\n{stdout}"
    );
    assert!(
        !stdout.contains("Missing/Other"),
        "Old 'Missing/Other' label must not appear:\n{stdout}"
    );
}

#[test]
fn test_memy_stats_plain_no_time_chart_with_single_path() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "solo.txt", "solo");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "plain"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        !stdout.contains("Time Distribution"),
        "Time chart should not appear with only one path:\n{stdout}"
    );
}

#[test]
fn test_memy_stats_json_format() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "json"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    let parsed_json: Result<Value, _> = serde_json::from_str(&stdout);
    assert!(parsed_json.is_ok(), "Output is not valid JSON");
}

#[test]
fn test_memy_stats_json_includes_file_dir_counts() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    let test_dir = create_test_directory(&ctx.working_path, "test_dir");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, test_dir.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "json"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    let json: Value = serde_json::from_str(&stdout).expect("Output is not valid JSON");

    assert_eq!(
        json["files_count"].as_u64(),
        Some(1),
        "Expected files_count=1 in JSON"
    );
    assert_eq!(
        json["dirs_count"].as_u64(),
        Some(1),
        "Expected dirs_count=1 in JSON"
    );
    assert!(
        json["missing_count"].is_number(),
        "Expected missing_count field in JSON"
    );
}

#[test]
fn test_memy_stats_json_excludes_charts() {
    let ctx = TestContext::new();

    let test_file = create_test_file(&ctx.working_path, "test_note.txt", "Test content");
    note_path(&ctx.db_path, None, test_file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["stats", "--format", "json"],
        vec![],
    );
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    assert!(
        !stdout.contains("Count Distribution"),
        "JSON output should not contain histogram"
    );
    assert!(
        !stdout.contains("Time Distribution"),
        "JSON output should not contain time chart"
    );
}
