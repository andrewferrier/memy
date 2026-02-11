use chrono::{DateTime, Local, TimeZone as _};
use std::borrow::Cow;
use std::env::home_dir;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Parse a "newer than" time string into a Unix timestamp.
/// Accepts both duration strings (e.g., "4d", "3h", "4d3h") and ISO-8601 date/time strings.
/// Returns the Unix timestamp representing the cutoff time.
pub fn parse_newer_than(input: &str) -> Result<UnixTimestamp, Box<dyn std::error::Error>> {
    // First try parsing as a duration using humantime
    if let Ok(duration) = humantime::parse_duration(input) {
        let now = get_unix_timestamp();
        let cutoff = now - duration.as_secs() as UnixTimestamp;
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
        let local_datetime = Local.from_local_datetime(&datetime).single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    // Try parsing as datetime without timezone
    if let Ok(naive_datetime) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        let local_datetime = Local.from_local_datetime(&naive_datetime).single()
            .ok_or("Failed to convert to local time")?;
        return Ok(local_datetime.timestamp());
    }

    Err(format!("Unable to parse '{}' as a duration or date/time", input).into())
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

pub fn expand_tilde<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let home = home_dir().expect("Could not get home dir");
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/"), home);
        assert_eq!(expand_tilde("~/memy"), home.join("memy"));
        assert_eq!(expand_tilde("~/memy/"), home.join("memy"));
        assert_eq!(expand_tilde("/etc/hosts"), PathBuf::from("/etc/hosts"));
        assert_eq!(expand_tilde("etc/hosts"), PathBuf::from("etc/hosts"));
        assert_eq!(expand_tilde("hosts"), PathBuf::from("hosts"));
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
            let expanded = expand_tilde(&normalized_path);
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
