#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::env;

#[test]
fn test_output_filter_basic_with_head() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");
    let file_c = create_test_file(&ctx.working_path, "file_c.txt", "content c");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--output-filter",
    ];
    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &args,
        vec![("MEMY_OUTPUT_FILTER", "head -1")],
    );

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_with_command_flag() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let args = vec![
        "list",
        "--output-filter",
        "--output-filter-command",
        "head -1",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_tilde_expansion_in_home_directory() {
    let ctx = TestContext::new();

    let home = env::var("HOME").expect("HOME env var should be set");

    note_path(&ctx.db_path, None, "~", 1, &[], &[]);

    let args = vec![
        "list",
        "--output-filter",
        "--output-filter-command",
        "head -1",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert!(
        !result.starts_with('~'),
        "Tilde should be expanded in output, got: {result}"
    );
    assert!(
        result.starts_with(&home),
        "Output should start with home directory: {home}, got: {result}"
    );
}

#[test]
fn test_output_filter_with_invalid_command() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");

    note_path(&ctx.db_path, None, file_a.to_str().unwrap(), 1, &[], &[]);

    let args = vec![
        "list",
        "--output-filter",
        "--output-filter-command",
        "this_command_does_not_exist_12345",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);

    assert!(!output.status.success(), "Command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Output filter command failed via shell"),
        "Error message should mention shell command failure, got: {stderr}"
    );
}

#[test]
fn test_output_filter_with_config_option() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");

    create_config_file(
        &ctx.config_path,
        r#"
import_on_first_use = false
memy_output_filter = "head -1"
"#,
    );

    note_paths_with_delay(&ctx.db_path, Some(&ctx.config_path), &[&file_a, &file_b]);

    let output = memy_cmd_test_defaults(
        &ctx.db_path,
        Some(&ctx.config_path),
        &["list", "--output-filter"],
    );

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_priority_command_flag_over_env() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--output-filter",
        "--output-filter-command",
        "head -1",
    ];
    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &args,
        vec![("MEMY_OUTPUT_FILTER", "tail -1")],
    );

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should use head, not tail, so first file
    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_priority_env_over_config() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");

    create_config_file(
        &ctx.config_path,
        r#"
import_on_first_use = false
memy_output_filter = "tail -1"
"#,
    );

    note_paths_with_delay(&ctx.db_path, Some(&ctx.config_path), &[&file_a, &file_b]);

    // MEMY_OUTPUT_FILTER env var should take priority over config
    let args = vec!["list", "--output-filter"];
    let output = memy_cmd(
        Some(&ctx.db_path),
        Some(&ctx.config_path),
        &args,
        vec![("MEMY_OUTPUT_FILTER", "head -1")],
    );

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    // Should use head, not tail, so first file
    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_with_format_json_fails() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    note_path(&ctx.db_path, None, file_a.to_str().unwrap(), 1, &[], &[]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--output-filter",
        "--format",
        "json",
        "--output-filter-command",
        "head -1",
    ];
    let output = memy_cmd(Some(&ctx.db_path), None, &args, vec![]);

    assert!(
        !output.status.success(),
        "Command should fail when using --output-filter with --format json"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--output-filter (or --select) can only be used with --format plain"),
        "Error message should mention format constraint, got: {stderr}"
    );
}

#[test]
fn test_output_filter_supports_shell_pipes_and_quotes() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content b");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--output-filter",
        "--output-filter-command",
        "cat | grep -v 'file_b'",
    ];
    let output = memy_cmd(Some(&ctx.db_path), None, &args, vec![]);

    assert!(output.status.success(), "Command should succeed");

    let result = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = result.lines().collect();

    assert_eq!(lines.len(), 1, "Should only return one line");
    assert_eq!(lines[0], file_a.to_str().unwrap());
}

#[test]
fn test_output_filter_with_malformed_command() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content a");
    note_path(&ctx.db_path, None, file_a.to_str().unwrap(), 1, &[], &[]);

    let args = vec![
        "list",
        "--output-filter",
        "--output-filter-command",
        "grep 'unclosed",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);

    assert!(!output.status.success(), "Command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Output filter command failed via shell"),
        "Error message should mention shell command failure, got: {stderr}"
    );
}
