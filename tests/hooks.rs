#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

use std::path::Path;

enum Shell {
    Bash,
    Zsh,
    Fish,
}

const ALL_SHELLS: &[Shell] = &[Shell::Bash, Shell::Zsh, Shell::Fish];

impl Shell {
    const fn binary(&self) -> &str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
        }
    }

    const fn hook_filename(&self) -> &str {
        match self {
            Self::Bash => "hooks/bash",
            Self::Zsh => "hooks/zsh",
            Self::Fish => "hooks/fish.fish",
        }
    }

    /// Bash/zsh aliases aren't accessible in non-interactive mode.
    fn fn_name<'a>(&self, alias: &'a str) -> &'a str {
        match self {
            Self::Fish => alias,
            Self::Bash | Self::Zsh => match alias {
                "memy-cd" => "_memy_cd",
                "memy-open" => "_memy_open",
                "memy-go" => "_memy_go",
                other => other,
            },
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
        Shell::Bash | Shell::Fish => {
            format!("source {}; {command}", hook_path.display())
        }
    };

    let mut cmd = std::process::Command::new(shell.binary());
    cmd.args(["-c", &script]);
    run_hook(db_path, config_path, cmd)
}

/// Runs a shell script that sources the hook and then exercises one of the
/// `memy-cd` / `memy-open` / `memy-go` functions.
///
/// `selected_path` is injected as a fixed `MEMY_OUTPUT_FILTER` so the interactive
/// fzf step is bypassed completely. `cd` and `memy open` are replaced by thin
/// wrappers that emit `CD_CALLED:<path>` or `OPEN_CALLED:<path>` to stdout so
/// the test can assert on them without needing a GUI or a real application launcher.
///
/// Returns the combined stdout of the script (the wrapper lines).
fn run_shell_function_test(
    shell: &Shell,
    db_path: &Path,
    config_path: &Path,
    function_name: &str,
    selected_path: &str,
) -> std::process::Output {
    let hook_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(shell.hook_filename());
    let memy_bin = std::path::PathBuf::from(env!("CARGO_BIN_EXE_memy"));
    let memy_dir = memy_bin.parent().expect("memy binary has no parent dir");
    let original_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{original_path}", memy_dir.display());

    let script = match shell {
        Shell::Bash | Shell::Zsh => format!(
            r#"
real_memy="{memy}"
cd() {{ echo "CD_CALLED:$1"; }}
memy() {{
    if [[ "$1" == "open" ]]; then
        echo "OPEN_CALLED:$2"
    else
        "$real_memy" "$@"
    fi
}}
export MEMY_OUTPUT_FILTER='cat > /dev/null; printf "%s\n" "{selected}"'
source {hook}
{function_name}
"#,
            memy = memy_bin.display(),
            selected = selected_path,
            hook = hook_path.display(),
        ),
        Shell::Fish => format!(
            r#"
set real_memy "{memy}"
function cd; echo "CD_CALLED:$argv[1]"; end
function memy
    if test "$argv[1]" = open
        echo "OPEN_CALLED:$argv[2]"
    else
        $real_memy $argv
    end
end
set -x MEMY_OUTPUT_FILTER 'cat > /dev/null; printf "%s\n" "{selected}"'
source {hook}
{function_name}
"#,
            memy = memy_bin.display(),
            selected = selected_path,
            hook = hook_path.display(),
        ),
    };

    let mut cmd = std::process::Command::new(shell.binary());
    cmd.args(["-c", &script])
        .env("MEMY_DB_DIR", db_path)
        .env("MEMY_CONFIG_DIR", config_path)
        .env("PATH", &new_path);
    cmd.output().expect("failed to run shell function test")
}

#[test]
fn test_bash_hook_notes_file() {
    let ctx = TestContext::new();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&ctx.config_path, "import_on_first_use = false");

    let test_file = create_test_file(&data_path, "target.txt", "hello");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &ctx.db_path,
        &ctx.config_path,
        &format!("ls {}", test_file.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_lines_eq(&paths, &[test_file.to_string_lossy().as_ref()]);
}

#[test]
fn test_bash_hook_does_not_note_nonexistent_args() {
    let ctx = TestContext::new();

    create_config_file(&ctx.config_path, "import_on_first_use = false");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &ctx.db_path,
        &ctx.config_path,
        "echo hello nonexistent_path_xyz_abc",
    );
    assert!(output.status.success());

    let paths = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert!(
        paths.is_empty(),
        "expected no paths to be noted, got: {paths:?}"
    );
}

#[test]
fn test_bash_hook_notes_cd_target() {
    let ctx = TestContext::new();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&ctx.config_path, "import_on_first_use = false");

    let test_dir = create_test_directory(&data_path, "subdir");

    let output = run_shell_with_hook(
        &Shell::Bash,
        &ctx.db_path,
        &ctx.config_path,
        &format!("cd {}", test_dir.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_lines_eq(&paths, &[test_dir.to_string_lossy().as_ref()]);
}

#[test]
fn test_zsh_hook_notes_file() {
    let ctx = TestContext::new();
    let (_data_dir, data_path) = temp_dir();

    create_config_file(&ctx.config_path, "import_on_first_use = false");

    let test_file = create_test_file(&data_path, "target.txt", "hello");

    let output = run_shell_with_hook(
        &Shell::Zsh,
        &ctx.db_path,
        &ctx.config_path,
        &format!("ls {}", test_file.display()),
    );
    assert!(output.status.success());

    let paths = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_lines_eq(&paths, &[test_file.to_string_lossy().as_ref()]);
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
    let ctx = TestContext::new();

    create_config_file(&ctx.config_path, "import_on_first_use = false");

    let test_file = create_test_file(&ctx.data_path, "target.txt", "hello");

    let output = run_nvim_with_hook(&ctx.db_path, &ctx.config_path, &test_file);
    assert!(
        output.status.success(),
        "nvim failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let paths = list_paths(&ctx.db_path, Some(&ctx.config_path), &[], &[]);
    assert_lines_eq(&paths, &[test_file.to_string_lossy().as_ref()]);
}

fn assert_cd_called(output: &std::process::Output, path: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("CD_CALLED:{path}")),
        "expected CD_CALLED:{path} in stdout, got:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_open_called(output: &std::process::Output, path: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&format!("OPEN_CALLED:{path}")),
        "expected OPEN_CALLED:{path} in stdout, got:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_neither_called(output: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("CD_CALLED:") && !stdout.contains("OPEN_CALLED:"),
        "expected neither CD_CALLED nor OPEN_CALLED in stdout, got:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn do_test_memy_cd_selects_directory(shell: &Shell) {
    let ctx = TestContext::new();
    let selected = ctx.data_path.to_string_lossy().into_owned();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-cd"),
        &selected,
    );
    assert_cd_called(&output, &selected);
}

fn do_test_memy_cd_no_selection(shell: &Shell) {
    let ctx = TestContext::new();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-cd"),
        "",
    );
    assert_neither_called(&output);
}

fn do_test_memy_open_selects_file(shell: &Shell) {
    let ctx = TestContext::new();
    let test_file = create_test_file(&ctx.data_path, "target.txt", "hello");
    let selected = test_file.to_string_lossy().into_owned();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-open"),
        &selected,
    );
    assert_open_called(&output, &selected);
}

fn do_test_memy_open_no_selection(shell: &Shell) {
    let ctx = TestContext::new();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-open"),
        "",
    );
    assert_neither_called(&output);
}

fn do_test_memy_go_directory_branch(shell: &Shell) {
    let ctx = TestContext::new();
    let selected = ctx.data_path.to_string_lossy().into_owned();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-go"),
        &selected,
    );
    assert_cd_called(&output, &selected);
}

fn do_test_memy_go_file_branch(shell: &Shell) {
    let ctx = TestContext::new();
    let test_file = create_test_file(&ctx.data_path, "target.txt", "hello");
    let selected = test_file.to_string_lossy().into_owned();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-go"),
        &selected,
    );
    assert_open_called(&output, &selected);
}

fn do_test_memy_go_no_selection(shell: &Shell) {
    let ctx = TestContext::new();
    let output = run_shell_function_test(
        shell,
        &ctx.db_path,
        &ctx.config_path,
        shell.fn_name("memy-go"),
        "",
    );
    assert_neither_called(&output);
}

#[test]
fn test_memy_cd_selects_directory() {
    for shell in ALL_SHELLS {
        do_test_memy_cd_selects_directory(shell);
    }
}

#[test]
fn test_memy_cd_no_selection() {
    for shell in ALL_SHELLS {
        do_test_memy_cd_no_selection(shell);
    }
}

#[test]
fn test_memy_open_selects_file() {
    for shell in ALL_SHELLS {
        do_test_memy_open_selects_file(shell);
    }
}

#[test]
fn test_memy_open_no_selection() {
    for shell in ALL_SHELLS {
        do_test_memy_open_no_selection(shell);
    }
}

#[test]
fn test_memy_go_directory_branch() {
    for shell in ALL_SHELLS {
        do_test_memy_go_directory_branch(shell);
    }
}

#[test]
fn test_memy_go_file_branch() {
    for shell in ALL_SHELLS {
        do_test_memy_go_file_branch(shell);
    }
}

#[test]
fn test_memy_go_no_selection() {
    for shell in ALL_SHELLS {
        do_test_memy_go_no_selection(shell);
    }
}
