#![allow(clippy::unwrap_used)]

mod support;
use std::os::unix::fs::symlink;
use support::*;

#[test]
fn test_note_symlink_with_no_normalize_option() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let dummy_file_path = create_test_file(&working_path, "dummy_file_A", "dummy content");
    let symlink_path = working_path.join("symlink_B");
    symlink(&dummy_file_path, &symlink_path).expect("failed to create symlink");

    let config_contents = "normalize_symlinks_on_note = false\n";
    create_config_file(&config_path, config_contents);

    note_path(
        &db_path,
        Some(&config_path),
        symlink_path.to_str().unwrap(),
        1,
        &[],
        &[],
    );

    let lines = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], symlink_path.to_str().unwrap());
}
