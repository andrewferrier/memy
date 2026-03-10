#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::path::Path;

enum Shell {
    Bash,
    Zsh,
}

impl Shell {
    const fn binary(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
        }
    }

    const fn hook_filename(&self) -> &str {
        match self {
            Self::Bash => "hooks/bash",
            Self::Zsh => "hooks/zsh",
        }
    }
}

#[test]
fn test_hook_lfrc() {
    let output = memy_cmd(None, None, &["hook", "lfrc"], vec![]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    assert!(stdout.contains("cmd on-cd"));
}

fn run_hook(
    db_path: &Path,
    config_path: &Path,
    mut cmd: std::process::Command,
) -> std::process::Output {
    let memy_bin = std::path::PathBuf::from(env!("CARGO_BIN_EXE_memy"));
    let memy_dir = memy_bin.parent().expect("memy binary has no parent dir");

    let original_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{original_path}", memy_dir.display());

    cmd.env("MEMY_DB_DIR", db_path)
        .env("MEMY_CONFIG_DIR", config_path)
        .env("PATH", &new_path)
        .output()
        .expect("failed to run hook process")
}

fn run_shell_with_hook(
    shell: &Shell,
    db_path: &Path,
    config_path: &Path,
    command: &str,
) -> std::process::Output {
    let hook_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(shell.hook_filename());

    // zsh's preexec hook only fires in interactive sessions, so call memy_preexec
    // directly. The hook also runs `memy note` asynchronously with &!, so sleep
    // briefly to let it complete before the shell exits.
    let script = match shell {
        Shell::Zsh => format!(
            "source {}; memy_preexec '{}'; sleep 0.3",
            hook_path.display(),
            command
        ),
        Shell::Bash => format!("source {}; {command}", hook_path.display()),
    };

    let mut cmd = std::process::Command::new(shell.binary());
    cmd.args(["-c", &script]);
    run_hook(db_path, config_path, cmd)
}

#[test]
fn test_bash_hook_notes_file() {
    let (_db_dir, db_path) = temp_dir();
    let (_config_dir, config_path) = temp_dir();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&config_path, "import_on_first_use = false");

    let test_file = create_test_file(&data_path, "target.txt", "hello");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &db_path,
        &config_path,
        &format!("ls {}", test_file.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert_eq!(
        paths,
        [test_file.to_string_lossy().as_ref()],
        "expected only {test_file:?} to be noted"
    );
}

#[test]
fn test_bash_hook_does_not_note_nonexistent_args() {
    let (_db_dir, db_path) = temp_dir();
    let (_config_dir, config_path) = temp_dir();

    create_config_file(&config_path, "import_on_first_use = false");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &db_path,
        &config_path,
        "echo hello nonexistent_path_xyz_abc",
    );
    assert!(output.status.success());

    let paths = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert!(
        paths.is_empty(),
        "expected no paths to be noted, got: {paths:?}"
    );
}

#[test]
fn test_bash_hook_notes_cd_target() {
    let (_db_dir, db_path) = temp_dir();
    let (_config_dir, config_path) = temp_dir();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&config_path, "import_on_first_use = false");

    let test_dir = create_test_directory(&data_path, "subdir");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &db_path,
        &config_path,
        &format!("cd {}", test_dir.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert_eq!(
        paths,
        [test_dir.to_string_lossy().as_ref()],
        "expected only {test_dir:?} to be noted after cd"
    );
}

#[test]
fn test_zsh_hook_notes_file() {
    let (_db_dir, db_path) = temp_dir();
    let (_config_dir, config_path) = temp_dir();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&config_path, "import_on_first_use = false");

    let test_file = create_test_file(&data_path, "target.txt", "hello");

    let output = run_shell_with_hook(
        &Shell::Zsh,
        &db_path,
        &config_path,
        &format!("ls {}", test_file.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert_eq!(
        paths,
        [test_file.to_string_lossy().as_ref()],
        "expected only {test_file:?} to be noted"
    );
}

fn run_nvim_with_hook(
    db_path: &Path,
    config_path: &Path,
    file_to_open: &Path,
) -> std::process::Output {
    let hook_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("hooks/neovim.lua");

    // Open the file headlessly with the hook loaded. The hook calls `memy note`
    // with detach=true, so sleep briefly inside nvim to let it complete before quit.
    let mut cmd = std::process::Command::new("nvim");
    cmd.args([
        "--headless",
        "-u",
        "NONE",
        "--noplugin",
        "--cmd",
        &format!("luafile {}", hook_path.display()),
        file_to_open
            .to_str()
            .expect("test path must be valid UTF-8"),
        "-c",
        "sleep 500m",
        "-c",
        "quit",
    ]);
    run_hook(db_path, config_path, cmd)
}

#[test]
fn test_neovim_hook_notes_opened_file() {
    let (_db_dir, db_path) = temp_dir();
    let (_config_dir, config_path) = temp_dir();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&config_path, "import_on_first_use = false");

    let test_file = create_test_file(&data_path, "target.txt", "hello");

    let output = run_nvim_with_hook(&db_path, &config_path, &test_file);
    assert!(
        output.status.success(),
        "nvim failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = list_paths(&db_path, Some(&config_path), &[], &[]);
    assert_eq!(
        paths,
        [test_file.to_string_lossy().as_ref()],
        "expected only {test_file:?} to be noted"
    );
}
