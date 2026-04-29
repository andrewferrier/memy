#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::env::home_dir;

fn z_command(
    db_path: &std::path::Path,
    config_path: Option<&std::path::Path>,
    z_args: &[&str],
) -> std::process::Output {
    let mut args = vec!["z"];
    args.extend(z_args);
    memy_cmd_test_defaults(db_path, config_path, &args)
}

#[test]
fn test_z_no_args_outputs_home() {
    let ctx = TestContext::new();

    let output = z_command(&ctx.db_path, None, &[]);

    assert!(
        output.status.success(),
        "memy z with no args should succeed"
    );

    let home = home_dir().expect("HOME must be set");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, home.to_string_lossy().as_ref());
}

#[test]
fn test_z_single_absolute_existing_dir() {
    let ctx = TestContext::new();

    // ctx.working_path is a real temp dir
    let dir_str = ctx.working_path.to_str().unwrap();
    let output = z_command(&ctx.db_path, None, &[dir_str]);

    assert!(
        output.status.success(),
        "Should succeed for existing absolute dir"
    );
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_str);
}

#[test]
fn test_z_single_tilde_home() {
    let ctx = TestContext::new();

    let output = z_command(&ctx.db_path, None, &["~"]);

    assert!(output.status.success(), "memy z ~ should succeed");
    let home = home_dir().expect("HOME must be set");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, home.to_string_lossy().as_ref());
}

#[test]
fn test_z_single_relative_dotdot() {
    let ctx = TestContext::new();

    let output = z_command(&ctx.db_path, None, &[".."]);

    assert!(output.status.success(), "memy z .. should succeed");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    // The result should be an absolute path (parent of cwd)
    assert!(
        result.starts_with('/'),
        "Result should be absolute, got: {result}"
    );
}

#[test]
fn test_z_single_nonexistent_path_triggers_db_search() {
    let ctx = TestContext::new();

    // A path that definitely does not exist; should fall through to DB search and fail
    let output = z_command(
        &ctx.db_path,
        None,
        &["/this/path/definitely/does/not/exist/xyz"],
    );

    assert!(
        !output.status.success(),
        "Should fail when path doesn't exist and DB is empty"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}

#[test]
fn test_z_dash_arg_exits_nonzero() {
    let ctx = TestContext::new();

    // z - requires OLDPWD which memy can't access; should error
    let output = z_command(&ctx.db_path, None, &["--", "-"]);

    // `-` is not an existing directory, so it falls through to DB search (empty DB → no match)
    // OR: if the single-arg check for "-" fires first, it errors with the special message.
    // Either way, it must exit non-zero.
    assert!(!output.status.success(), "z - should exit non-zero");
}

#[test]
fn test_z_basic_keyword_match() {
    let ctx = TestContext::new();

    let dir_bar = create_test_directory(&ctx.working_path, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar"]);

    assert!(output.status.success(), "Should match 'bar'");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_bar.to_str().unwrap());
}

#[test]
fn test_z_case_insensitive() {
    let ctx = TestContext::new();

    let dir_bar = create_test_directory(&ctx.working_path, "BAR");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar"]);

    assert!(output.status.success(), "Should match case-insensitively");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_bar.to_str().unwrap());
}

#[test]
fn test_z_multiple_keywords_in_order() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["foo", "bar"]);

    assert!(output.status.success(), "Should match foo then bar");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_bar.to_str().unwrap());
}

#[test]
fn test_z_ordered_keywords_no_reverse_match() {
    let ctx = TestContext::new();

    // /foo/bar should NOT match keywords ["bar", "foo"] (wrong order)
    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar", "foo"]);

    assert!(
        !output.status.success(),
        "Reversed keywords should not match"
    );
}

#[test]
fn test_z_last_component_rule() {
    let ctx = TestContext::new();

    // /bar/foo should NOT match keyword "bar" because the last component is "foo", not "bar"
    let dir_bar = create_test_directory(&ctx.working_path, "bar");
    let dir_foo = create_test_directory(&dir_bar, "foo");
    note_path(&ctx.db_path, None, dir_foo.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar"]);

    assert!(
        !output.status.success(),
        "Should not match when 'bar' is not in last component"
    );
}

#[test]
fn test_z_only_returns_dirs_not_files() {
    let ctx = TestContext::new();

    // Note a file matching "bar" — should NOT be returned by z
    let file_bar = create_test_file(&ctx.working_path, "bar.txt", "content");
    note_path(&ctx.db_path, None, file_bar.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar"]);

    assert!(!output.status.success(), "z should not return files");
}

#[test]
fn test_z_returns_most_frecent_of_multiple_matches() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "bar_a");
    let dir_b = create_test_directory(&ctx.working_path, "bar_b");

    // Note dir_a once, dir_b three times → dir_b is more frecent
    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 3, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["bar"]);

    assert!(output.status.success(), "Should find a match");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(
        result,
        dir_b.to_str().unwrap(),
        "Should return the more frecent dir_b"
    );
}

#[test]
fn test_z_no_match_exits_nonzero_with_message() {
    let ctx = TestContext::new();

    let output = z_command(&ctx.db_path, None, &["zzz_no_such_keyword_xyz"]);

    assert!(
        !output.status.success(),
        "Should exit non-zero when no match"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}

#[test]
fn test_z_slash_in_keyword_matches_path_component() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    // "foo/bar" as a single keyword should match /...working_path.../foo/bar
    let output = z_command(&ctx.db_path, None, &["foo/bar"]);

    assert!(
        output.status.success(),
        "foo/bar keyword should match /foo/bar"
    );
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_bar.to_str().unwrap());
}

#[test]
fn test_z_slash_in_keyword_does_not_match_deeper_path() {
    let ctx = TestContext::new();

    // "foo/bar" should NOT match /foo/bar/baz (last component "baz" doesn't contain "bar")
    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    let dir_baz = create_test_directory(&dir_bar, "baz");
    note_path(&ctx.db_path, None, dir_baz.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["foo/bar"]);

    assert!(
        !output.status.success(),
        "foo/bar keyword should not match /foo/bar/baz"
    );
}

#[test]
fn test_z_empty_db_exits_nonzero() {
    let ctx = TestContext::new();

    let output = z_command(&ctx.db_path, None, &["anything"]);

    assert!(!output.status.success(), "Empty DB should produce no match");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}
