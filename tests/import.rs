#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

use std::fs;

mod support;
use support::*;

#[test]
fn test_import_fasd_state_file() {
    let (_db_temp, db_path) = temp_dir();
    let (_cache_temp, cache_path) = temp_dir();
    let (_empty_temp, empty_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path_1 = create_test_file(&working_path, "test_file1", "test content");
    let test_file_path_2 = create_test_file(&working_path, "test_file2", "test content");

    let fasd_state_file = cache_path.join("fasd");
    let fasd_contents = format!(
        "{}|10.5|1633036800\n{}|20.0|1633123200",
        test_file_path_1.to_string_lossy(),
        test_file_path_2.to_string_lossy()
    );
    fs::write(&fasd_state_file, fasd_contents).expect("Failed to write mock fasd state file");

    let output = memy_cmd(
        &db_path,
        None,
        &["list"],
        vec![
            ("XDG_CACHE_HOME", cache_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the autojump state
            ("XDG_DATA_HOME", empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the zoxide state
            ("_ZO_DATA_DIR", empty_path.to_str().unwrap()),
        ],
    )
    .output()
    .expect("wibble");

    assert!(output.status.success());

    let lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], test_file_path_1.to_string_lossy());
    assert_eq!(lines[1], test_file_path_2.to_string_lossy());
}

#[test]
fn test_import_autojump_state_file() {
    let (_db_temp, db_path) = temp_dir();
    let (_empty_temp, empty_path) = temp_dir();
    let (_data_temp, data_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path_1 = create_test_file(&working_path, "test_file1", "test content");
    let test_file_path_2 = create_test_file(&working_path, "test_file2", "test content");

    let autojump_state_path = data_path.join("autojump");
    fs::create_dir(&autojump_state_path).unwrap();
    let autojump_state_file = autojump_state_path.join("autojump.txt");
    let autojump_contents = format!(
        "10.5\t{}\n20.0\t{}",
        test_file_path_1.to_string_lossy(),
        test_file_path_2.to_string_lossy()
    );
    fs::write(&autojump_state_file, autojump_contents)
        .expect("Failed to write mock autojump state file");

    let output = memy_cmd(
        &db_path,
        None,
        &["list", "-vv"],
        vec![
            // This makes sure the test does't accidentally import from the fasd state
            ("XDG_CACHE_HOME", empty_path.to_str().unwrap()),
            ("XDG_DATA_HOME", data_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the zoxide state
            ("_ZO_DATA_DIR", empty_path.to_str().unwrap()),
        ],
    )
    .output()
    .expect("Failed to execute memy list command");

    assert!(output.status.success());

    let lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], test_file_path_1.to_string_lossy());
    assert_eq!(lines[1], test_file_path_2.to_string_lossy());
}
