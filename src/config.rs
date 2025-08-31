use config::{Config, File, FileFormat, Value};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use log::{debug, error};
use serde::Deserialize as _;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::OnceLock;
use toml::Value as TomlValue;
use tracing::instrument;
use xdg::BaseDirectories;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeniedFilesOnList {
    SkipSilently,
    Warn,
    Delete,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemyConfig {
    pub denylist: Option<Vec<String>>,
    pub normalize_symlinks_on_note: Option<bool>,
    pub missing_files_warn_on_note: Option<bool>,
    pub denied_files_warn_on_note: Option<bool>,
    pub denied_files_on_list: Option<DeniedFilesOnList>,
    #[serde(default, deserialize_with = "validate_recency_bias")]
    pub recency_bias: Option<f64>,
}

fn validate_recency_bias<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<f64> = Option::deserialize(deserializer)?;
    if let Some(v) = value {
        if !(0.0..=1.0).contains(&v) {
            error!("recency_bias configuration option must be between 0 and 1");
            std::process::exit(1);
        }
    }
    Ok(value)
}

static CACHED_CONFIG: LazyLock<MemyConfig> = LazyLock::new(load_config);
static CONFIG_OVERRIDES: OnceLock<Vec<(String, String)>> = OnceLock::new();

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

# When listing, how much should *recency of file noting* dominate over *frequency of noting* in the sort? 0.0 means pure frequency; 1.0 means pure recency.
recency_bias = 0.5
"#;

#[instrument(level = "trace")]
fn get_config_file_path() -> PathBuf {
    if let Ok(dir) = env::var("MEMY_CONFIG_DIR") {
        let path = PathBuf::from(dir).join("memy.toml");
        return path;
    }

    let xdg_dirs = BaseDirectories::with_prefix("memy");
    xdg_dirs
        .get_config_file("memy.toml")
        .expect("Couldn't calculate XDG path for config file")
}

fn toml_to_config_value(toml_val: &TomlValue) -> Value {
    match toml_val {
        TomlValue::String(s) => Value::from(s.clone()),
        TomlValue::Array(arr) => {
            let vec_vals = arr.iter().map(toml_to_config_value).collect::<Vec<_>>();
            Value::from(vec_vals)
        }
        _ => {
            unimplemented!("TOML parser can't support any other data type here")
        }
    }
}

fn parse_toml_value(s: &str) -> Result<Value, Box<dyn core::error::Error>> {
    let toml_snippet = format!("value = {s}");
    let parsed: TomlValue = toml::from_str(&toml_snippet)?;
    let inner = parsed
        .get("value")
        .ok_or("Missing 'value' in parsed toml")?;
    let config_value = toml_to_config_value(inner);

    Ok(config_value)
}

fn test_config_file_issues(config_path: &PathBuf) {
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(err) => {
            error!("Failed to read configuration file: {err}");
            std::process::exit(1);
        }
    };

    match toml::from_str::<MemyConfig>(&config_content) {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to parse configuration file: {err}");
            std::process::exit(1);
        }
    }
}

#[instrument(level = "trace")]
pub fn load_config() -> MemyConfig {
    let default_config = Config::builder()
        .add_source(File::from_str(TEMPLATE_CONFIG, FileFormat::Toml))
        .build()
        .expect("Defaults didn't load");

    let mut builder = Config::builder().add_source(default_config.clone());

    let config_path: PathBuf = get_config_file_path();
    debug!("Config file path resolved to {}", config_path.display());

    if config_path.exists() {
        test_config_file_issues(&config_path);
        debug!("Config file looks OK");
        builder = builder.add_source(File::from(config_path).format(FileFormat::Toml));
    }

    for (key, value_str) in CONFIG_OVERRIDES.get().expect("Overrides not loaded") {
        if key == "denylist" {
            let value = parse_toml_value(value_str)
                .expect("Failed to parse TOML value for denylist override");
            builder = builder
                .set_override(key, value)
                .expect("Failed to set override for denylist");
        } else {
            builder = builder
                .set_override(key, value_str.as_str())
                .expect("Failed to set override");
        }
    }

    let config = builder
        .build()
        .and_then(Config::try_deserialize::<MemyConfig>)
        .unwrap_or_else(|e| {
            log::warn!(
                "Failed to build or deserialize final config, falling back to defaults: {e}"
            );
            default_config
                .try_deserialize()
                .expect("Default config invalid")
        });

    debug!("Config loaded: {config:?}");

    config
}

pub fn set_config_overrides(overrides: Vec<(String, String)>) {
    CONFIG_OVERRIDES
        .set(overrides)
        .expect("Overrides could not be set");
}

#[instrument(level = "trace")]
pub fn generate_config(filename: Option<&str>) -> Result<(), Box<dyn core::error::Error>> {
    let filename_pathbuf: Option<PathBuf> = filename.map(PathBuf::from);
    let final_filename = filename_pathbuf.unwrap_or_else(get_config_file_path);

    if final_filename.exists() {
        return Err(format!("Config file already exists at {}", final_filename.display()).into());
    }

    fs::write(&final_filename, TEMPLATE_CONFIG)?;
    println!("Config file created at {}", final_filename.display());
    Ok(())
}

pub fn get_denylist_matcher() -> Gitignore {
    let config = &*CACHED_CONFIG;
    let mut builder = GitignoreBuilder::new("");
    for pat in config.denylist.clone().unwrap_or_default() {
        builder
            .add_line(None, &pat)
            .unwrap_or_else(|_| panic!("Pattern {pat} not valid."));
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

pub fn get_recency_bias() -> f64 {
    CACHED_CONFIG.recency_bias.unwrap_or(0.5)
}
