#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_list_default_sort_is_descending() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_eq!(lines.len(), 2, "Should have 2 files");
    assert!(
        lines[0].contains("file_b"),
        "Most frecent (file_b) should be first with default descending sort"
    );
    assert!(
        lines[1].contains("file_a"),
        "Least frecent (file_a) should be last"
    );
}

#[test]
fn test_list_sort_descending_explicit() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--sort", "descending"]);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("file_b"), "Most frecent should be first");
    assert!(lines[1].contains("file_a"), "Least frecent should be last");
}

#[test]
fn test_list_sort_ascending_explicit() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--sort", "ascending"]);
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].contains("file_a"),
        "Least frecent (file_a) should be first with ascending sort"
    );
    assert!(
        lines[1].contains("file_b"),
        "Most frecent (file_b) should be last"
    );
}

#[test]
fn test_list_sort_config_default_ascending() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");

    note_paths_with_delay(&ctx.db_path, Some(&ctx.config_path), &[&file_a, &file_b]);

    create_config_file(&ctx.config_path, "default_sort = \"ascending\"\n");

    let lines = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(
        lines[0].contains("file_a"),
        "Config default_sort=ascending: least frecent should be first"
    );
}

#[test]
fn test_list_sort_flag_overrides_config() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");

    note_paths_with_delay(&ctx.db_path, Some(&ctx.config_path), &[&file_a, &file_b]);

    // Config says ascending, but --sort descending should override
    create_config_file(&ctx.config_path, "default_sort = \"ascending\"\n");

    let lines = list_paths(
        &ctx.db_path,
        Some(&ctx.config_path),
        &[],
        &["--sort", "descending"],
    );
    assert!(
        lines[0].contains("file_b"),
        "--sort descending should override config default_sort=ascending"
    );
}

#[test]
fn test_limit_results_flag_works() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");
    let file_c = create_test_file(&ctx.working_path, "file_c.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--limit-results", "2"]);
    assert_eq!(lines.len(), 2, "--limit-results 2 should return 2 results");
}

#[test]
fn test_head_alias_still_works() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");
    let file_c = create_test_file(&ctx.working_path, "file_c.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--head", "2"]);
    assert_eq!(
        lines.len(),
        2,
        "--head 2 (hidden alias) should still return 2 results"
    );
}

#[test]
fn test_limit_results_descending_returns_most_frecent() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");
    let file_c = create_test_file(&ctx.working_path, "file_c.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let lines = list_paths(&ctx.db_path, None, &[], &["--limit-results", "1"]);
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].contains("file_c"),
        "--limit-results 1 with descending should return the single most frecent item (file_c)"
    );
}

#[test]
fn test_limit_results_ascending_returns_least_frecent() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "file_a.txt", "content");
    let file_b = create_test_file(&ctx.working_path, "file_b.txt", "content");
    let file_c = create_test_file(&ctx.working_path, "file_c.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let lines = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["--sort", "ascending", "--limit-results", "1"],
    );
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].contains("file_a"),
        "--limit-results 1 with ascending should return the single least frecent item (file_a)"
    );
}

#[test]
fn test_list_limit_results_with_keyword() {
    let ctx = TestContext::new();

    let file_a = create_test_file(&ctx.working_path, "main_proj.rs", "fn main() {}");
    let file_b = create_test_file(&ctx.working_path, "lib_proj.rs", "pub mod foo;");
    let file_c = create_test_file(&ctx.working_path, "unrelated.txt", "content");

    note_paths_with_delay(&ctx.db_path, None, &[&file_a, &file_b, &file_c]);

    let all_matching = list_paths(&ctx.db_path, None, &[], &["--", "proj"]);
    assert_eq!(all_matching.len(), 2, "Should have 2 matching files");

    let head_one = list_paths(
        &ctx.db_path,
        None,
        &[],
        &["--limit-results", "1", "--", "proj"],
    );
    assert_eq!(
        head_one.len(),
        1,
        "Should return 1 matching file with --limit-results 1"
    );
}
