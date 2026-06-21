#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

use std::fs;

mod support;
use support::*;

#[test]
fn test_import_fasd_state_file() {
    let ctx = TestContext::new();

    let test_file_path_1 = create_test_file(&ctx.working_path, "test_file1", "test content");
    let test_file_path_2 = create_test_file(&ctx.working_path, "test_file2", "test content");

    let fasd_state_file = &ctx.cache_path.join("fasd");
    let fasd_contents = format!(
        "{}|10.5|1633036800\n{}|20.0|1633123200",
        test_file_path_1.to_string_lossy(),
        test_file_path_2.to_string_lossy()
    );
    fs::write(fasd_state_file, fasd_contents).expect("Failed to write mock fasd state file");

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["list"],
        vec![
            ("XDG_CACHE_HOME", ctx.cache_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the autojump state
            ("XDG_DATA_HOME", ctx.empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the zoxide state
            ("_ZO_DATA_DIR", ctx.empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the jumper state
            (
                "__JUMPER_FOLDERS",
                ctx.empty_path.join("jfolders").to_str().unwrap(),
            ),
            (
                "__JUMPER_FILES",
                ctx.empty_path.join("jfiles").to_str().unwrap(),
            ),
        ],
    );

    assert!(output.status.success());

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(
        &lines,
        &[
            test_file_path_1.to_str().unwrap(),
            test_file_path_2.to_str().unwrap(),
        ],
    );
}

#[test]
fn test_import_autojump_state_file() {
    let ctx = TestContext::new();

    let test_file_path_1 = create_test_file(&ctx.working_path, "test_file1", "test content");
    let test_file_path_2 = create_test_file(&ctx.working_path, "test_file2", "test content");

    let autojump_state_path = &ctx.data_path.join("autojump");
    fs::create_dir(autojump_state_path).unwrap();
    let autojump_state_file = autojump_state_path.join("autojump.txt");
    let autojump_contents = format!(
        "10.5\t{}\n20.0\t{}",
        test_file_path_1.to_string_lossy(),
        test_file_path_2.to_string_lossy()
    );
    fs::write(&autojump_state_file, autojump_contents)
        .expect("Failed to write mock autojump state file");

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["list", "-vv"],
        vec![
            // This makes sure the test does't accidentally import from the fasd state
            ("XDG_CACHE_HOME", ctx.empty_path.to_str().unwrap()),
            ("XDG_DATA_HOME", ctx.data_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the zoxide state
            ("_ZO_DATA_DIR", ctx.empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the jumper state
            (
                "__JUMPER_FOLDERS",
                ctx.empty_path.join("jfolders").to_str().unwrap(),
            ),
            (
                "__JUMPER_FILES",
                ctx.empty_path.join("jfiles").to_str().unwrap(),
            ),
        ],
    );

    assert!(output.status.success());

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(
        &lines,
        &[
            test_file_path_1.to_str().unwrap(),
            test_file_path_2.to_str().unwrap(),
        ],
    );
}

#[test]
fn test_import_jumper_state_files() {
    let ctx = TestContext::new();

    let test_dir_path = create_test_directory(&ctx.working_path, "test_dir");
    let test_file_path = create_test_file(&ctx.working_path, "test_file", "test content");

    let jumper_folders_file = &ctx.cache_path.join("jfolders");
    let jumper_folders_contents =
        format!("{}|0.600000|1780252584", test_dir_path.to_string_lossy());
    fs::write(jumper_folders_file, jumper_folders_contents)
        .expect("Failed to write mock jumper folders file");

    let jumper_files_file = &ctx.cache_path.join("jfiles");
    let jumper_files_contents = format!("{}|1.900000|1780252589", test_file_path.to_string_lossy());
    fs::write(jumper_files_file, jumper_files_contents)
        .expect("Failed to write mock jumper files file");

    let output = memy_cmd(
        Some(&ctx.db_path),
        None,
        &["list"],
        vec![
            // This makes sure the test does't accidentally import from the fasd state
            ("XDG_CACHE_HOME", ctx.empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the autojump state
            ("XDG_DATA_HOME", ctx.empty_path.to_str().unwrap()),
            // This makes sure the test does't accidentally import from the zoxide state
            ("_ZO_DATA_DIR", ctx.empty_path.to_str().unwrap()),
            ("__JUMPER_FOLDERS", jumper_folders_file.to_str().unwrap()),
            ("__JUMPER_FILES", jumper_files_file.to_str().unwrap()),
        ],
    );

    assert!(output.status.success());

    let lines = list_paths(&ctx.db_path, None, &[], &[]);
    assert_lines_eq(
        &lines,
        &[
            test_dir_path.to_str().unwrap(),
            test_file_path.to_str().unwrap(),
        ],
    );
}
