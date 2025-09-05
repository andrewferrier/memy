#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

use std::env;
use std::fs;

mod support;
use support::*;

#[test]
fn test_import_fasd_state_file() {
    let (_db_temp, db_path) = temp_dir();
    let (_cache_temp, cache_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let test_file_path_1 = create_test_file(&working_path, "test_file1", "test content");
    let test_file_path_2 = create_test_file(&working_path, "test_file2", "test content");

    env::set_var("XDG_CACHE_HOME", cache_path.to_str().unwrap());

    let fasd_state_file = cache_path.join("fasd");
    let fasd_contents = format!(
        "{}|10.5|1633036800\n{}|20.0|1633123200",
        test_file_path_1.to_string_lossy(),
        test_file_path_2.to_string_lossy()
    );
    fs::write(&fasd_state_file, fasd_contents).expect("Failed to write mock fasd state file");

    let output = memy_cmd(&db_path, None, &["list"])
        .output()
        .expect("Failed to execute memy list command");

    assert!(output.status.success());

    let lines = list_paths(&db_path, None, &[], &[]);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], test_file_path_1.to_string_lossy());
    assert_eq!(lines[1], test_file_path_2.to_string_lossy());
}
