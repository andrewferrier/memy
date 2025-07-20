mod support;
use support::*;

use std::fs;

#[test]
fn test_denylist_excludes_file() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let deny_file = create_test_file(&working_path, "denyme.txt", "deny me");

    let deny_pattern = deny_file.to_str().unwrap();
    let config_contents = format!("denylist = [\"{deny_pattern}\"]\n");
    create_config_file(&config_path, &config_contents);

    let output = note_path(
        &db_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_excludes_file_with_subdir_glob() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let subdir = create_test_directory(&working_path, "subdir");
    let deny_file = create_test_file(&subdir, "denyme.txt", "deny me");

    let deny_pattern = format!("{}/*", subdir.to_str().unwrap());
    let config_contents = format!("denylist = [\"{deny_pattern}\"]\n");
    create_config_file(&config_path, &config_contents);

    let output = note_path(
        &db_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_excludes_file_with_double_star_glob() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let nested_dir = working_path.join("a/b/c");
    fs::create_dir_all(&nested_dir).expect("failed to create nested dirs");
    let deny_file = create_test_file(&nested_dir, "deny.txt", "deny me");

    let config_contents = "denylist = [\"**/deny.txt\"]\n";
    create_config_file(&config_path, config_contents);

    let output = note_path(
        &db_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_excludes_multiple_patterns() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let file1 = create_test_file(&working_path, "deny1.txt", "deny1");
    let file2 = create_test_file(&working_path, "deny2.txt", "deny2");

    let config_contents = format!(
        "denylist = [\"{}\", \"{}\"]\n",
        file1.to_str().unwrap(),
        file2.to_str().unwrap()
    );
    create_config_file(&config_path, &config_contents);

    let output1 = note_path(
        &db_path,
        Some(&config_path),
        file1.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");

    assert!(output1.status.success());
    let stderr1 = String::from_utf8_lossy(&output1.stderr);
    assert!(stderr1.contains("denied"));

    let output2 = note_path(
        &db_path,
        Some(&config_path),
        file2.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");

    assert!(output2.status.success());
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(stderr2.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_pattern_with_leading_double_star() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let file1 = create_test_file(&working_path, "notrooted.txt", "should not be denied");

    let config_contents = "denylist = [\"**/notrooted.txt\"]\n";
    create_config_file(&config_path, config_contents);

    let output = note_path(
        &db_path,
        Some(&config_path),
        file1.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("denied"));

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_denylist_excludes_file_no_warning_when_warn_disabled() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();
    let (_config_temp_dir, config_path) = temp_dir();

    let deny_file = create_test_file(&working_path, "denyme.txt", "deny me");

    let deny_pattern = deny_file.to_str().unwrap();
    let config_contents =
        format!("denylist = [\"{deny_pattern}\"]\ndenied_files_warn_on_note = false\n");
    create_config_file(&config_path, &config_contents);

    let output = note_path(
        &db_path,
        Some(&config_path),
        deny_file.to_str().unwrap(),
        1,
        false,
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty());

    let lines: Vec<String> = list_paths(&db_path, Some(&config_path), &[]);
    assert_eq!(lines.len(), 0);
}
