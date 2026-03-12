#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_note_nonexistent_file_ignore_silently() {
    let ctx = TestContext::new();
    let config_contents = "missing_files_warn_on_note = false\n";
    create_config_file(&ctx.config_path, config_contents);

    let test_path = "/tmp/this_file_should_not_exist_ignore_silently";
    let output = note_path(&ctx.db_path, Some(&ctx.config_path), test_path, 1, &[], &[]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());
    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(lines.is_empty());
}

#[test]
fn test_note_nonexistent_file_ignore_with_warning() {
    let ctx = TestContext::new();
    let config_contents = "missing_files_warn_on_note = true\n";
    create_config_file(&ctx.config_path, config_contents);

    let test_path = "/tmp/this_file_should_not_exist_ignore_with_warning";
    let output = note_path(&ctx.db_path, Some(&ctx.config_path), test_path, 1, &[], &[]);

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(lines.is_empty());
}
