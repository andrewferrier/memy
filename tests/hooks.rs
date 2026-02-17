#![allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]

mod support;
use support::*;

#[test]
fn test_hook_lfrc() {
    let output = memy_cmd(None, None, &["hook", "lfrc"], vec![]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");
    assert!(stdout.contains("cmd on-cd"));
}

#[test]
fn test_hook_fish() {
    let output = memy_cmd(None, None, &["hook", "fish.fish"], vec![]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in output");

    // Verify the hook contains the fish_preexec function
    assert!(stdout.contains("function fish_preexec"));
    assert!(stdout.contains("--on-event fish_preexec"));

    // Verify it does NOT use the unsafe eval printf pattern
    // This pattern causes stray lines when running commands like:
    //   memy hook fish.fish >> ~/.config/fish/config.fish
    // in an interactive fish shell with the hook already loaded.
    // The eval causes the command arguments to be written to the file.
    assert!(!stdout.contains("eval printf"),
        "Fish hook should not use 'eval printf' as it evaluates shell operators like redirections");

    // Verify it does NOT use eval echo either
    assert!(!stdout.contains("eval echo"),
        "Fish hook should not use 'eval echo' as it evaluates shell operators");

    // Verify it uses safe word splitting with $cmd
    assert!(stdout.contains("for word in $cmd"),
        "Fish hook should use safe word splitting with 'for word in $cmd'");

    // Verify the memy-cd function exists
    assert!(stdout.contains("function memy-cd"));
}
