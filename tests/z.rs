#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::env::home_dir;
use std::fs;

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
    assert!(
        result.starts_with('/'),
        "Result should be absolute, got: {result}"
    );
}

#[test]
fn test_z_single_nonexistent_path_triggers_db_search() {
    let ctx = TestContext::new();

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

#[test]
fn test_zi_no_filter_available_fails() {
    let ctx = TestContext::new();

    let dir_bar = create_test_directory(&ctx.working_path, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    // Unset MEMY_OUTPUT_FILTER and prevent auto-detection of fzf/sk/fzy by
    // clearing PATH so no external binaries can be found.
    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "bar",
        ],
        vec![("MEMY_OUTPUT_FILTER", ""), ("PATH", "")],
    );

    assert!(
        !output.status.success(),
        "zi should fail when no output filter is available"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No output filter command found"),
        "Should report missing filter, got: {stderr}"
    );
}

#[test]
fn test_z_denylist_excludes_dir() {
    let ctx = TestContext::new();

    let denied_dir = create_test_directory(&ctx.working_path, "secret");
    note_path(
        &ctx.db_path,
        None,
        denied_dir.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let deny_pattern = denied_dir.to_str().unwrap();
    let config_contents = format!("denylist = [\"{deny_pattern}\"]\n");
    create_config_file(&ctx.config_path, &config_contents);

    let output = z_command(&ctx.db_path, Some(&ctx.config_path), &["secret"]);

    assert!(
        !output.status.success(),
        "z should not return a denied directory"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found' for denied dir, got: {stderr}"
    );
}

#[test]
fn test_z_missing_dir_not_yet_expired() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);

    let output = z_command(&ctx.db_path, None, &["mydir"]);
    assert!(output.status.success(), "Should find dir before deletion");

    fs::remove_dir(&dir).expect("failed to remove test dir");

    let output2 = z_command(&ctx.db_path, None, &["mydir"]);
    assert!(
        !output2.status.success(),
        "Should not return a missing directory"
    );

    fs::create_dir(&dir).expect("failed to recreate test dir");

    let output3 = z_command(&ctx.db_path, None, &["mydir"]);
    assert!(
        output3.status.success(),
        "Should find dir again after recreation when entry was not expired"
    );
    let result = String::from_utf8_lossy(&output3.stdout).trim().to_owned();
    assert_eq!(result, dir.to_str().unwrap());
}

#[test]
fn test_z_missing_dir_expired_deleted_from_db() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);

    age_all_paths(&ctx.db_path, 45);

    fs::remove_dir(&dir).expect("failed to remove test dir");

    let output = z_command(&ctx.db_path, None, &["mydir"]);
    assert!(
        !output.status.success(),
        "Should not return a missing/expired directory"
    );

    fs::create_dir(&dir).expect("failed to recreate test dir");

    let output2 = z_command(&ctx.db_path, None, &["mydir"]);
    assert!(
        !output2.status.success(),
        "Should not find dir after recreation when expired entry was deleted from DB"
    );
    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}

#[test]
fn test_zi_no_keywords_shows_all_dirs() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "alpha");
    let dir_b = create_test_directory(&ctx.working_path, "beta");
    let dir_c = create_test_directory(&ctx.working_path, "gamma");
    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_c.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["--config", "import_on_first_use=false", "z", "-i"],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(
        output.status.success(),
        "zi with no keywords should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "All 3 dirs should be shown, got: {lines:?}");
}

/// zi with a keyword shows ALL dirs where the keyword appears in the last path
/// component — multiple matching dirs all reach fzf, not just the top one.
#[test]
fn test_zi_keyword_shows_all_dirs_with_keyword_in_last_component() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "projects_a");
    let dir_b = create_test_directory(&ctx.working_path, "projects_b");
    let dir_other = create_test_directory(&ctx.working_path, "other");
    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_other.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "projects",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(
        output.status.success(),
        "zi 'projects' should succeed with multiple matches"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Both 'projects_a' and 'projects_b' should appear in fzf, got: {lines:?}"
    );
    assert!(
        stdout.contains(dir_a.to_str().unwrap()),
        "dir_a should be present"
    );
    assert!(
        stdout.contains(dir_b.to_str().unwrap()),
        "dir_b should be present"
    );
    assert!(
        !stdout.contains(dir_other.to_str().unwrap()),
        "dir_other should be absent"
    );
}

/// zi applies the same strict last-component rule as `z`: dirs where the keyword
/// appears only in a non-final component are excluded (matching zoxide's behaviour).
#[test]
fn test_zi_keyword_does_not_show_dirs_with_keyword_only_in_non_last_component() {
    let ctx = TestContext::new();

    // "personal" is only in the middle component, not the last
    let personal = create_test_directory(&ctx.working_path, "personal");
    let notes = create_test_directory(&personal, "notes");
    note_path(&ctx.db_path, None, notes.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "personal",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(
        !output.status.success(),
        "zi should not match 'personal' when it only appears in a non-last path component"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}
#[test]
fn test_zi_keyword_no_match_fails() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    note_path(&ctx.db_path, None, dir_foo.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "zzz_no_such_keyword_xyz",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(
        !output.status.success(),
        "zi should fail when keyword matches nothing"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}

#[test]
fn test_zi_keyword_excludes_non_matching_dirs() {
    let ctx = TestContext::new();

    let dir_match = create_test_directory(&ctx.working_path, "notes");
    let dir_nomatch = create_test_directory(&ctx.working_path, "work");
    note_path(&ctx.db_path, None, dir_match.to_str().unwrap(), 1, &[], &[]);
    note_path(
        &ctx.db_path,
        None,
        dir_nomatch.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "notes",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(output.status.success(), "zi 'notes' should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Only matching dir should appear, got: {lines:?}"
    );
    assert!(
        stdout.contains(dir_match.to_str().unwrap()),
        "dir_match should be present"
    );
    assert!(
        !stdout.contains(dir_nomatch.to_str().unwrap()),
        "dir_nomatch should be absent"
    );
}

#[test]
fn test_zi_only_returns_dirs_not_files() {
    let ctx = TestContext::new();

    let file = create_test_file(&ctx.working_path, "notes.txt", "content");
    note_path(&ctx.db_path, None, file.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "notes",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(!output.status.success(), "zi should not return files");
}

#[test]
fn test_zi_multiple_keywords_in_order() {
    let ctx = TestContext::new();

    let foo = create_test_directory(&ctx.working_path, "foo");
    let foo_bar = create_test_directory(&foo, "bar");
    let other = create_test_directory(&ctx.working_path, "other");
    note_path(&ctx.db_path, None, foo_bar.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, other.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "foo",
            "bar",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(output.status.success(), "zi foo bar should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "Only foo/bar should match, got: {lines:?}");
    assert!(stdout.contains(foo_bar.to_str().unwrap()));
}

#[test]
fn test_zi_multiple_keywords_reversed_no_match() {
    let ctx = TestContext::new();

    let foo = create_test_directory(&ctx.working_path, "foo");
    let foo_bar = create_test_directory(&foo, "bar");
    note_path(&ctx.db_path, None, foo_bar.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "z",
            "-i",
            "--",
            "bar",
            "foo",
        ],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(
        !output.status.success(),
        "zi with reversed keywords should not match"
    );
}

#[test]
fn test_zi_empty_db_fails() {
    let ctx = TestContext::new();

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["--config", "import_on_first_use=false", "z", "-i"],
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    );

    assert!(!output.status.success(), "zi on empty DB should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no match found"),
        "Should report 'no match found', got: {stderr}"
    );
}
