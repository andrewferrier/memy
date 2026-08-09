#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_list_keyword_basic_match() {
    let ctx = TestContext::new();

    let file_foo = create_test_file(&ctx.working_path, "foobar.txt", "content");
    let file_baz = create_test_file(&ctx.working_path, "baz.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_foo, &file_baz]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--", "foobar"]);
    assert_eq!(lines.len(), 1, "Should match only foobar.txt");
    assert!(lines[0].contains("foobar.txt"));
}

#[test]
fn test_list_keyword_case_insensitive() {
    let ctx = TestContext::new();

    let file_foo = create_test_file(&ctx.working_path, "FooBar.txt", "content");
    let file_baz = create_test_file(&ctx.working_path, "baz.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_foo, &file_baz]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--", "foobar"]);
    assert_eq!(lines.len(), 1, "Should match case-insensitively");
    assert!(lines[0].to_lowercase().contains("foobar"));
}

#[test]
fn test_list_keyword_no_match() {
    let ctx = TestContext::new();

    let file = create_test_file(&ctx.working_path, "file.txt", "content");
    note_paths_with_delay(&ctx.db_path, None, &[&file]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--",
        "zzznomatch",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(
        !output.status.success(),
        "Should fail when no keyword matches"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no match found"));
}

#[test]
fn test_list_keyword_last_component_rule() {
    let ctx = TestContext::new();

    // dir/file.txt — keyword "dir" must not match because "dir" is not in the last component
    let dir = create_test_directory(&ctx.working_path, "mydir");
    let file = create_test_file(&dir, "other.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file]);

    let args_dir_keyword = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--",
        "mydir",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args_dir_keyword);
    assert!(
        !output.status.success(),
        "Keyword 'mydir' should not match when it's only in a non-last component"
    );

    let lines = list_paths(&ctx.db_path, None, &[], &["--", "other"]);
    assert_eq!(lines.len(), 1, "Keyword 'other' should match filename");
}

#[test]
fn test_list_keyword_without_double_dash() {
    let ctx = TestContext::new();

    let file_foo = create_test_file(&ctx.working_path, "foobar.txt", "content");
    let file_baz = create_test_file(&ctx.working_path, "baz.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_foo, &file_baz]);

    let lines = list_paths(&ctx.db_path, None, &[], &["foobar"]);
    assert_eq!(
        lines.len(),
        1,
        "Should match foobar.txt without -- separator"
    );
    assert!(lines[0].contains("foobar.txt"));
}

#[test]
fn test_list_multiple_keywords_without_double_dash() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let file = create_test_file(&dir_foo, "bar.txt", "content");
    let other = create_test_file(&ctx.working_path, "baz.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file, &other]);

    let lines = list_paths(&ctx.db_path, None, &[], &["foo", "bar"]);
    assert_eq!(
        lines.len(),
        1,
        "Should match foo then bar in order without -- separator"
    );
    assert!(lines[0].contains("bar.txt"));
}

#[test]
fn test_list_keyword_multiple_ordered() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let file = create_test_file(&dir_foo, "bar.txt", "content");
    let other = create_test_file(&ctx.working_path, "baz.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file, &other]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--", "foo", "bar"]);
    assert_eq!(lines.len(), 1, "Should match foo then bar in order");
    assert!(lines[0].contains("bar.txt"));
}

#[test]
fn test_list_keyword_reversed_no_match() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let file = create_test_file(&dir_foo, "bar.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file]);

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "bar",
        "foo",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(
        !output.status.success(),
        "Reversed keywords should not match"
    );
}

#[test]
fn test_list_keyword_with_files_only_flag() {
    let ctx = TestContext::new();

    let file = create_test_file(&ctx.working_path, "report.txt", "content");
    let dir = create_test_directory(&ctx.working_path, "report_dir");

    note_paths_with_delay(&ctx.db_path, None, &[&file, &dir]);

    let lines = list_paths(&ctx.db_path, None, &[], &["-f", "--", "report"]);
    assert_eq!(lines.len(), 1, "Should return only the file with -f");
    assert!(lines[0].contains("report.txt"));
}

#[test]
fn test_list_keyword_with_directories_only_flag() {
    let ctx = TestContext::new();

    let file = create_test_file(&ctx.working_path, "report.txt", "content");
    let dir = create_test_directory(&ctx.working_path, "report_dir");

    note_paths_with_delay(&ctx.db_path, None, &[&file, &dir]);

    let lines = list_paths(&ctx.db_path, None, &[], &["-d", "--", "report"]);
    assert_eq!(lines.len(), 1, "Should return only the directory with -d");
    assert!(lines[0].contains("report_dir"));
}

#[test]
fn test_list_zoxide_compatible_no_keywords_prints_home() {
    let ctx = TestContext::new();

    let home = std::env::home_dir().expect("HOME must be set");
    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--zoxide-compatible",
        "-d",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(output.status.success(), "Should succeed");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, home.to_string_lossy().as_ref());
}

#[test]
fn test_list_zoxide_compatible_existing_dir_passthrough() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "mydir");
    let dir_str = dir.to_str().unwrap();

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--zoxide-compatible",
        "-d",
        "--",
        dir_str,
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(output.status.success(), "Should succeed for existing dir");
    let result = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    assert_eq!(result, dir_str);
}

#[test]
fn test_list_zoxide_compatible_tilde_home() {
    let ctx = TestContext::new();

    let home = std::env::home_dir().expect("HOME must be set");
    let lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["--zoxide-compatible", "-d", "--", "~"],
    );
    assert_lines_eq(&lines, &[home.to_string_lossy().as_ref()]);
}

#[test]
fn test_list_zoxide_compatible_dotdot() {
    let ctx = TestContext::new();

    let lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["--zoxide-compatible", "-d", "--", ".."],
    );
    assert_eq!(lines.len(), 1, "memy list .. should return one path");
    assert!(
        lines[0].starts_with('/'),
        "Result should be absolute, got: {}",
        lines[0]
    );
}

#[test]
fn test_list_zoxide_compatible_nonexistent_triggers_search() {
    let ctx = TestContext::new();

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--zoxide-compatible",
        "-d",
        "--",
        "/this/path/does/not/exist",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
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
fn test_list_zoxide_compatible_dash_exits_nonzero() {
    let ctx = TestContext::new();

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--zoxide-compatible",
        "-d",
        "--",
        "-",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(
        !output.status.success(),
        "list --zoxide-compatible -d -- - should fail"
    );
}

#[test]
fn test_list_keyword_slash_matches_component() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    note_path(&ctx.db_path, None, dir_bar.to_str().unwrap(), 1, &[], &[]);

    let lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["-d", "--limit-results", "1", "--", "foo/bar"],
    );
    assert_lines_eq(&lines, &[dir_bar.to_str().unwrap()]);
}

#[test]
fn test_list_keyword_slash_no_deeper_match() {
    let ctx = TestContext::new();

    let dir_foo = create_test_directory(&ctx.working_path, "foo");
    let dir_bar = create_test_directory(&dir_foo, "bar");
    let nested_dir = create_test_directory(&dir_bar, "baz");
    note_path(
        &ctx.db_path,
        None,
        nested_dir.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "-d",
        "--limit-results",
        "1",
        "--",
        "foo/bar",
    ];
    let output = memy_cmd_test_defaults(&ctx.db_path, None, &args);
    assert!(
        !output.status.success(),
        "foo/bar keyword should not match /foo/bar/baz"
    );
}

#[test]
fn test_list_returns_most_frecent_of_multiple_matches() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "bar_a");
    let dir_b = create_test_directory(&ctx.working_path, "bar_b");

    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 3, &[], &[]);

    let lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["-d", "--limit-results", "1", "--", "bar"],
    );
    assert_lines_eq(&lines, &[dir_b.to_str().unwrap()]);
}

#[test]
fn test_list_missing_dir_not_yet_expired() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);

    let initial_lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["-d", "--limit-results", "1", "--", "mydir"],
    );
    assert_lines_eq(&initial_lines, &[dir.to_str().unwrap()]);

    std::fs::remove_dir(&dir).expect("failed to remove test dir");

    let output = memy_cmd_test_defaults(
        &ctx.db_path,
        None,
        &["list", "-d", "--limit-results", "1", "--", "mydir"],
    );
    assert!(
        !output.status.success(),
        "Should not return a missing directory"
    );

    std::fs::create_dir(&dir).expect("failed to recreate test dir");

    let restored_lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["-d", "--limit-results", "1", "--", "mydir"],
    );
    assert_lines_eq(&restored_lines, &[dir.to_str().unwrap()]);
}

#[test]
fn test_list_missing_dir_expired_deleted_from_db() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "mydir");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);

    age_all_paths(&ctx.db_path, 45);

    std::fs::remove_dir(&dir).expect("failed to remove test dir");

    let output = memy_cmd_test_defaults(
        &ctx.db_path,
        None,
        &["list", "-d", "--limit-results", "1", "--", "mydir"],
    );
    assert!(
        !output.status.success(),
        "Should not return a missing/expired directory"
    );

    std::fs::create_dir(&dir).expect("failed to recreate test dir");

    let output2 = memy_cmd_test_defaults(
        &ctx.db_path,
        None,
        &["list", "-d", "--limit-results", "1", "--", "mydir"],
    );
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
fn test_list_output_filter_dirs_no_keywords_shows_all() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "alpha");
    let dir_b = create_test_directory(&ctx.working_path, "beta");
    let dir_c = create_test_directory(&ctx.working_path, "gamma");
    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_c.to_str().unwrap(), 1, &[], &[]);

    let output = list_dirs_with_output_filter(&ctx.db_path, &[]);
    assert!(
        output.status.success(),
        "list --output-filter -d with no keywords should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3, "All 3 dirs should be shown, got: {lines:?}");
}

#[test]
fn test_list_output_filter_dirs_keyword_shows_all_matches() {
    let ctx = TestContext::new();

    let dir_a = create_test_directory(&ctx.working_path, "projects_a");
    let dir_b = create_test_directory(&ctx.working_path, "projects_b");
    let dir_other = create_test_directory(&ctx.working_path, "other");
    note_path(&ctx.db_path, None, dir_a.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_b.to_str().unwrap(), 1, &[], &[]);
    note_path(&ctx.db_path, None, dir_other.to_str().unwrap(), 1, &[], &[]);

    let output = list_dirs_with_output_filter(&ctx.db_path, &["--", "projects"]);
    assert!(
        output.status.success(),
        "list --output-filter -d projects should succeed with multiple matches"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Both 'projects_a' and 'projects_b' should appear, got: {lines:?}"
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

#[test]
fn test_list_output_filter_dirs_excludes_files() {
    let ctx = TestContext::new();

    let file = create_test_file(&ctx.working_path, "notes.txt", "content");
    note_path(&ctx.db_path, None, file.to_str().unwrap(), 1, &[], &[]);

    let output = list_dirs_with_output_filter(&ctx.db_path, &["--", "notes"]);
    assert!(
        !output.status.success(),
        "list --output-filter -d should not return files"
    );
}

#[test]
fn test_list_output_filter_no_filter_fails() {
    let ctx = TestContext::new();

    let dir = create_test_directory(&ctx.working_path, "bar");
    note_path(&ctx.db_path, None, dir.to_str().unwrap(), 1, &[], &[]);

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &[
            "--config",
            "import_on_first_use=false",
            "list",
            "--output-filter",
            "-d",
            "--",
            "bar",
        ],
        vec![("MEMY_OUTPUT_FILTER", ""), ("PATH", "")],
    );

    assert!(
        !output.status.success(),
        "list --output-filter -d should fail when no output filter is available"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No output filter command found"),
        "Should report missing filter, got: {stderr}"
    );
}

fn list_dirs_with_output_filter(
    db_path: &std::path::Path,
    list_args: &[&str],
) -> std::process::Output {
    let mut args = vec![
        "--config",
        "import_on_first_use=false",
        "list",
        "--output-filter",
        "-d",
    ];
    args.extend(list_args);
    memy_cmd(
        Some(db_path),
        None,
        &args,
        vec![("MEMY_OUTPUT_FILTER", "cat")],
    )
}
