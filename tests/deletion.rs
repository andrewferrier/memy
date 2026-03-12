#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::fs;

#[test]
fn test_note_deleted_file_not_yet_expired() {
    let ctx = TestContext::new();

    let test_file_path = create_test_file(&ctx.working_path, "test_file", "test content");

    note_path(
        &ctx.db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines_before: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_before, &[test_file_path.to_str().unwrap()]);

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_delete, &[]);

    let test_file_path2 = create_test_file(&ctx.working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(
        &lines_after_recreation,
        &[test_file_path2.to_str().unwrap()],
    );
}

#[test]
fn test_not_deleted_file_still_present() {
    let ctx = TestContext::new();

    let test_file_path = create_test_file(&ctx.working_path, "test_file", "test content");

    note_path(
        &ctx.db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines_before_update_timestamp: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(
        &lines_before_update_timestamp,
        &[test_file_path.to_str().unwrap()],
    );

    age_all_paths(&ctx.db_path, 45);

    let lines_after_update_timestamp: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(
        &lines_after_update_timestamp,
        &[test_file_path.to_str().unwrap()],
    );
}

#[test]
fn test_note_deleted_file_fake_expiry() {
    let ctx = TestContext::new();

    let test_file_path = create_test_file(&ctx.working_path, "test_file", "test content");

    note_path(
        &ctx.db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    age_all_paths(&ctx.db_path, 45);

    let lines_before: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_before, &[test_file_path.to_str().unwrap()]);

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_delete, &[]);

    _ = create_test_file(&ctx.working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_recreation.len(), 0);
}

#[test]
fn test_note_deleted_file_fake_expiry_space_in_filename() {
    let ctx = TestContext::new();

    let test_file_path = create_test_file(&ctx.working_path, "test file", "test content");

    note_path(
        &ctx.db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    age_all_paths(&ctx.db_path, 45);

    let lines_before: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_before, &[test_file_path.to_str().unwrap()]);

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_delete, &[]);

    _ = create_test_file(&ctx.working_path, "test file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_recreation, &[]);
}

#[test]
fn test_note_multiple_deleted_files_fake_expiry_one_retained() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b", "content b");
    let file_c = create_test_file(&ctx.working_path, "file_c", "content c");

    for path in [&file_a, &file_b, &file_c] {
        note_path(&ctx.db_path, None, path.to_str().unwrap(), 1, &[], &[]);
    }

    age_all_paths(&ctx.db_path, 45);

    let lines_before: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 3);

    fs::remove_file(&file_a).expect("failed to delete file_a");
    fs::remove_file(&file_b).expect("failed to delete file_b");

    let lines_after_delete: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_delete, &[file_c.to_str().unwrap()]);
}

#[test]
fn test_note_deleted_file_not_quite_expired() {
    let ctx = TestContext::new();

    let test_file_path = create_test_file(&ctx.working_path, "test_file", "test content");

    note_path(
        &ctx.db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    age_all_paths(&ctx.db_path, 29);

    let lines_before: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_before, &[test_file_path.to_str().unwrap()]);

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(&lines_after_delete, &[]);

    let test_file_path2 = create_test_file(&ctx.working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&ctx.db_path, None, &["-vvv"], &[]);
    assert_lines_eq(
        &lines_after_recreation,
        &[test_file_path2.to_str().unwrap()],
    );
}
