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
use std::process::Output;
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
    db_path: Option<&std::path::Path>,
    config_path: Option<&std::path::Path>,
    args: &[&str],
    env_vars: Vec<(&str, &str)>,
) -> Output {
    #[allow(
        clippy::collection_is_never_read,
        reason = "Keeping these dirs in scope stops them being deleted"
    )]
    let mut _temp_dir_db: Option<TempDir> = None;
    #[allow(
        clippy::collection_is_never_read,
        reason = "Keeping these dirs in scope stops them being deleted"
    )]
    let mut _temp_dir_config: Option<TempDir> = None;

    let mut cmd = Command::cargo_bin("memy").expect("Cannot set up memy command");

    if let Some(db) = db_path {
        cmd.env("MEMY_DB_DIR", db);
    } else {
        let (temp_dir_db, temp_path_db) = temp_dir();
        cmd.env("MEMY_DB_DIR", &temp_path_db);
        _temp_dir_db = Some(temp_dir_db);
    }

    if let Some(config) = config_path {
        cmd.env("MEMY_CONFIG_DIR", config);
    } else {
        let (temp_dir_config, temp_path_config) = temp_dir();
        cmd.env("MEMY_CONFIG_DIR", &temp_path_config);
        _temp_dir_config = Some(temp_dir_config);
    }

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    cmd.args(args);

    cmd.output().expect("Could not run memy")
}

pub fn memy_cmd_test_defaults(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    original_args: &[&str],
) -> Output {
    let mut args = Vec::new();
    args.push("--config");
    args.push("import_on_first_use=false");
    args.extend(original_args);

    memy_cmd(Some(db_path), config_path, &args, vec![])
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

pub fn note_path(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    path: &str,
    count: usize,
    common_args: &[&str],
    note_args: &[&str],
) -> Output {
    assert!(count != 0, "Test somehow asked for note_path count==0");

    let mut last_output = None;

    for _ in 0..count {
        let mut args = Vec::new();
        args.extend(common_args);
        args.push("note");
        args.extend(note_args);
        args.push(path);

        let output = memy_cmd_test_defaults(db_path, config_path, &args);
        assert!(output.status.success(), "Should always be successful");
        last_output = Some(output);

        sleep(100);
    }

    last_output.unwrap()
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

    let output = memy_cmd_test_defaults(db_path, config_path, &args);
    assert!(output.status.success(), "Should always be successful");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}
