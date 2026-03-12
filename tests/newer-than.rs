#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#![allow(clippy::similar_names, reason = "deliberately similar variable names")]

mod support;
use support::*;

#[test]
fn test_newer_than_with_duration_filters_correctly() {
    let ctx = TestContext::new();

    let file1 = create_test_file(&ctx.working_path, "file1.txt", "content1");
    let file2 = create_test_file(&ctx.working_path, "file2.txt", "content2");
    let file3 = create_test_file(&ctx.working_path, "file3.txt", "content3");

    note_paths_with_delay(&ctx.db_path, None, &[&file1, &file2, &file3]);

    // Manually update timestamps in database to simulate different noted times
    age_path_by(&ctx.db_path, &file1, 10 * 24 * 3600);
    age_path_by(&ctx.db_path, &file2, 2 * 24 * 3600);
    // file3 remains with current timestamp

    let all_lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_eq!(all_lines.len(), 3, "Should have 3 files total");

    let newer_5d_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "5d"]);
    assert_eq!(
        newer_5d_lines.len(),
        2,
        "Should have 2 files newer than 5 days"
    );
    assert!(newer_5d_lines.iter().any(|s| s.contains("file2.txt")));
    assert!(newer_5d_lines.iter().any(|s| s.contains("file3.txt")));

    let newer_1d_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "1d"]);
    assert_eq!(
        newer_1d_lines.len(),
        1,
        "Should have 1 file newer than 1 day"
    );
    assert!(newer_1d_lines.iter().any(|s| s.contains("file3.txt")));

    let newer_3h_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "3h"]);
    assert_eq!(
        newer_3h_lines.len(),
        1,
        "Should have 1 file newer than 3 hours"
    );
    assert!(newer_3h_lines.iter().any(|s| s.contains("file3.txt")));
}

#[test]
fn test_newer_than_with_iso8601_date() {
    let ctx = TestContext::new();

    let file1 = create_test_file(&ctx.working_path, "file1.txt", "content1");
    let file2 = create_test_file(&ctx.working_path, "file2.txt", "content2");

    note_paths_with_delay(&ctx.db_path, None, &[&file1, &file2]);

    execute_sql(
        &ctx.db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            1577836800, // 2020-01-01 00:00:00 UTC
            file1.to_str().unwrap()
        ),
    );

    execute_sql(
        &ctx.db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            1748736000, // 2025-06-01 00:00:00 UTC
            file2.to_str().unwrap()
        ),
    );

    let newer_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "2024-01-01"]);
    assert_eq!(
        newer_lines.len(),
        1,
        "Should have 1 file newer than 2024-01-01"
    );
    assert!(newer_lines.iter().any(|s| s.contains("file2.txt")));

    let all_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "2019-01-01"]);
    assert_eq!(
        all_lines.len(),
        2,
        "Should have 2 files newer than 2019-01-01"
    );
}

#[test]
fn test_newer_than_with_combined_duration() {
    let ctx = TestContext::new();

    let file1 = create_test_file(&ctx.working_path, "file1.txt", "content1");
    let file2 = create_test_file(&ctx.working_path, "file2.txt", "content2");

    note_paths_with_delay(&ctx.db_path, None, &[&file1, &file2]);

    // Set file1 to 2 days and 5 hours ago
    age_path_by(&ctx.db_path, &file1, (2 * 24 * 3600) + (5 * 3600));
    // file2 remains with current timestamp

    let newer_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "2d4h"]);
    assert_lines_eq(&newer_lines, &[file2.to_str().unwrap()]);
}

#[test]
fn test_newer_than_invalid_format() {
    let ctx = TestContext::new();

    let file1 = create_test_file(&ctx.working_path, "file1.txt", "content1");
    note_path(&ctx.db_path, None, file1.to_str().unwrap(), 1, &[], &[]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--newer-than",
        "invalid-format",
    ];

    let output = memy_cmd(Some(&ctx.db_path), None, &args, vec![]);
    assert!(!output.status.success(), "Should fail with invalid format");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unable to parse"));
}

#[test]
fn test_newer_than_no_results() {
    let ctx = TestContext::new();

    let file1 = create_test_file(&ctx.working_path, "file1.txt", "content1");
    note_path(&ctx.db_path, None, file1.to_str().unwrap(), 1, &[], &[]);

    execute_sql(
        &ctx.db_path,
        &format!(
            "UPDATE paths SET last_noted_timestamp = {} WHERE path = '{}'",
            946684800, // 2000-01-01 00:00:00 UTC
            file1.to_str().unwrap()
        ),
    );

    let newer_lines = list_paths(&ctx.db_path, None, &[], &["--newer-than", "1h"]);
    assert_eq!(
        newer_lines.len(),
        0,
        "Should have no files newer than 1 hour"
    );
}
