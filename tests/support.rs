#![allow(clippy::missing_panics_doc)]
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[must_use]
pub fn temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir
        .path()
        .canonicalize()
        .expect("failed to canonicalize temp dir path");
    (temp_dir, path)
}

#[cfg_attr(test, allow(dead_code))]
pub fn create_config_file(config_path: &std::path::Path, contents: &str) {
    let config_toml_path = config_path.join("memy.toml");
    fs::write(&config_toml_path, contents).expect("failed to write config");
}

#[cfg_attr(test, allow(dead_code))]
#[must_use]
pub fn create_test_file(
    dir: &std::path::Path,
    filename: &str,
    contents: &str,
) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, contents).expect("failed to create test file");
    file_path
}

#[cfg_attr(test, allow(dead_code))]
#[must_use]
pub fn create_test_directory(base: &std::path::Path, dirname: &str) -> std::path::PathBuf {
    let dir_path = base.join(dirname);
    std::fs::create_dir(&dir_path).expect("failed to create test directory");
    dir_path
}

#[must_use]
pub fn memy_cmd(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    args: &[&str],
) -> Command {
    let mut cmd = Command::cargo_bin("memy").unwrap();
    cmd.env("MEMY_DB_DIR", db_path);

    let _temp_config_dir;
    if let Some(config) = config_path {
        cmd.env("MEMY_CONFIG_DIR", config);
    } else {
        let (temp_dir, temp_path) = temp_dir();
        _temp_config_dir = temp_dir;
        cmd.env("MEMY_CONFIG_DIR", &temp_path);
    }

    cmd.args(args);
    cmd
}

#[cfg_attr(test, allow(dead_code))]
pub fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

#[cfg_attr(test, allow(dead_code))]
pub fn note_path(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    path: &str,
    count: usize,
    no_normalize_symlinks: bool,
) {
    for _ in 0..count {
        let mut args = vec!["note", path];
        if no_normalize_symlinks {
            args.push("--no-normalize-symlinks");
        }

        memy_cmd(db_path, config_path, &args).assert().success();
        sleep(100);
    }
}

pub fn list_paths(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    flags: &[&str],
) -> Vec<String> {
    let mut args = vec!["list"];
    args.extend(flags);

    let output = memy_cmd(db_path, config_path, &args)
        .output()
        .expect("failed to run memy");

    assert!(output.status.success());

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}
