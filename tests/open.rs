#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_open_directory_fails() {
    let ctx = TestContext::new();

    let output = memy_cmd(
        Some(&ctx.db_path),
        Some(&ctx.config_path),
        &["open", ctx.data_path.to_str().unwrap()],
        vec![],
    );

    assert!(
        !output.status.success(),
        "memy open on a directory should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("directory"),
        "expected 'directory' in error: {stderr}"
    );
}

#[test]
fn test_open_nonexistent_fails() {
    let ctx = TestContext::new();

    let output = memy_cmd(
        Some(&ctx.db_path),
        Some(&ctx.config_path),
        &["open", "/nonexistent/path/that/does/not/exist.txt"],
        vec![],
    );

    assert!(
        !output.status.success(),
        "memy open on a nonexistent path should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist"),
        "expected 'does not exist' in error: {stderr}"
    );
}

#[test]
fn test_open_no_args_fails() {
    let ctx = TestContext::new();

    let output = memy_cmd(
        Some(&ctx.db_path),
        Some(&ctx.config_path),
        &["open"],
        vec![],
    );

    assert!(
        !output.status.success(),
        "memy open with no args should fail"
    );
}
