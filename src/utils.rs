use chrono::{DateTime, Local, TimeZone as _};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_secs_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't get seconds since epoch")
        .as_secs()
}

pub fn timestamp_to_iso8601(timestamp: u64) -> String {
    let datetime: DateTime<Local> = Local
        .timestamp_opt(timestamp.try_into().expect("Can't convert timestamp"), 0)
        .single()
        .expect("Can't convert timestamp");
    datetime.to_rfc3339()
}

pub fn detect_shell() -> Option<clap_complete::Shell> {
    std::env::var("SHELL").ok().as_deref().and_then(|path| {
        let name = Path::new(path)
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(str::to_lowercase)?;

        match name.as_str() {
            "bash" => Some(clap_complete::Shell::Bash),
            "zsh" => Some(clap_complete::Shell::Zsh),
            "fish" => Some(clap_complete::Shell::Fish),
            "powershell" => Some(clap_complete::Shell::PowerShell),
            "elvish" => Some(clap_complete::Shell::Elvish),
            _ => None,
        }
    })
}
