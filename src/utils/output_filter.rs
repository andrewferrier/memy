use log::debug;
use std::env;
use std::io::Write as _;
use std::process::{Command, Stdio};

use super::config;

use super::expand_tildes_in_multiline_string;
use super::is_command_available;

/// Returns the output filter command to use, checking (in order):
/// 1. An explicit override supplied by the caller
/// 2. The `MEMY_OUTPUT_FILTER` environment variable
/// 3. The `memy_output_filter` configuration option
/// 4. Auto-detected fuzzy finders (`fzf`, `sk`, `fzy`)
pub fn get_output_filter_command(override_cmd: Option<&str>) -> Option<String> {
    if let Some(cmd) = override_cmd {
        debug!("Output filter detected from command line: {cmd}");
        return Some(cmd.to_owned());
    }

    if let Ok(cmd) = env::var("MEMY_OUTPUT_FILTER")
        && !cmd.is_empty()
    {
        debug!("Output filter detected from environment: {cmd}");
        return Some(cmd);
    }

    if let Some(cmd) = config::get_memy_output_filter() {
        debug!("Output filter detected from config: {cmd}");
        return Some(cmd);
    }

    if is_command_available("fzf") {
        debug!("Output filter automatically set for fzf");
        return Some("fzf --ansi --tac".to_owned());
    }

    if is_command_available("sk") {
        debug!("Output filter automatically set for sk");
        return Some("sk --ansi --tac".to_owned());
    }

    if is_command_available("fzy") {
        debug!("Output filter automatically set for fzy");
        return Some("tac | fzy".to_owned());
    }

    None
}

/// Pipes `output` through the given shell `filter_cmd` and returns the
/// filter's stdout. Expands tildes in the returned string.
pub fn run_output_filter(
    output: &str,
    filter_cmd: &str,
) -> Result<String, Box<dyn core::error::Error>> {
    debug!("Running through external filter command {filter_cmd}");

    let shell = env::var("SHELL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "sh".to_owned());

    let mut cmd = Command::new(&shell)
        .arg("-c")
        .arg(filter_cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            format!(
                "Failed to execute output filter command via shell {shell:?} (command: {filter_cmd:?}): {err}"
            )
        })?;

    let stdin = cmd.stdin.as_mut().ok_or("Failed to open stdin")?;
    stdin.write_all(output.as_bytes())?;

    let output_data = cmd.wait_with_output()?;
    if !output_data.status.success() {
        let stderr = String::from_utf8_lossy(&output_data.stderr);
        return Err(format!(
            "Output filter command failed via shell {shell:?} with status {}: {}",
            output_data.status,
            stderr.trim()
        )
        .into());
    }

    let result = String::from_utf8(output_data.stdout)
        .map_err(|_| "Output filter output is not valid UTF-8")?;
    Ok(expand_tildes_in_multiline_string(&result))
}
