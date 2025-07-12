use assert_cmd::Command;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

fn memy_cmd(cache_path: &std::path::Path, args: &[&str]) -> Command {
    let mut cmd = Command::cargo_bin("memy").unwrap();
    cmd.env("XDG_CACHE_HOME", cache_path);
    cmd.args(args);
    cmd
}

fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn note_path(cache_path: &std::path::Path, path: &str, count: usize, no_normalize_symlinks: bool) {
    for _ in 0..count {
        let mut args = vec!["--note"];
        args.push(path);

        if no_normalize_symlinks {
            args.push("--no-normalize-symlinks");
        }

        memy_cmd(cache_path, &args).assert().success();
        sleep(100);
    }
}

fn list_paths(cache_path: &std::path::Path, flags: &[&str]) -> String {
    let mut args = vec!["--list"];
    args.extend(flags);
    let output = memy_cmd(cache_path, &args)
        .output()
        .expect("failed to run memy");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_note_and_list_paths() {
    let (_temp_dir, cache_path) = temp_dir();

    note_path(&cache_path, "/tmp", 1, false);
    sleep(1000);
    note_path(&cache_path, "/usr", 1, false);

    let stdout = list_paths(&cache_path, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "/tmp");
    assert_eq!(lines[1], "/usr");
}

#[test]
fn test_note_and_list_paths_with_scores() {
    let (_temp_dir, cache_path) = temp_dir();

    note_path(&cache_path, "/tmp", 1, false);
    sleep(1000);
    note_path(&cache_path, "/usr", 1, false);

    let stdout = list_paths(&cache_path, &["--include-frecency-score"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("/tmp"));
    assert!(lines[1].starts_with("/usr"));

    let tmp_score: f64 = lines[0]
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .expect("score parse");
    let usr_score: f64 = lines[1]
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .expect("score parse");

    assert!(usr_score >= tmp_score);
}

#[test]
fn test_frecency_ordering() {
    let (_temp_dir, cache_path) = temp_dir();

    note_path(&cache_path, "/tmp", 10, false);
    sleep(500);
    note_path(&cache_path, "/usr", 1, false);
    sleep(500);
    note_path(&cache_path, "/etc", 10, false);

    let stdout = list_paths(&cache_path, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "/usr");
    assert_eq!(lines[1], "/tmp");
    assert_eq!(lines[2], "/etc");
}

#[test]
fn test_frecency_ordering_with_scores() {
    let (_temp_dir, cache_path) = temp_dir();

    note_path(&cache_path, "/tmp", 10, false);
    sleep(500);
    note_path(&cache_path, "/usr", 1, false);
    sleep(500);
    note_path(&cache_path, "/etc", 10, false);

    let stdout = list_paths(&cache_path, &["--include-frecency-score"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("/usr"));
    assert!(lines[1].starts_with("/tmp"));
    assert!(lines[2].starts_with("/etc"));
}

#[test]
fn test_note_nonexistent_path() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_path = "/this/path/definitely/does/not/exist";

    let output = memy_cmd(&cache_path, &["--note", test_path])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(),);
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected_error = format!("Path {} does not exist.", test_path);
    assert!(stderr.contains(&expected_error),);
}

#[test]
fn test_help_flag() {
    let (_temp_dir, cache_path) = temp_dir();

    let output = memy_cmd(&cache_path, &["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_note_symlink_resolves_to_target() {
    let (_temp_dir, cache_path) = temp_dir();

    let dummy_file_path = cache_path.join("dummy_file_A");
    fs::write(&dummy_file_path, "dummy content").expect("failed to create dummy file");

    let symlink_path = cache_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&cache_path, symlink_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], dummy_file_path.to_str().unwrap());
}

#[test]
fn test_note_symlink_with_no_normalize_option() {
    let (_temp_dir, cache_path) = temp_dir();

    let dummy_file_path = cache_path.join("dummy_file_A");
    fs::write(&dummy_file_path, "dummy content").expect("failed to create dummy file");

    let symlink_path = cache_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&cache_path, symlink_path.to_str().unwrap(), 1, true);

    let stdout = list_paths(&cache_path, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], symlink_path.to_str().unwrap());
}

#[test]
fn test_note_deleted_file_not_in_list() {
    let (_temp_dir, cache_path) = temp_dir();

    let temp_file_path = cache_path.join("test_file");
    fs::write(&temp_file_path, "test content").expect("failed to create test file");

    note_path(&cache_path, temp_file_path.to_str().unwrap(), 1, false);

    let stdout_before = list_paths(&cache_path, &[]);
    let lines_before: Vec<&str> = stdout_before.lines().collect();
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], temp_file_path.to_str().unwrap());

    fs::remove_file(&temp_file_path).expect("failed to delete test file");

    let stdout_after = list_paths(&cache_path, &[]);
    let lines_after: Vec<&str> = stdout_after.lines().collect();
    assert_eq!(lines_after.len(), 0);
}

#[test]
fn test_files_only_flag() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_file_path = cache_path.join("test_file");
    fs::write(&test_file_path, "test content").expect("failed to create test file");

    let test_dir_path = cache_path.join("test_dir");
    fs::create_dir(&test_dir_path).expect("failed to create test directory");

    note_path(&cache_path, test_file_path.to_str().unwrap(), 1, false);
    note_path(&cache_path, test_dir_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, &["--files-only"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_directories_only_flag() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_file_path = cache_path.join("test_file");
    fs::write(&test_file_path, "test content").expect("failed to create test file");

    let test_dir_path = cache_path.join("test_dir");
    fs::create_dir(&test_dir_path).expect("failed to create test directory");

    note_path(&cache_path, test_file_path.to_str().unwrap(), 1, false);
    note_path(&cache_path, test_dir_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, &["--directories-only"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_dir_path.to_str().unwrap());
}
