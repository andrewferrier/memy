#![allow(clippy::unwrap_used)]

mod support;
use support::*;

#[test]
fn test_generate_config_creates_file_if_not_exists() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp, config_path) = temp_dir();
    let config_file = config_path.join("memy.toml");

    let output = memy_cmd(&db_path, Some(&config_path), &["generate-config"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(config_file.exists());
    let contents = std::fs::read_to_string(&config_file).unwrap();
    assert!(contents.contains("normalize_symlinks_on_note"));
}

#[test]
fn test_generate_config_errors_if_file_exists() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp, config_path) = temp_dir();
    let config_file = config_path.join("memy.toml");
    std::fs::write(&config_file, "dummy = true").unwrap();

    let output = memy_cmd(&db_path, Some(&config_path), &["generate-config"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Config file already exists"));
}

#[test]
fn test_generate_config_explicit_filename_even_if_default_exists() {
    let (_db_temp, db_path) = temp_dir();
    let (_config_temp, config_path) = temp_dir();
    let config_file = config_path.join("memy.toml");
    let explicit_file = config_path.join("custom_config.toml");
    std::fs::write(&config_file, "dummy = true").unwrap();

    let output = memy_cmd(
        &db_path,
        Some(&config_path),
        &["generate-config", &explicit_file.to_string_lossy()],
    )
    .output()
    .expect("Failed to execute command");

    assert!(output.status.success());
    assert!(explicit_file.exists());
    let contents = std::fs::read_to_string(&explicit_file).unwrap();
    assert!(contents.contains("normalize_symlinks_on_note"));
}
