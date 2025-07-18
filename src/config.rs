use config::{Config, File, FileFormat};
use glob::Pattern;
use log::{debug, error, info};
use std::env;
use std::path::PathBuf;
use xdg::BaseDirectories;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MemyConfig {
    pub denylist_silent: Option<Vec<String>>,
    pub normalize_symlinks_on_note: Option<bool>,
}

impl Default for MemyConfig {
    fn default() -> Self {
        Self {
            denylist_silent: Some(vec![]),
            normalize_symlinks_on_note: Some(true),
        }
    }
}

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

pub fn generate_config() {
    let config_path = get_config_file_path();

    if config_path.exists() {
        error!("Config file already exists at {}", config_path.display());
        std::process::exit(1);
    }

    let config = MemyConfig::default();
    let toml = toml::to_string_pretty(&config).expect("Failed to serialize config");
    std::fs::write(&config_path, toml).expect("Failed to write config file");
    info!("Config file created at {}", config_path.display());
}

pub fn get_denylist_patterns() -> Vec<Pattern> {
    let config = load_config();
    config
        .denylist_silent
        .unwrap_or_default()
        .into_iter()
        .filter_map(|pat| Pattern::new(&pat).ok())
        .collect()
}

pub fn get_normalize_symlinks_on_note() -> bool {
    load_config().normalize_symlinks_on_note.unwrap_or(true)
}
