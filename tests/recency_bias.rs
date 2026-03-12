#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use core::time::Duration;
use std::thread::sleep;

#[test]
fn test_recency_bias_0() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "dir_a");
    let dir_b = create_test_directory(&ctx.working_path, "dir_b");

    note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        dir_a.to_str().unwrap(),
        2,
        &[],
        &[],
    );
    sleep(Duration::from_secs(1));
    note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        dir_b.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let config_contents = "recency_bias=0\n";
    create_config_file(&ctx.config_path, config_contents);

    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);

    assert!(
        lines.iter().position(|line| line.contains("dir_b"))
            < lines.iter().position(|line| line.contains("dir_a"))
    );
}

#[test]
fn test_recency_bias_1() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "dir_a");
    let dir_b = create_test_directory(&ctx.working_path, "dir_b");

    note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        dir_a.to_str().unwrap(),
        2,
        &[],
        &[],
    );
    sleep(Duration::from_secs(1));
    note_path(
        &ctx.db_path,
        Some(&ctx.config_path),
        dir_b.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let config_contents = "recency_bias=1\n";
    create_config_file(&ctx.config_path, config_contents);

    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_path_before(&lines, "dir_a", "dir_b");
}

#[test]
fn test_recency_bias_below_0() {
    let ctx = TestContext::new();

    let config_contents = "recency_bias=-1\n";
    create_config_file(&ctx.config_path, config_contents);

    let output = memy_cmd(None, Some(&ctx.config_path), &["list"], vec![]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be between 0 and 1"),);
}

#[test]
fn test_recency_bias_above_1() {
    let ctx = TestContext::new();

    let config_contents = "recency_bias=1.5\n";
    create_config_file(&ctx.config_path, config_contents);

    let output = memy_cmd(None, Some(&ctx.config_path), &["list"], vec![]);
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("must be between 0 and 1"),);
}
