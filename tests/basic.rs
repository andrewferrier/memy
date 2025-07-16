mod support;
use support::*;

use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn test_note_and_list_paths() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 1, false);
    sleep(1000);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &[]);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], dir_a.to_str().unwrap());
    assert_eq!(lines[1], dir_b.to_str().unwrap());
}

#[test]
fn test_note_and_list_paths_with_scores() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 1, false);
    sleep(1000);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &["--include-frecency-score"]);

    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(dir_a.to_str().unwrap()));
    assert!(lines[1].starts_with(dir_b.to_str().unwrap()));

    let a_score: f64 = lines[0]
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .expect("score parse");
    let b_score: f64 = lines[1]
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .expect("score parse");

    assert!(b_score >= a_score);
}

#[test]
fn test_note_relative_path() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_name = "rel_test_file";
    let test_file_path = create_test_file(&working_path, test_file_name, "test content");

    let orig_dir = std::env::current_dir().expect("failed to get current dir");
    std::env::set_current_dir(&working_path).expect("failed to change dir");

    note_path(&db_path, None, test_file_name, 1, false);

    std::env::set_current_dir(orig_dir).expect("failed to restore dir");

    let lines = list_paths(&db_path, None, &[]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_frecency_ordering() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");
    let dir_c = create_test_directory(&working_path, "dir_c");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 10, false);
    sleep(500);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, false);
    sleep(500);
    note_path(&db_path, None, dir_c.to_str().unwrap(), 10, false);

    let lines = list_paths(&db_path, None, &[]);

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], dir_b.to_str().unwrap());
    assert_eq!(lines[1], dir_a.to_str().unwrap());
    assert_eq!(lines[2], dir_c.to_str().unwrap());
}

#[test]
fn test_frecency_ordering_with_scores() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");
    let dir_c = create_test_directory(&working_path, "dir_c");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 10, false);
    sleep(500);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, false);
    sleep(500);
    note_path(&db_path, None, dir_c.to_str().unwrap(), 10, false);

    let lines = list_paths(&db_path, None, &["--include-frecency-score"]);

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with(dir_b.to_str().unwrap()));
    assert!(lines[1].starts_with(dir_a.to_str().unwrap()));
    assert!(lines[2].starts_with(dir_c.to_str().unwrap()));
}

#[test]
fn test_note_nonexistent_path() {
    let (_db_temp, db_path) = temp_dir();

    let test_path = "/this/path/definitely/does/not/exist";

    let output = memy_cmd(&db_path, None, &["--note", test_path])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(),);
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected_error = format!("Path {test_path} does not exist.");
    assert!(stderr.contains(&expected_error),);
}

#[test]
fn test_help_flag() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd(&db_path, None, &["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_note_symlink_resolves_to_target() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dummy_file_path = create_test_file(&working_path, "dummy_file_A", "dummy content");

    let symlink_path = working_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&db_path, None, symlink_path.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &[]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], dummy_file_path.to_str().unwrap());
}

#[test]
fn test_note_symlink_with_no_normalize_option() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dummy_file_path = create_test_file(&working_path, "dummy_file_A", "dummy content");

    let symlink_path = working_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&db_path, None, symlink_path.to_str().unwrap(), 1, true);

    let lines = list_paths(&db_path, None, &[]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], symlink_path.to_str().unwrap());
}

#[test]
fn test_note_deleted_file_not_in_list() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");

    note_path(&db_path, None, test_file_path.to_str().unwrap(), 1, false);

    let lines_before: Vec<String> = list_paths(&db_path, None, &[]);
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], test_file_path.to_str().unwrap());

    fs::remove_file(&test_file_path).expect("failed to delete test file");

    let lines_after: Vec<String> = list_paths(&db_path, None, &[]);
    assert_eq!(lines_after.len(), 0);
}

#[test]
fn test_files_only_flag() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");
    let test_dir = create_test_directory(&working_path, "test_dir");

    note_path(&db_path, None, test_file_path.to_str().unwrap(), 1, false);
    note_path(&db_path, None, test_dir.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &["--files-only"]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_directories_only_flag() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");
    let test_dir = create_test_directory(&working_path, "test_dir");

    note_path(&db_path, None, test_file_path.to_str().unwrap(), 1, false);
    note_path(&db_path, None, test_dir.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &["--directories-only"]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_dir.to_str().unwrap());
}
