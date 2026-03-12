#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use std::os::unix::fs::symlink;
use support::*;

#[test]
fn test_note_symlink_resolves_to_target() {
    let ctx = TestContext::new();

    let dummy_file_path = create_test_file(&ctx.working_path, "dummy_file_A", "dummy content");

    let symlink_path = ctx.working_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    note_path(
        &ctx.db_path,
        None,
        symlink_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(&lines, &[dummy_file_path.to_str().unwrap()]);
}

#[test]
fn test_note_symlink_with_no_normalize_option() {
    let ctx = TestContext::new();

    let dummy_file_path = create_test_file(&ctx.working_path, "dummy_file_A", "dummy content");
    let symlink_path = ctx.working_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    let config_contents = "normalize_symlinks_on_note = false\n";
    create_config_file(&ctx.config_path, config_contents);

    note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        symlink_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_lines_eq(&lines, &[symlink_path.to_str().unwrap()]);
}
