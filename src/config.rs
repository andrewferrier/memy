use config::{Config, File, FileFormat};
use glob::Pattern;
use log::debug;
use std::env;
use std::path::PathBuf;
use xdg::BaseDirectories;

#[derive(Debug, serde::Deserialize)]
pub struct MemyConfig {
    pub denylist_silent: Option<Vec<String>>,
}

impl Default for MemyConfig {
    fn default() -> Self {
        Self {
            denylist_silent: Some(vec![]),
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

pub fn load_config() -> MemyConfig {
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

pub fn get_denylist_patterns() -> Vec<Pattern> {
    let config = load_config();
    config
        .denylist_silent
        .unwrap_or_default()
        .into_iter()
        .filter_map(|pat| Pattern::new(&pat).ok())
        .collect()
}
