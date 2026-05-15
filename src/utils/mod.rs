pub mod cli;
pub mod config;
pub mod db;
pub mod denylist_default;
pub mod frecency;
pub mod logging;
pub mod output;
pub mod path;
pub mod query;
pub mod time;
pub mod types;

use std::process::Command;

pub fn is_command_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
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
