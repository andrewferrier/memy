use chrono::{DateTime, Local, TimeZone as _};
use colored::Colorize as _;
use log::debug;
use std::borrow::Cow;
use std::env;
use std::env::home_dir;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config;
use crate::types::UnixTimestamp;
use crate::types::UnixTimestampHours;

#[allow(
    clippy::cast_possible_wrap,
    reason = "Value is never going to be large enough in practice that it can't be cast"
)]
pub fn get_unix_timestamp() -> UnixTimestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as UnixTimestamp
}

pub fn timestamp_to_iso8601(timestamp: UnixTimestamp) -> String {
    let datetime: DateTime<Local> = Local
        .timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(|| panic!("Can't convert timestamp {timestamp}"));

    datetime.to_rfc3339()
}

pub fn timestamp_age_hours(now: UnixTimestamp, timestamp: UnixTimestamp) -> UnixTimestampHours {
    let age_seconds = now - timestamp;
    age_seconds as f64 / 3600.0
}

pub fn parse_newer_than(input: &str) -> Result<UnixTimestamp, Box<dyn core::error::Error>> {
    // First try parsing as a duration using humantime
    if let Ok(duration) = humantime::parse_duration(input) {
        let now = get_unix_timestamp();
        let cutoff = now - duration.as_secs().cast_signed() as UnixTimestamp;
        return Ok(cutoff);
    }

    // Try parsing as ISO-8601 datetime
    if let Ok(datetime) = DateTime::parse_from_rfc3339(input) {
        return Ok(datetime.timestamp());
    }

    // Try parsing as a date-only string (partial ISO-8601)
    // Handle formats like "2025-01-01"
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let datetime = naive_date
            .and_hms_opt(0, 0, 0)
            .ok_or("Failed to create datetime")?;
        let local_datetime = Local
            .from_local_datetime(&datetime)
            .single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    // Try parsing as datetime without timezone
    if let Ok(naive_datetime) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        let local_datetime = Local
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    Err(format!("Unable to parse '{input}' as a duration or date/time").into())
}

pub fn detect_shell() -> Option<clap_complete::Shell> {
    let name = std::env::var("SHELL")
        .ok()
        .and_then(|path| path.rsplit('/').next().map(str::to_lowercase))?;

    match name.as_str() {
        "bash" => Some(clap_complete::Shell::Bash),
        "zsh" => Some(clap_complete::Shell::Zsh),
        "fish" => Some(clap_complete::Shell::Fish),
        "powershell" => Some(clap_complete::Shell::PowerShell),
        "elvish" => Some(clap_complete::Shell::Elvish),
        _ => None,
    }
}

pub fn is_command_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
}

pub fn expand_tilde_in_path<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
    let p = path.as_ref();

    if let Some(Component::Normal(first)) = p.components().next()
        && first == "~"
        && let Some(home) = home_dir()
    {
        let mut comps = p.components();
        comps.next(); // skip "~"
        let expanded = home.join(comps.as_path());
        Cow::Owned(expanded)
    } else {
        Cow::Borrowed(p)
    }
}

fn expand_tilde_in_string(line: &str) -> Cow<'_, str> {
    if (line == "~" || line.starts_with("~/"))
        && let Ok(home) = env::var("HOME")
    {
        return Cow::Owned(format!("{home}{}", &line[1..]));
    }

    Cow::Borrowed(line)
}

pub fn expand_tildes_in_multiline_string(text: &str) -> String {
    let had_trailing_newline = text.ends_with('\n');

    let mut expanded = text
        .lines()
        .map(expand_tilde_in_string)
        .collect::<Vec<_>>()
        .join("\n");

    if had_trailing_newline {
        expanded.push('\n');
    }

    expanded
}

pub fn collapse_to_tilde<P: AsRef<Path>>(path: P) -> String {
    let p = path.as_ref();

    if let Some(home) = home_dir()
        && let Ok(stripped) = p.strip_prefix(&home)
    {
        if stripped.as_os_str().is_empty() {
            return "~".to_owned();
        }

        return PathBuf::from("~")
            .join(stripped)
            .to_string_lossy()
            .into_owned();
    }

    p.to_string_lossy().into_owned()
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "Reference required for Serialize"
)]
pub fn serialize_file_type<S>(ft: &std::fs::FileType, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(match (ft.is_dir(), ft.is_file(), ft.is_symlink()) {
        (true, _, _) => "dir",
        (_, true, _) => "file",
        (_, _, true) => "symlink",
        _ => "other",
    })
}

/// Returns a colored display string for a path. The last path component is
/// colored blue for directories and green for files.
pub fn format_path_colored(path: &str, is_dir: bool) -> String {
    let display: Cow<str> = if config::get_use_tilde_on_list() {
        Cow::Owned(collapse_to_tilde(path))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let home = home_dir().expect("Could not get home dir");
        assert_eq!(expand_tilde_in_path("~"), home);
        assert_eq!(expand_tilde_in_path("~/"), home);
        assert_eq!(expand_tilde_in_path("~/memy"), home.join("memy"));
        assert_eq!(expand_tilde_in_path("~/memy/"), home.join("memy"));
        assert_eq!(
            expand_tilde_in_path("/etc/hosts"),
            PathBuf::from("/etc/hosts")
        );
        assert_eq!(
            expand_tilde_in_path("etc/hosts"),
            PathBuf::from("etc/hosts")
        );
        assert_eq!(expand_tilde_in_path("hosts"), PathBuf::from("hosts"));
    }

    #[test]
    fn test_reduce_to_tilde() {
        let home = home_dir().expect("Could not get home dir");

        assert_eq!(collapse_to_tilde(&home), "~");
        assert_eq!(collapse_to_tilde(home.join("memy")), "~/memy");
        assert_eq!(collapse_to_tilde(home.join("memy/other")), "~/memy/other");
        assert_eq!(collapse_to_tilde("/etc/hosts"), "/etc/hosts");
        assert_eq!(collapse_to_tilde("etc/hosts"), "etc/hosts");
        assert_eq!(collapse_to_tilde("hosts"), "hosts");
    }

    #[test]
    fn test_expand_tildes_in_multiline_string() {
        let home = env::var("HOME").expect("HOME environment variable not set for test");

        assert_eq!(expand_tildes_in_multiline_string(""), "");
        assert_eq!(expand_tildes_in_multiline_string("~"), home);
        assert_eq!(expand_tildes_in_multiline_string("\n"), "\n");
        assert_eq!(
            expand_tildes_in_multiline_string("/etc/hosts"),
            "/etc/hosts"
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/config"),
            format!("{home}/config")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file1\n~/dir/file2"),
            format!("{home}/file1\n{home}/dir/file2")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("/absolute/path\nrelative/path"),
            "/absolute/path\nrelative/path"
        );
        assert_eq!(
            expand_tildes_in_multiline_string(
                "~/file1\n/absolute/path\n~/dir/file2\nrelative/path"
            ),
            format!("{home}/file1\n/absolute/path\n{home}/dir/file2\nrelative/path",)
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file~name"),
            format!("{home}/file~name")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file1\n/absolute/path\n"),
            format!("{home}/file1\n/absolute/path\n")
        );
    }

    use normalize_path::NormalizePath as _;
    use proptest::prelude::*;
    use proptest::strategy::Strategy;
    use proptest::string::string_regex;

    proptest! {
        #[test]
        fn round_trip_timestamp_serialization(timestamp in 0..=DateTime::parse_from_rfc3339("9999-12-31T23:59:59+00:00").expect("Cannot parse").timestamp()) {
            let iso8601 = timestamp_to_iso8601(timestamp);
            let parsed_datetime = DateTime::parse_from_rfc3339(&iso8601).unwrap_or_else(|_| panic!("Failed to parse ISO8601 string {iso8601}"));
            let round_trip_timestamp = parsed_datetime.timestamp();

            prop_assert_eq!(timestamp, round_trip_timestamp);
        }
    }

    proptest! {
        #[test]
        fn test_tilde_expand_collapse(path in generate_unix_path()) {
            // Path normalization is needed for test paths like `~/.`
            let normalized_path = Path::new(&path).normalize();
            let expanded = expand_tilde_in_path(&normalized_path);
            let collapsed = collapse_to_tilde(&expanded);

            prop_assert_eq!(collapsed, normalized_path.to_string_lossy());
        }
    }

    fn generate_unix_path() -> impl Strategy<Value = String> {
        let component_char = r"[^/]+"; // one or more chars except '/'
        let components = proptest::collection::vec(
            string_regex(component_char).expect("string_regex failed"),
            1..6,
        );
        let base_path = components.prop_map(|comps| comps.join("/"));
        base_path
            .prop_flat_map(|s| prop_oneof![Just(format!("~/{s}")), Just(format!("/{s}")), Just(s)])
    }
}
