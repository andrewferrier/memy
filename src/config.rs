use config::{Config, File, FileFormat};
use dirs_next::config_dir;
use glob::Pattern;
use std::env;
use std::path::PathBuf;

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
    env::var("MEMY_CONFIG_DIR").map_or_else(
        |_| {
            config_dir()
                .map(|dir| dir.join("memy/memy.toml"))
                .expect("Cannot determine config directory")
        },
        |env_path| PathBuf::from(env_path).join("memy.toml"),
    )
}

pub fn load_config() -> MemyConfig {
    let config_path = get_config_file_path();
    if config_path.exists() {
        let settings = Config::builder()
            .add_source(File::from(config_path).format(FileFormat::Toml))
            .build();

        if let Ok(settings) = settings {
            if let Ok(cfg) = settings.try_deserialize::<MemyConfig>() {
                return cfg;
            }
        }
    }
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
