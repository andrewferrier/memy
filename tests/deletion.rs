#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::fs;

#[test]
fn test_note_deleted_file_not_yet_expired() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines_before: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], test_file_path.to_str().unwrap());

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_delete.len(), 0);

    let test_file_path2 = create_test_file(&working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_recreation.len(), 1);
    assert_eq!(lines_after_recreation[0], test_file_path2.to_str().unwrap());
}

#[test]
fn test_not_deleted_file_still_present() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines_before_update_timestamp: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before_update_timestamp.len(), 1);
    assert_eq!(
        lines_before_update_timestamp[0],
        test_file_path.to_str().unwrap()
    );

    execute_sql(
        &db_path,
        "UPDATE paths SET last_noted_timestamp = strftime('%s', 'now') - (45 * 24 * 60 * 60);",
    );

    let lines_after_update_timestamp: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_update_timestamp.len(), 1);
    assert_eq!(
        lines_after_update_timestamp[0],
        test_file_path.to_str().unwrap()
    );
}

#[test]
fn test_note_deleted_file_fake_expiry() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    execute_sql(
        &db_path,
        "UPDATE paths SET last_noted_timestamp = strftime('%s', 'now') - (45 * 24 * 60 * 60);",
    );

    let lines_before: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], test_file_path.to_str().unwrap());

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_delete.len(), 0);

    _ = create_test_file(&working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_recreation.len(), 0);
}

#[test]
fn test_note_deleted_file_fake_expiry_space_in_filename() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test file", "test content");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    execute_sql(
        &db_path,
        "UPDATE paths SET last_noted_timestamp = strftime('%s', 'now') - (45 * 24 * 60 * 60);",
    );

    let lines_before: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], test_file_path.to_str().unwrap());

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_delete.len(), 0);

    _ = create_test_file(&working_path, "test file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_recreation.len(), 0);
}

#[test]
fn test_note_multiple_deleted_files_fake_expiry_one_retained() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let file_a = create_test_file(&working_path, "file_a", "content a");
    let file_b = create_test_file(&working_path, "file_b", "content b");
    let file_c = create_test_file(&working_path, "file_c", "content c");

    for path in [&file_a, &file_b, &file_c] {
        note_path(&db_path, None, path.to_str().unwrap(), 1, &[], &[]);
    }

    execute_sql(
        &db_path,
        "UPDATE paths SET last_noted_timestamp = strftime('%s', 'now') - (45 * 24 * 60 * 60);",
    );

    let lines_before: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 3);

    fs::remove_file(&file_a).expect("failed to delete file_a");
    fs::remove_file(&file_b).expect("failed to delete file_b");

    let lines_after_delete: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_delete.len(), 1);
    assert_eq!(lines_after_delete[0], file_c.to_str().unwrap());
}

#[test]
fn test_note_deleted_file_not_quite_expired() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    execute_sql(
        &db_path,
        "UPDATE paths SET last_noted_timestamp = strftime('%s', 'now') - (29 * 24 * 60 * 60);",
    );

    let lines_before: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], test_file_path.to_str().unwrap());

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after_delete: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_delete.len(), 0);

    let test_file_path2 = create_test_file(&working_path, "test_file", "test content");

    let lines_after_recreation: Vec<String> = list_paths(&db_path, None, &["-vvv"], &[]);
    assert_eq!(lines_after_recreation.len(), 1);
    assert_eq!(lines_after_recreation[0], test_file_path2.to_str().unwrap());
}
