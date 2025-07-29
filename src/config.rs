use config::{Config, File, FileFormat};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{debug, error, info};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use xdg::BaseDirectories;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeniedFilesOnList {
    SkipSilently,
    Warn,
    Delete,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MemyConfig {
    pub denylist: Option<Vec<String>>,
    pub normalize_symlinks_on_note: Option<bool>,
    pub missing_files_warn_on_note: Option<bool>,
    pub denied_files_warn_on_note: Option<bool>,
    pub denied_files_on_list: Option<DeniedFilesOnList>,
}

impl Default for MemyConfig {
    fn default() -> Self {
        Self {
            denylist: Some(vec![]),
            normalize_symlinks_on_note: Some(true),
            missing_files_warn_on_note: Some(true),
            denied_files_warn_on_note: Some(true),
            denied_files_on_list: Some(DeniedFilesOnList::Delete),
        }
    }
}

const TEMPLATE_CONFIG: &str = r#"# This is a configuration file for memy - the values below are the defaults.
# **************************************

# List of paths that won't be saved to the memy database even if they are noted - these follow the same syntax as gitignore rules: https://git-scm.com/docs/gitignore
# Example: denylist = ['*.log', '*.out']
denylist = []

# When noting symlinks using memy, should the symlink be saved, or the file the symlink points at?
normalize_symlinks_on_note = true

# When noting files that aren't there, should a warning be emitted?
missing_files_warn_on_note = true

# When noting files that are in the denylist, should a warning be emitted?
denied_files_warn_on_note = true

# When listing files that are in the denylist (they've been added to the denylist after being noted),
# what should happen? Valid values are "skip-silently", "warn", "delete"
denied_files_on_list = "delete"
"#;

fn get_config_file_path() -> PathBuf {
    if let Ok(dir) = env::var("MEMY_CONFIG_DIR") {
        let mut path = PathBuf::from(dir);
        path.push("memy.toml");
        return path;
    }

    let xdg_dirs = BaseDirectories::with_prefix("memy");
    xdg_dirs
        .place_config_file("memy.toml")
        .expect("Cannot determine config file path")
}

fn load_config() -> MemyConfig {
    let config_path = get_config_file_path();
    debug!("Config path: {}", config_path.display());
    if config_path.exists() {
        debug!("Config file exists. Loading config...");
        let settings = Config::builder()
            .add_source(File::from(config_path).format(FileFormat::Toml))
            .build();

        if let Ok(settings) = settings {
            if let Ok(cfg) = settings.try_deserialize::<MemyConfig>() {
                return cfg;
            }
        }
    }
    debug!("Defaulting config");
    MemyConfig::default()
}

static CACHED_CONFIG: LazyLock<MemyConfig> = LazyLock::new(load_config);

pub fn generate_config(filename: Option<&str>) {
    let (config_path, check_exists) = filename.map_or_else(
        || (get_config_file_path(), true),
        |fname| {
            let path = std::path::Path::new(fname);
            let has_parent = path.parent().is_some_and(|p| p != std::path::Path::new(""));
            let final_path = if has_parent {
                path.to_path_buf()
            } else if let Ok(dir) = env::var("MEMY_CONFIG_DIR") {
                let mut p = PathBuf::from(dir);
                p.push(fname);
                p
            } else {
                let mut p = env::current_dir().expect("Failed to get current directory");
                p.push(fname);
                p
            };
            (final_path, false)
        },
    );

    if check_exists && config_path.exists() {
        error!("Config file already exists at {}", config_path.display());
        std::process::exit(1);
    }

    fs::write(&config_path, TEMPLATE_CONFIG).expect("Failed to write config file");
    info!("Config file created at {}", config_path.display());
}

pub fn get_denylist_matcher() -> Gitignore {
    let config = &*CACHED_CONFIG;
    let mut builder = GitignoreBuilder::new("");
    for pat in config.denylist.clone().unwrap_or_default() {
        builder.add_line(None, &pat).ok();
    }
    builder.build().expect("Failed to build denylist matcher")
}

pub fn get_normalize_symlinks_on_note() -> bool {
    CACHED_CONFIG.normalize_symlinks_on_note.unwrap_or(true)
}

pub fn get_missing_files_warn_on_note() -> bool {
    CACHED_CONFIG.missing_files_warn_on_note.unwrap_or(true)
}

pub fn get_denied_files_warn_on_note() -> bool {
    CACHED_CONFIG.denied_files_warn_on_note.unwrap_or(true)
}

pub fn get_denied_files_on_list() -> DeniedFilesOnList {
    CACHED_CONFIG
        .denied_files_on_list
        .clone()
        .unwrap_or(DeniedFilesOnList::Delete)
}
