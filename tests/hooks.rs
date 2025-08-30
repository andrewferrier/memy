#![allow(clippy::unwrap_used)]

mod support;
use support::*;

#[test]
fn test_hook_lfrc() {
    let (_db_temp, db_path) = temp_dir();
    let output = memy_cmd(&db_path, None, &["hook", "lfrc"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    assert!(stdout.contains("cmd on-cd"));
}
