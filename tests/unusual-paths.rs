#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_file_with_space_in_filename() {
    let ctx = TestContext::new();

    let filename = "file with space.txt";
    let file_path = create_test_file(&ctx.working_path, filename, "test content");

    note_path(&ctx.db_path, None, file_path.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(&lines, &[file_path.to_str().unwrap()]);
}

#[test]
fn test_file_with_emoji_in_filename() {
    let ctx = TestContext::new();

    let filename = "file_😀.txt";
    let file_path = create_test_file(&ctx.working_path, filename, "test content");

    note_path(&ctx.db_path, None, file_path.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(&lines, &[file_path.to_str().unwrap()]);
}
