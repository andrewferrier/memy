#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_generate_config_outputs_template() {
    let (_db_temp, db_path) = temp_dir();

    let output = memy_cmd(&db_path, None, &["generate-config"], vec![])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Failed to parse stdout as UTF-8");
    assert!(
        stdout.contains("normalize_symlinks_on_note"),
        "Template config not found in output"
    );
}
