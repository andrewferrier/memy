#![allow(clippy::unwrap_used)]

mod support;
use support::*;

use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_recency_bias_0() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

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

    let config_contents = "recency_bias=0\n";
    create_config_file(&config_path, config_contents);

    let lines = list_paths(&db_path, Some(&config_path), &[], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_b"))
            < lines.iter().position(|line| line.contains("dir_a"))
    );
}

#[test]
fn test_recency_bias_1() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

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

    let config_contents = "recency_bias=1\n";
    create_config_file(&config_path, config_contents);

    let lines = list_paths(&db_path, Some(&config_path), &[], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_a"))
            < lines.iter().position(|line| line.contains("dir_b")),
    );
}

#[test]
fn test_recency_bias_below_0() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let config_contents = "recency_bias=-1\n";
    create_config_file(&config_path, config_contents);

    let output = memy_cmd(&db_path, Some(&config_path), &["list"])
        .output()
        .expect("Failed to execute command");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be between 0 and 1"),);
}

#[test]
fn test_recency_bias_above_1() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let config_contents = "recency_bias=1.5\n";
    create_config_file(&config_path, config_contents);

    let output = memy_cmd(&db_path, Some(&config_path), &["list"])
        .output()
        .expect("Failed to execute command");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be between 0 and 1"),);
}
