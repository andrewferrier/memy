use assert_cmd::Command;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let path = temp_dir
        .path()
        .canonicalize()
        .expect("failed to canonicalize temp dir path");
    (temp_dir, path)
}

fn create_config_file(config_path: &std::path::Path, contents: &str) {
    let config_toml_path = config_path.join("memy.toml");
    fs::write(&config_toml_path, contents).expect("failed to write config");
}

fn create_test_file(dir: &std::path::Path, filename: &str, contents: &str) -> std::path::PathBuf {
    let file_path = dir.join(filename);
    fs::write(&file_path, contents).expect("failed to create test file");
    file_path
}

fn memy_cmd(
    cache_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    args: &[&str],
) -> Command {
    let mut cmd = Command::cargo_bin("memy").unwrap();
    cmd.env("MEMY_CACHE_DIR", cache_path);

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

fn sleep(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn note_path(
    cache_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    path: &str,
    count: usize,
    no_normalize_symlinks: bool,
) {
    for _ in 0..count {
        let mut args = vec!["--note"];
        args.push(path);

        if no_normalize_symlinks {
            args.push("--no-normalize-symlinks");
        }

        memy_cmd(cache_path, config_path, &args).assert().success();
        sleep(100);
    }
}

fn list_paths(
    cache_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    flags: &[&str],
) -> String {
    let mut args = vec!["--list"];
    args.extend(flags);
    let output = memy_cmd(cache_path, config_path, &args)
        .output()
        .expect("failed to run memy");
    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_note_and_list_paths() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = working_path.join("dir_a");
    let dir_b = working_path.join("dir_b");
    fs::create_dir(&dir_a).expect("failed to create dir_a");
    fs::create_dir(&dir_b).expect("failed to create dir_b");

    note_path(&cache_path, None, dir_a.to_str().unwrap(), 1, false);
    sleep(1000);
    note_path(&cache_path, None, dir_b.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, None, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], dir_a.to_str().unwrap());
    assert_eq!(lines[1], dir_b.to_str().unwrap());
}

#[test]
fn test_note_and_list_paths_with_scores() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = working_path.join("dir_a");
    let dir_b = working_path.join("dir_b");
    fs::create_dir(&dir_a).expect("failed to create dir_a");
    fs::create_dir(&dir_b).expect("failed to create dir_b");

    note_path(&cache_path, None, dir_a.to_str().unwrap(), 1, false);
    sleep(1000);
    note_path(&cache_path, None, dir_b.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, None, &["--include-frecency-score"]);
    let lines: Vec<&str> = stdout.lines().collect();

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
    let (_temp_dir, cache_path) = temp_dir();

    let test_file_name = "rel_test_file";
    let test_file_path = create_test_file(&cache_path, test_file_name, "test content");

    let orig_dir = std::env::current_dir().expect("failed to get current dir");
    std::env::set_current_dir(&cache_path).expect("failed to change dir");

    note_path(&cache_path, None, test_file_name, 1, false);

    std::env::set_current_dir(orig_dir).expect("failed to restore dir");

    let stdout = list_paths(&cache_path, None, &[]);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_frecency_ordering() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = working_path.join("dir_a");
    let dir_b = working_path.join("dir_b");
    let dir_c = working_path.join("dir_c");
    fs::create_dir(&dir_a).expect("failed to create dir_a");
    fs::create_dir(&dir_b).expect("failed to create dir_b");
    fs::create_dir(&dir_c).expect("failed to create dir_c");

    note_path(&cache_path, None, dir_a.to_str().unwrap(), 10, false);
    sleep(500);
    note_path(&cache_path, None, dir_b.to_str().unwrap(), 1, false);
    sleep(500);
    note_path(&cache_path, None, dir_c.to_str().unwrap(), 10, false);

    let stdout = list_paths(&cache_path, None, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], dir_b.to_str().unwrap());
    assert_eq!(lines[1], dir_a.to_str().unwrap());
    assert_eq!(lines[2], dir_c.to_str().unwrap());
}

#[test]
fn test_frecency_ordering_with_scores() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let dir_a = working_path.join("dir_a");
    let dir_b = working_path.join("dir_b");
    let dir_c = working_path.join("dir_c");
    fs::create_dir(&dir_a).expect("failed to create dir_a");
    fs::create_dir(&dir_b).expect("failed to create dir_b");
    fs::create_dir(&dir_c).expect("failed to create dir_c");

    note_path(&cache_path, None, dir_a.to_str().unwrap(), 10, false);
    sleep(500);
    note_path(&cache_path, None, dir_b.to_str().unwrap(), 1, false);
    sleep(500);
    note_path(&cache_path, None, dir_c.to_str().unwrap(), 10, false);

    let stdout = list_paths(&cache_path, None, &["--include-frecency-score"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with(dir_b.to_str().unwrap()));
    assert!(lines[1].starts_with(dir_a.to_str().unwrap()));
    assert!(lines[2].starts_with(dir_c.to_str().unwrap()));
}

#[test]
fn test_note_nonexistent_path() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_path = "/this/path/definitely/does/not/exist";

    let output = memy_cmd(&cache_path, None, &["--note", test_path])
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
    let (_temp_dir, cache_path) = temp_dir();

    let output = memy_cmd(&cache_path, None, &["--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_note_symlink_resolves_to_target() {
    let (_temp_dir, cache_path) = temp_dir();

    let dummy_file_path = create_test_file(&cache_path, "dummy_file_A", "dummy content");

    let symlink_path = cache_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&cache_path, None, symlink_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, None, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], dummy_file_path.to_str().unwrap());
}

#[test]
fn test_note_symlink_with_no_normalize_option() {
    let (_temp_dir, cache_path) = temp_dir();

    let dummy_file_path = create_test_file(&cache_path, "dummy_file_A", "dummy content");

    let symlink_path = cache_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(&cache_path, None, symlink_path.to_str().unwrap(), 1, true);

    let stdout = list_paths(&cache_path, None, &[]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], symlink_path.to_str().unwrap());
}

#[test]
fn test_note_deleted_file_not_in_list() {
    let (_temp_dir, cache_path) = temp_dir();

    let temp_file_path = create_test_file(&cache_path, "test_file", "test content");

    note_path(
        &cache_path,
        None,
        temp_file_path.to_str().unwrap(),
        1,
        false,
    );

    let stdout_before = list_paths(&cache_path, None, &[]);
    let lines_before: Vec<&str> = stdout_before.lines().collect();
    assert_eq!(lines_before.len(), 1);
    assert_eq!(lines_before[0], temp_file_path.to_str().unwrap());

    fs::remove_file(&temp_file_path).expect("failed to delete test file");

    let stdout_after = list_paths(&cache_path, None, &[]);
    assert_eq!(stdout_after.lines().count(), 0);
}

#[test]
fn test_files_only_flag() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_file_path = create_test_file(&cache_path, "test_file", "test content");

    let test_dir_path = cache_path.join("test_dir");
    fs::create_dir(&test_dir_path).expect("failed to create test directory");

    note_path(
        &cache_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        false,
    );
    note_path(&cache_path, None, test_dir_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, None, &["--files-only"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_file_path.to_str().unwrap());
}

#[test]
fn test_directories_only_flag() {
    let (_temp_dir, cache_path) = temp_dir();

    let test_file_path = create_test_file(&cache_path, "test_file", "test content");

    let test_dir_path = cache_path.join("test_dir");
    fs::create_dir(&test_dir_path).expect("failed to create test directory");

    note_path(
        &cache_path,
        None,
        test_file_path.to_str().unwrap(),
        1,
        false,
    );
    note_path(&cache_path, None, test_dir_path.to_str().unwrap(), 1, false);

    let stdout = list_paths(&cache_path, None, &["--directories-only"]);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], test_dir_path.to_str().unwrap());
}

#[test]
fn test_denylist_excludes_file() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let deny_file = create_test_file(&working_path, "denyme.txt", "deny me");

    let deny_pattern = deny_file.to_str().unwrap();
    let config_contents = format!("denylist_silent = [\"{deny_pattern}\"]\n");
    create_config_file(&config_path, &config_contents);

    note_path(
        &cache_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    );

    let stdout = list_paths(&cache_path, Some(&config_path), &[]);
    assert_eq!(stdout.lines().count(), 0);
}

#[test]
fn test_denylist_excludes_file_with_subdir_glob() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let subdir = working_path.join("subdir");
    fs::create_dir(&subdir).expect("failed to create subdir");
    let deny_file = create_test_file(&subdir, "denyme.txt", "deny me");

    let deny_pattern = format!("{}/*", subdir.to_str().unwrap());
    let config_contents = format!("denylist_silent = [\"{deny_pattern}\"]\n");
    create_config_file(&config_path, &config_contents);

    note_path(
        &cache_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    );

    let stdout = list_paths(&cache_path, Some(&config_path), &[]);
    assert_eq!(stdout.lines().count(), 0);
}

#[test]
fn test_denylist_excludes_file_with_double_star_glob() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let nested_dir = working_path.join("a/b/c");
    fs::create_dir_all(&nested_dir).expect("failed to create nested dirs");
    let deny_file = create_test_file(&nested_dir, "deny.txt", "deny me");

    let config_contents = "denylist_silent = [\"**/deny.txt\"]\n";
    create_config_file(&config_path, config_contents);

    note_path(
        &cache_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    );

    let stdout = list_paths(&cache_path, Some(&config_path), &[]);
    assert_eq!(stdout.lines().count(), 0);
}

#[test]
fn test_denylist_excludes_multiple_patterns() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let file1 = create_test_file(&working_path, "deny1.txt", "deny1");
    let file2 = create_test_file(&working_path, "deny2.txt", "deny2");

    let config_contents = format!(
        "denylist_silent = [\"{}\", \"{}\"]\n",
        file1.to_str().unwrap(),
        file2.to_str().unwrap()
    );
    create_config_file(&config_path, &config_contents);

    note_path(
        &cache_path,
        Some(&config_path),
        file1.to_str().unwrap(),
        1,
        false,
    );
    note_path(
        &cache_path,
        Some(&config_path),
        file2.to_str().unwrap(),
        1,
        false,
    );

    let stdout = list_paths(&cache_path, Some(&config_path), &[]);
    assert_eq!(stdout.lines().count(), 0);
}

#[test]
fn test_denylist_pattern_not_rooted_does_not_match_absolute() {
    let (_temp_dir, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let file1 = create_test_file(&working_path, "notrooted.txt", "should not be denied");

    let config_contents = "denylist_silent = [\"notrooted.txt\"]\n";
    create_config_file(&config_path, config_contents);

    note_path(
        &cache_path,
        Some(&config_path),
        file1.to_str().unwrap(),
        1,
        false,
    );

    let stdout = list_paths(&cache_path, Some(&config_path), &[]);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], file1.to_str().unwrap());
}
