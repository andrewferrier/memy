#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use core::time::Duration;
use std::thread::sleep;

#[test]
fn test_config_override_float() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 2, &[], &[]);
    sleep(Duration::from_secs(1));
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&db_path, None, &["--config", "recency_bias=0"], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_b"))
            < lines.iter().position(|line| line.contains("dir_a"))
    );
}

#[test]
fn test_config_override_float_2() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(&db_path, None, dir_a.to_str().unwrap(), 2, &[], &[]);
    sleep(Duration::from_secs(1));
    note_path(&db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&db_path, None, &["--config", "recency_bias=1"], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_a"))
            < lines.iter().position(|line| line.contains("dir_b")),
    );
}

#[test]
fn test_config_override_float_with_config_file() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let config_contents = "recency_bias=0\n";
    create_config_file(&config_path, config_contents);

    let dir_a = create_test_directory(&working_path, "dir_a");
    let dir_b = create_test_directory(&working_path, "dir_b");

    note_path(
        &db_path,
        Some(&config_path),
        dir_a.to_str().unwrap(),
        2,
        &[],
        &[],
    );
    sleep(Duration::from_secs(1));
    note_path(
        &db_path,
        Some(&config_path),
        dir_b.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines = list_paths(&db_path, None, &["--config", "recency_bias=1"], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_a"))
            < lines.iter().position(|line| line.contains("dir_b")),
    );
}

#[test]
fn test_config_boolean() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dummy_file_path = create_test_file(&working_path, "dummy_file_A", "dummy content");
    let symlink_path = working_path.join("symlink_B");
    std::os::unix::fs::symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(
        &db_path,
        None,
        symlink_path.to_str().unwrap(),
        1,
        &["--config", "normalize_symlinks_on_note=false"],
        &[],
    );

    let lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], symlink_path.to_str().unwrap());
}

#[test]
fn test_denylist_excludes_file_override_quoted_filenames() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let deny_file = create_test_file(&working_path, "denyme.txt", "deny me");

    let deny_pattern = deny_file.to_str().unwrap();

    let output = note_path(
        &db_path,
        None,
        deny_file.to_str().unwrap(),
        1,
        &["--config", &format!("denylist=['a.txt', '{deny_pattern}']")],
        &[],
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_excludes_file_override_quoted_value() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let deny_file = create_test_file(&working_path, "denyme.txt", "deny me");

    let deny_pattern = deny_file.to_str().unwrap();

    let output = note_path(
        &db_path,
        None,
        deny_file.to_str().unwrap(),
        1,
        &[
            "--config",
            &format!("denylist=\"['a.txt', '{deny_pattern}']\""),
        ],
        &[],
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_invalid_param() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd(&db_path, None, &["--config", "foo='bar'", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown field `foo`, expected one of"));
}

#[test]
fn test_incorrect_type() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd(
        &db_path,
        None,
        &["--config", "normalize_symlinks_on_note=100", "list"],
    )
    .output()
    .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid type: string"));
    assert!(stderr.contains("expected a boolean"));
}
