use std::path::Path;

pub fn detect_shell() -> Option<clap_complete::Shell> {
    std::env::var("SHELL").ok().as_deref().and_then(|path| {
        Path::new(path)
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(str::to_lowercase)
            .and_then(|name| match name.as_str() {
                "bash" => Some(clap_complete::Shell::Bash),
                "zsh" => Some(clap_complete::Shell::Zsh),
                "fish" => Some(clap_complete::Shell::Fish),
                "powershell" => Some(clap_complete::Shell::PowerShell),
                "elvish" => Some(clap_complete::Shell::Elvish),
                _ => None,
            })
    })
}
