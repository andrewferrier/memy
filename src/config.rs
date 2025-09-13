use config::{Config, File, FileFormat, Value};
use core::error::Error;
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

pub type RecencyBias = f64;

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
    pub import_on_first_use: Option<bool>,
    pub denylist: Option<Vec<String>>,
    pub normalize_symlinks_on_note: Option<bool>,
    pub missing_files_warn_on_note: Option<bool>,
    pub denied_files_warn_on_note: Option<bool>,
    pub denied_files_on_list: Option<DeniedFilesOnList>,
    #[serde(default, deserialize_with = "validate_recency_bias")]
    pub recency_bias: Option<RecencyBias>,
    pub missing_files_delete_from_db_after: Option<u64>,
}

fn validate_recency_bias<'de, D>(deserializer: D) -> Result<Option<RecencyBias>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<RecencyBias> = Option::deserialize(deserializer)?;
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

const TEMPLATE_CONFIG: &str = include_str!("../config/template-memy.toml");

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

fn parse_toml_value(s: &str) -> Result<Value, Box<dyn Error>> {
    let toml_snippet = format!("value = {s}");
    let parsed: TomlValue = toml::from_str(&toml_snippet)?;
    let inner = parsed
        .get("value")
        .ok_or("Missing 'value' in parsed toml")?;
    let config_value = toml_to_config_value(inner);

    Ok(config_value)
}

fn test_config_file_issues(config_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(err) => {
            return Err(format!("Failed to read configuration file: {err}").into());
        }
    };

    match toml::from_str::<MemyConfig>(&config_content) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to parse configuration file: {err}").into()),
    }
}

#[instrument(level = "trace")]
pub fn load_config() -> MemyConfig {
    let default_config = Config::builder()
        .add_source(File::from_str(TEMPLATE_CONFIG, FileFormat::Toml))
        .build()
        .expect("Defaults didn't load");

    let mut builder = Config::builder().add_source(default_config);

    let config_path: PathBuf = get_config_file_path();
    debug!("Config file path resolved to {}", config_path.display());

    if config_path.exists() {
        if let Err(err) = test_config_file_issues(&config_path) {
            error!("{err}");
            std::process::exit(1);
        }

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
            error!("Failed to build or deserialize final config, falling back to defaults: {e}");
            std::process::exit(1);
        });

    debug!("Config loaded: {config:?}");

    config
}

pub fn set_config_overrides(overrides: Vec<(String, String)>) {
    CONFIG_OVERRIDES
        .set(overrides)
        .expect("Overrides could not be set");
}

pub fn output_template_config() {
    print!("{TEMPLATE_CONFIG}");
}

pub fn get_import_on_first_use() -> bool {
    CACHED_CONFIG.import_on_first_use.unwrap_or(true)
}

fn build_gitignore(patterns: Vec<String>) -> Gitignore {
    let mut builder = GitignoreBuilder::new("/");
    for pat in patterns {
        builder
            .add_line(None, &pat)
            .unwrap_or_else(|_| panic!("Pattern {pat} not valid."));
    }
    builder.build().expect("Failed to build denylist matcher")
}

pub fn get_denylist_matcher() -> Gitignore {
    let config = &*CACHED_CONFIG;
    build_gitignore(config.denylist.clone().unwrap_or_default())
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

pub fn get_recency_bias() -> RecencyBias {
    CACHED_CONFIG.recency_bias.unwrap_or(0.5)
}

pub fn get_missing_files_delete_from_db_after() -> u64 {
    CACHED_CONFIG
        .missing_files_delete_from_db_after
        .unwrap_or(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(
        clippy::cognitive_complexity,
        reason = "It's OK as these tests are all short"
    )]
    fn gitignore_tests() {
        struct OwnedMatch {
            is_ignore: bool,
            is_whitelist: bool,
            is_none: bool,
        }

        fn wrapper(patterns: &[&str], path: &str, is_dir: bool) -> OwnedMatch {
            let strings: Vec<String> = patterns
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            let gitignore = build_gitignore(strings);
            let matched = gitignore.matched_path_or_any_parents(path, is_dir);

            OwnedMatch {
                is_ignore: matched.is_ignore(),
                is_whitelist: matched.is_whitelist(),
                is_none: matched.is_none(),
            }
        }

        // This is somewhat repeating testing which is presumably done on the Gitignore crate
        // anyway, but it's in the context of the way we build it in build_gitignore() with
        // root='/' and so on. See also ../tests/denylist.rs

        assert!(wrapper(&["foo.txt"], "foo.txt", false).is_ignore);
        assert!(wrapper(&["*.log"], "foo.log", false).is_ignore);
        assert!(wrapper(&["*.log"], "foo.txt", false).is_none);
        assert!(wrapper(&["*.log", "!important.log"], "app.log", false).is_ignore);
        assert!(wrapper(&["*.log", "!important.log"], "important.log", false).is_whitelist);
        assert!(wrapper(&["foo"], "foo/bar.txt", false).is_ignore);
        assert!(wrapper(&["foo/"], "foo/bar.txt", false).is_ignore);
        assert!(wrapper(&["foo/*"], "foo/bar.txt", false).is_ignore);
        assert!(wrapper(&["foo/", "!foo/bar/"], "foo/bar/wibble.txt", false).is_whitelist);
        assert!(wrapper(&["docs/"], "sub/docs/foo.txt", false).is_ignore);
        assert!(wrapper(&["/docs/"], "sub/docs/foo.txt", false).is_none);
        assert!(wrapper(&["**/docs/"], "sub/docs/foo.txt", false).is_ignore);
        assert!(wrapper(&["**/docs/**"], "sub/docs/foo.txt", false).is_ignore);
        assert!(wrapper(&["*.[oa]"], "a.a", false).is_ignore);
        assert!(wrapper(&["*.[oa]"], "a.b", false).is_none);
        assert!(wrapper(&["a/**/z"], "a/z", false).is_ignore);
        assert!(wrapper(&["a/**/z"], "a/b/z", false).is_ignore);
        assert!(wrapper(&["a/**/z"], "a/b/c/z", false).is_ignore);
        assert!(wrapper(&["\\!important.txt"], "!important.txt", false).is_ignore);
        assert!(wrapper(&["\\#notes.txt"], "#notes.txt", false).is_ignore);
        assert!(wrapper(&[""], "foo.txt", false).is_none);
        assert!(wrapper(&["#foo.txt"], "foo.txt", false).is_none);
        assert!(wrapper(&[" foo.txt"], "foo.txt", false).is_none);
        assert!(wrapper(&["foo.txt", "!foo.txt"], "foo.txt", false).is_whitelist);
        assert!(wrapper(&["*.txt", "!*.txt"], "foo.txt", false).is_whitelist);

        assert!(wrapper(&["foo/"], "foo", true).is_ignore);
        assert!(wrapper(&["foo/"], "bar", true).is_none);
        assert!(wrapper(&["bar/"], "foo", true).is_none);
        assert!(wrapper(&["bar"], "foo/bar", true).is_ignore);
        assert!(wrapper(&["bar"], "bar/bar", true).is_ignore);
        assert!(wrapper(&["/bar"], "bar", true).is_ignore);
        assert!(wrapper(&["/bar"], "foo", true).is_none);
        assert!(wrapper(&["/bar"], "/foo", true).is_none);
        assert!(wrapper(&["/bar"], "/bar", true).is_ignore);
    }
}
