use crate::utils::path;

use colored::Colorize as _;
use std::borrow::Cow;
use std::env;
use std::io::Write as _;
use std::process::{Command, Stdio};
use tracing::debug;

use super::config;
use super::is_command_available;
use super::path::expand_tildes_in_multiline_string;

fn format_path_colored(path: &str, is_dir: bool) -> String {
    let display: Cow<str> = if config::get_use_tilde_on_list() {
        Cow::Owned(path::collapse_to_tilde(path))
    } else {
        Cow::Borrowed(path)
    };

    if let Some((parent, base)) = display.rsplit_once('/') {
        if is_dir {
            format!("{}/{}", parent, base.blue())
        } else {
            format!("{}/{}", parent, base.green())
        }
    } else if is_dir {
        display.blue().to_string()
    } else {
        display.green().to_string()
    }
}

pub fn format_paths_colored<'a>(items: impl Iterator<Item = (&'a str, bool)>) -> String {
    let mut output = String::new();
    for (path, is_dir) in items {
        output.push_str(&format_path_colored(path, is_dir));
        output.push('\n');
    }
    output
}

fn get_output_filter_command(override_cmd: Option<&str>) -> Result<String, &str> {
    if let Some(cmd) = override_cmd {
        debug!("Output filter detected from command line: {cmd}");
        return Ok(cmd.to_owned());
    }

    if let Ok(cmd) = env::var("MEMY_OUTPUT_FILTER")
        && !cmd.is_empty()
    {
        debug!("Output filter detected from environment: {cmd}");
        return Ok(cmd);
    }

    if let Some(cmd) = config::get_memy_output_filter() {
        debug!("Output filter detected from config: {cmd}");
        return Ok(cmd);
    }

    if is_command_available("fzf") {
        debug!("Output filter automatically set for fzf");
        return Ok("fzf --ansi --tac".to_owned());
    }

    if is_command_available("sk") {
        debug!("Output filter automatically set for sk");
        return Ok("sk --ansi --tac".to_owned());
    }

    if is_command_available("fzy") {
        debug!("Output filter automatically set for fzy");
        return Ok("tac | fzy".to_owned());
    }

    Err(
        "No output filter command found. Set MEMY_OUTPUT_FILTER environment variable, \
         memy_output_filter in config, or install fzf/sk/fzy.",
    )
}

pub fn pipe_through_filter(
    input: &str,
    override_cmd: Option<&str>,
) -> Result<String, Box<dyn core::error::Error>> {
    let filter_cmd_string = get_output_filter_command(override_cmd)?;
    let filter_cmd = filter_cmd_string.as_str();

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
    stdin.write_all(input.as_bytes())?;

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
