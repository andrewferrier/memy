#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#![allow(clippy::missing_panics_doc, reason = "Missing docs OK inside tests")]
#![allow(
    clippy::must_use_candidate,
    reason = "Missing annotation OK inside tests"
)]
#![allow(dead_code, reason = "Dead code false +ves inside support.rs")]

use assert_cmd::Command;
use core::time::Duration;
use std::fs;
use std::path::PathBuf;
use std::thread;
use tempfile::TempDir;

pub fn temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir
        .path()
        .canonicalize()
        .expect("failed to canonicalize temp dir path");
    (temp_dir, path)
}

pub fn create_test_file(
    dir: &std::path::Path,
    filename: &str,
    contents: &str,
) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, contents).expect("failed to create test file");
    file_path
}

pub fn create_config_file(config_path: &std::path::Path, contents: &str) {
    create_test_file(config_path, "memy.toml", contents);
}

pub fn create_test_directory(base: &std::path::Path, dirname: &str) -> std::path::PathBuf {
    let dir_path = base.join(dirname);
    std::fs::create_dir(&dir_path).expect("failed to create test directory");
    dir_path
}

pub fn memy_cmd(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    args: &[&str],
) -> Command {
    let mut cmd = Command::cargo_bin("memy").expect("Cannot set up memy command");
    cmd.env("MEMY_DB_DIR", db_path);

    if let Some(config) = config_path {
        cmd.env("MEMY_CONFIG_DIR", config);
    } else {
        let (_temp_dir, temp_path) = temp_dir();
        cmd.env("MEMY_CONFIG_DIR", &temp_path);
    }

    cmd.args(args);
    cmd
}

pub fn memy_cmd_test_defaults(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    original_args: &[&str],
) -> Command {
    let mut args = Vec::new();
    args.push("--config");
    args.push("import_on_first_use=false");
    args.extend(original_args);

    memy_cmd(db_path, config_path, &args)
}

pub fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

pub fn execute_sql(db_path: &std::path::Path, sql: &str) {
    let db_file = db_path.join("memy.sqlite3");
    let connection = rusqlite::Connection::open(db_file).expect("failed to open database");
    connection
        .execute(sql, [])
        .expect("failed to execute SQL command");
}

// TODO: Consider changing default to return Output
pub fn note_path(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    path: &str,
    count: usize,
    common_args: &[&str],
    note_args: &[&str],
) -> assert_cmd::Command {
    let mut last_cmd = None;

    for _ in 0..count {
        let mut args = Vec::new();
        args.extend(common_args);
        args.push("note");
        args.extend(note_args);
        args.push(path);

        let mut cmd = memy_cmd_test_defaults(db_path, config_path, &args);
        cmd.assert().success();
        last_cmd = Some(cmd);
        sleep(100);
    }

    last_cmd.expect("note_path called with count == 0")
}

pub fn list_paths(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    common_args: &[&str],
    list_args: &[&str],
) -> Vec<String> {
    let mut args = Vec::new();
    args.extend(common_args);
    args.push("list");
    args.extend(list_args);

    let output = memy_cmd_test_defaults(db_path, config_path, &args)
        .output()
        .expect("failed to run memy");

    assert!(output.status.success(), "memy list failed.");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}
