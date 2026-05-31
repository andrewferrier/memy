#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::fs;

#[test]
fn test_file_replaced_by_directory_appears_as_directory() {
    let ctx = TestContext::new();

    let path = ctx.working_path.join("type_change_path");
    fs::write(&path, "test content").expect("failed to create test file");
    note_path(&ctx.db_path, None, path.to_str().unwrap(), 1, &[], &[]);

    fs::remove_file(&path).expect("failed to delete test file");
    fs::create_dir(&path).expect("failed to create replacement directory");

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(&lines, &[path.to_str().unwrap()]);

    let lines_dirs = list_paths(&ctx.db_path, None, &[], &["--directories-only"]);
    assert_lines_eq(&lines_dirs, &[path.to_str().unwrap()]);

    let lines_files = list_paths(&ctx.db_path, None, &[], &["--files-only"]);
    assert_lines_eq(&lines_files, &[]);
}

#[test]
fn test_directory_replaced_by_file_appears_as_file() {
    let ctx = TestContext::new();

    let path = ctx.working_path.join("type_change_path");
    fs::create_dir(&path).expect("failed to create test directory");
    note_path(&ctx.db_path, None, path.to_str().unwrap(), 1, &[], &[]);

    fs::remove_dir(&path).expect("failed to delete test directory");
    fs::write(&path, "test content").expect("failed to create replacement file");

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(&lines, &[path.to_str().unwrap()]);

    let lines_files = list_paths(&ctx.db_path, None, &[], &["--files-only"]);
    assert_lines_eq(&lines_files, &[path.to_str().unwrap()]);

    let lines_dirs = list_paths(&ctx.db_path, None, &[], &["--directories-only"]);
    assert_lines_eq(&lines_dirs, &[]);
}
