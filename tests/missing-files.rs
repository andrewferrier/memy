#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_note_nonexistent_file_ignore_silently() {
    let ctx = TestContext::new();
    create_config_file(&ctx.config_path, "missing_files_warn_on_note = false\n");

    let output = note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        "/tmp/this_file_should_not_exist_ignore_silently",
        1,
        &[],
        &[],
    );

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(lines.is_empty());
}

#[test]
fn test_note_nonexistent_file_ignore_with_warning() {
    let ctx = TestContext::new();
    create_config_file(&ctx.config_path, "missing_files_warn_on_note = true\n");

    let output = note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        "/tmp/this_file_should_not_exist_ignore_with_warning",
        1,
        &[],
        &[],
    );

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(lines.is_empty());
}
