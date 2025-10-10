use chrono::{DateTime, Local, TimeZone as _};
use std::env::home_dir;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::UnixTimestamp;

pub fn get_unix_timestamp() -> UnixTimestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Couldn't get seconds since epoch")
        .as_secs()
}

pub fn timestamp_to_iso8601(timestamp: UnixTimestamp) -> String {
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

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();

    if let Some(Component::Normal(first)) = p.components().next() {
        if first == "~" {
            if let Some(home) = home_dir() {
                let mut comps = p.components();
                comps.next();
                return home.join(comps.as_path());
            }
        }
    }

    p.to_path_buf()
}

pub fn collapse_to_tilde<P: AsRef<Path>>(path: P) -> String {
    let p = path.as_ref();

    if let Some(home) = home_dir() {
        if let Ok(stripped) = p.strip_prefix(&home) {
            if stripped.as_os_str().is_empty() {
                return "~".to_owned();
            }

            return PathBuf::from("~")
                .join(stripped)
                .to_string_lossy()
                .into_owned();
        }
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
    let kind = if ft.is_dir() {
        "dir"
    } else if ft.is_file() {
        "file"
    } else if ft.is_symlink() {
        "symlink"
    } else {
        "other"
    };
    s.serialize_str(kind)
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
}
