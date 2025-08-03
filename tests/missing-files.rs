#![allow(clippy::unwrap_used)]

mod support;
use support::*;

#[test]
fn test_note_nonexistent_file_ignore_silently() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp, config_path) = temp_dir();
    let config_contents = "missing_files_warn_on_note = false\n";
    create_config_file(&config_path, config_contents);

    let test_path = "/tmp/this_file_should_not_exist_ignore_silently";
    let output = memy_cmd(&db_path, Some(&config_path), &["note", test_path])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
    let lines = list_paths(&db_path, Some(&config_path), &[]);
    assert!(lines.is_empty());
}

#[test]
fn test_note_nonexistent_file_ignore_with_warning() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp, config_path) = temp_dir();
    let config_contents = "missing_files_warn_on_note = true\n";
    create_config_file(&config_path, config_contents);

    let test_path = "/tmp/this_file_should_not_exist_ignore_with_warning";
    let output = memy_cmd(&db_path, Some(&config_path), &["note", test_path])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
    let lines = list_paths(&db_path, Some(&config_path), &[]);
    assert!(lines.is_empty());
}
