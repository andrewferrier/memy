#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

use serde_json::Value;
use std::path::Path;

mod support;
use support::*;

use std::env::home_dir;

#[test]
fn test_note_and_list_paths() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    sleep(1000);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&db_path, None, &[], &[]);

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], dir_a.to_str().unwrap());
    assert_eq!(lines[1], dir_b.to_str().unwrap());
}

#[test]
fn test_note_homedir() {
    let (_db_temp, db_path) = temp_dir();

    note_path(&db_path, None, "~", 1, &[], &[]);

    let lines = list_paths(&db_path, None, &[], &[]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], home_dir().unwrap().to_str().unwrap());
}

#[test]
fn test_note_and_list_paths_multiarg() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    // Note both paths in a single memy_cmd call
    let output = memy_cmd_test_defaults(
        &db_path,
        None,
        &["note", dir_a.to_str().unwrap(), dir_b.to_str().unwrap()],
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 2);
    let paths: Vec<&str> = vec![dir_a.to_str().unwrap(), dir_b.to_str().unwrap()];
    for path in paths {
        assert!(lines.contains(&path.to_owned()), "Missing path: {path}");
    }
}

#[test]
fn test_note_relative_path() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_name = "rel_test_file";
    let test_file_path = create_test_file(&working_path, test_file_name, "test content");

    let orig_dir = std::env::current_dir().expect("failed to get current dir");
    std::env::set_current_dir(&working_path).expect("failed to change dir");

    note_path(&db_path, None, test_file_name, 1, &[], &[]);

    std::env::set_current_dir(orig_dir).expect("failed to restore dir");

    let lines = list_paths(&db_path, None, &[], &[]);
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

    note_path(&db_path, None, dir_a.to_str().unwrap(), 10, &[], &[]);
    sleep(500);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);
    sleep(500);
    note_path(&db_path, None, dir_c.to_str().unwrap(), 10, &[], &[]);

    let lines = list_paths(&db_path, None, &[], &[]);

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], dir_b.to_str().unwrap());
    assert_eq!(lines[1], dir_a.to_str().unwrap());
    assert_eq!(lines[2], dir_c.to_str().unwrap());
}

#[test]
fn test_list_json_format() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd_test_defaults(&db_path, None, &["list", "--format=json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    let json: Value = serde_json::from_str(&stdout).expect("Output is not valid JSON");

    assert!(json.is_array(), "Output JSON is not an array");
    let array = json.as_array().unwrap();

    for item in array {
        assert!(item.is_object(), "Array item is not an object");
        let obj = item.as_object().unwrap();
        assert!(obj.contains_key("path"), "Object missing 'path' field");
        assert!(
            obj.contains_key("frecency"),
            "Object missing 'frecency' field"
        );
        assert!(obj.contains_key("count"), "Object missing 'count' field");
        assert!(
            obj.contains_key("last_noted"),
            "Object missing 'last_noted' field"
        );
    }
}

#[test]
fn test_list_csv_format() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd_test_defaults(&db_path, None, &["list", "--format=csv"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    let lines: Vec<&str> = stdout.lines().collect();

    assert!(!lines.is_empty(), "CSV output is empty");

    for line in lines {
        let fields = line.split(',').count();
        assert_eq!(fields, 4, "CSV line does not have 4 fields");
    }
}

#[test]
fn test_help_flag() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd_test_defaults(&db_path, None, &["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_files_only_flag() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");
    let test_dir = create_test_directory(&working_path, "test_dir");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );
    note_path(&db_path, None, test_dir.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&db_path, None, &[], &["--files-only"]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_directories_only_flag() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path = create_test_file(&working_path, "test_file", "test content");
    let test_dir = create_test_directory(&working_path, "test_dir");

    note_path(
        &db_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );
    note_path(&db_path, None, test_dir.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&db_path, None, &[], &["--directories-only"]);

    assert_eq!(lines[0], test_dir.to_str().unwrap());
}

#[test]
fn test_graceful_when_db_missing() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd_test_defaults(&db_path, None, &["list"])
        .output()
        .expect("Failed to run memy");

    assert!(output.status.success());
}

#[test]
fn test_graceful_when_dbdir_missing() {
    let output = memy_cmd_test_defaults(Path::new("/tmp/definitelydoesntexist"), None, &["list"])
        .output()
        .expect("Failed to run memy");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Database path /tmp/definitelydoesntexist doesn't exist"));
}

#[test]
fn test_graceful_when_configdir_missing() {
    // If the config path doesn't exist we just silently ignore it, it's the user's responsibility
    // to make sure the config file is there.

    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd_test_defaults(
        &db_path,
        Some(Path::new("/tmp/definitelydoesntexist")),
        &["list"],
    )
    .output()
    .expect("Failed to run memy");
    assert!(output.status.success());
}
