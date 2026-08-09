#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use rusqlite::Connection;

const BREAKING_CHANGE_SORT_WARNING: &str = "memy list' now outputs most-frecent-first by default";

fn create_v1_db(db_path: &std::path::Path) {
    let db_file = db_path.join("memy.sqlite3");
    let conn = Connection::open(&db_file).expect("Failed to open test DB");
    conn.execute(
        "CREATE TABLE paths (
            path TEXT PRIMARY KEY,
            noted_count INTEGER NOT NULL,
            last_noted_timestamp INTEGER NOT NULL
        )",
        [],
    )
    .expect("Failed to create paths table");
    conn.execute("PRAGMA user_version = 1;", [])
        .expect("Failed to set user_version to 1");
    conn.close().expect("Failed to close DB");
}

fn run_list_in_terminal(db_path: &std::path::Path) -> std::process::Output {
    memy_cmd_force_terminal(
        Some(db_path),
        None,
        &["--config", "import_on_first_use=false", "list"],
        vec![],
    )
}

#[test]
fn test_db_fresh_install_does_not_warn() {
    let ctx = TestContext::new();

    let output = run_list_in_terminal(&ctx.db_path);
    assert!(output.status.success(), "List command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(BREAKING_CHANGE_SORT_WARNING),
        "Fresh installs should not warn, got: {stdout}"
    );
}

#[test]
fn test_db_migration_warns_for_first_10_terminal_list_runs() {
    let ctx = TestContext::new();
    create_v1_db(&ctx.db_path);

    let mut warning_seen = Vec::new();

    for _ in 0..11 {
        let output = run_list_in_terminal(&ctx.db_path);
        assert!(output.status.success(), "List command should succeed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        warning_seen.push(stdout.contains(BREAKING_CHANGE_SORT_WARNING));
    }

    assert!(
        warning_seen.iter().take(10).all(|seen| *seen),
        "The first 10 list runs should warn: {warning_seen:?}"
    );
    assert!(
        !warning_seen[10],
        "The 11th list run should not warn: {warning_seen:?}"
    );
}
