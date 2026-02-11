#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_newer_than_with_duration_filters_correctly() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    // Create three test files
    let file1 = create_test_file(&working_path, "file1.txt", "content1");
    let file2 = create_test_file(&working_path, "file2.txt", "content2");
    let file3 = create_test_file(&working_path, "file3.txt", "content3");

    // Note file1
    note_path(&db_path, None, file1.to_str().unwrap(), 1, &[], &[]);
    sleep(200);

    // Note file2
    note_path(&db_path, None, file2.to_str().unwrap(), 1, &[], &[]);
    sleep(200);

    // Note file3
    note_path(&db_path, None, file3.to_str().unwrap(), 1, &[], &[]);

    // Manually update timestamps in database to simulate different noted times
    // Set file1 to 10 days ago
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = last_noted_timestamp - {} WHERE path = '{}'",
            10 * 24 * 3600,
            file1.to_str().unwrap()
        ),
    );

    // Set file2 to 2 days ago
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = last_noted_timestamp - {} WHERE path = '{}'",
            2 * 24 * 3600,
            file2.to_str().unwrap()
        ),
    );

    // file3 remains with current timestamp

    // List all files without filter
    let all_lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(all_lines.len(), 3, "Should have 3 files total");

    // List files newer than 5 days
    let newer_5d_lines = list_paths(&db_path, None, &[], &["--newer-than", "5d"]);
    assert_eq!(newer_5d_lines.len(), 2, "Should have 2 files newer than 5 days");
    assert!(newer_5d_lines.iter().any(|s| s.contains("file2.txt")));
    assert!(newer_5d_lines.iter().any(|s| s.contains("file3.txt")));

    // List files newer than 1 day
    let newer_1d_lines = list_paths(&db_path, None, &[], &["--newer-than", "1d"]);
    assert_eq!(newer_1d_lines.len(), 1, "Should have 1 file newer than 1 day");
    assert!(newer_1d_lines.iter().any(|s| s.contains("file3.txt")));

    // List files newer than 3 hours
    let newer_3h_lines = list_paths(&db_path, None, &[], &["--newer-than", "3h"]);
    assert_eq!(newer_3h_lines.len(), 1, "Should have 1 file newer than 3 hours");
    assert!(newer_3h_lines.iter().any(|s| s.contains("file3.txt")));
}

#[test]
fn test_newer_than_with_iso8601_date() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let file1 = create_test_file(&working_path, "file1.txt", "content1");
    let file2 = create_test_file(&working_path, "file2.txt", "content2");

    // Note both files
    note_path(&db_path, None, file1.to_str().unwrap(), 1, &[], &[]);
    sleep(200);
    note_path(&db_path, None, file2.to_str().unwrap(), 1, &[], &[]);

    // Set file1 to be noted on 2020-01-01 (arbitrary old date)
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            1577836800, // 2020-01-01 00:00:00 UTC
            file1.to_str().unwrap()
        ),
    );

    // Set file2 to be noted on 2025-06-01 (more recent date)
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            1748736000, // 2025-06-01 00:00:00 UTC
            file2.to_str().unwrap()
        ),
    );

    // List files newer than 2024-01-01
    let newer_lines = list_paths(&db_path, None, &[], &["--newer-than", "2024-01-01"]);
    assert_eq!(newer_lines.len(), 1, "Should have 1 file newer than 2024-01-01");
    assert!(newer_lines.iter().any(|s| s.contains("file2.txt")));

    // List files newer than 2019-01-01
    let all_lines = list_paths(&db_path, None, &[], &["--newer-than", "2019-01-01"]);
    assert_eq!(all_lines.len(), 2, "Should have 2 files newer than 2019-01-01");
}

#[test]
fn test_newer_than_with_combined_duration() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let file1 = create_test_file(&working_path, "file1.txt", "content1");
    let file2 = create_test_file(&working_path, "file2.txt", "content2");

    note_path(&db_path, None, file1.to_str().unwrap(), 1, &[], &[]);
    sleep(200);
    note_path(&db_path, None, file2.to_str().unwrap(), 1, &[], &[]);

    // Set file1 to 2 days and 5 hours ago
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = last_noted_timestamp - {} WHERE path = '{}'",
            (2 * 24 * 3600) + (5 * 3600),
            file1.to_str().unwrap()
        ),
    );

    // file2 remains with current timestamp

    // List files newer than 2 days 4 hours (should include file2 only)
    let newer_lines = list_paths(&db_path, None, &[], &["--newer-than", "2d4h"]);
    assert_eq!(newer_lines.len(), 1, "Should have 1 file newer than 2d4h");
    assert!(newer_lines.iter().any(|s| s.contains("file2.txt")));
}

#[test]
fn test_newer_than_invalid_format() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let file1 = create_test_file(&working_path, "file1.txt", "content1");
    note_path(&db_path, None, file1.to_str().unwrap(), 1, &[], &[]);

    // Try to list with invalid format
    let mut args = Vec::new();
    args.push("--config");
    args.push("import_on_first_use=false");
    args.push("list");
    args.push("--newer-than");
    args.push("invalid-format");

    let output = memy_cmd(Some(&db_path), None, &args, vec![]);
    assert!(!output.status.success(), "Should fail with invalid format");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to parse"));
}

#[test]
fn test_newer_than_no_results() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let file1 = create_test_file(&working_path, "file1.txt", "content1");
    note_path(&db_path, None, file1.to_str().unwrap(), 1, &[], &[]);

    // Set file1 to be very old
    execute_sql(
        &db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            946684800, // 2000-01-01 00:00:00 UTC
            file1.to_str().unwrap()
        ),
    );

    // List files newer than a very recent date
    let newer_lines = list_paths(&db_path, None, &[], &["--newer-than", "1h"]);
    assert_eq!(newer_lines.len(), 0, "Should have no files newer than 1 hour");
}
