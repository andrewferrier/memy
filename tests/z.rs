#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_z_basic_keyword_match() {
    let ctx = TestContext::new();
    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &["z", "mydir"]);
    assert!(output.status.success(), "memy z should match keyword");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir.to_str().unwrap());
}

#[test]
fn test_z_deprecation_warning() {
    let ctx = TestContext::new();
    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &["z", "mydir"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("deprecated"),
        "memy z should print a deprecation warning, got: {stderr}"
    );
}
