use clap::builder::PossibleValuesParser;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "memy")]
#[command(version = option_env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
#[command(subcommand_required = true)]
#[command(override_usage = r#"
  memy note <FILES...> - note some files
  memy list            - list noted files in frecency order"#)]
pub struct Cli {
    /// Enable verbose logging (add multiple times for more verbosity)
    #[arg(display_order = 100, short, long, global = true, action = clap::ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,

    /// Override advanced configuration options normally set in memy.toml
    #[arg(display_order = 101, short, long, global = true, value_parser = parse_key_val, value_name("OPTION=VALUE"), number_of_values = 1)]
    pub config: Vec<(String, String)>,

    #[command(subcommand)]
    pub command: Commands,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key=value pair: {s}"))?;
    let key = s[..pos].to_string();
    let mut value = s[pos + 1..].to_string();
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value = value[1..value.len() - 1].to_string();
    }
    Ok((key, value))
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Note usage of (add to database) one or more paths
    Note(NoteArgs),
    /// List paths by frecency score
    List(ListArgs),
    /// Show contents of a memy hook
    Hook {
        #[arg(value_enum)]
        hook_name: Option<String>,
    },
    /// Generate a default memy.toml config file on stdout
    GenerateConfig {},
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,
    },
}

#[derive(Args, Debug)]
pub struct NoteArgs {
    /// One or more paths to note
    #[arg(value_name = "PATHS")]
    pub paths: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show only files in the list
    #[arg(short, long, conflicts_with = "directories_only")]
    pub files_only: bool,

    /// Show only directories in the list
    #[arg(short, long, conflicts_with = "files_only")]
    pub directories_only: bool,

    /// Output format
    #[arg(long, default_value = "plain", value_name = "FORMAT", value_parser = PossibleValuesParser::new(["plain", "csv", "json"]))]
    pub format: String,

    /// Output colorization
    #[arg(long, default_value = "automatic", value_name = "WHEN", alias="colour", value_parser = PossibleValuesParser::new(["always", "automatic", "never"]))]
    pub color: String,
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_val_simple() {
        assert_eq!(
            parse_key_val("key=value").unwrap(),
            ("key".to_owned(), "value".to_owned())
        );
    }

    #[test]
    fn test_parse_key_val_with_double_quotes() {
        assert_eq!(
            parse_key_val("key=\"value with spaces\"").unwrap(),
            ("key".to_owned(), "value with spaces".to_owned())
        );
    }

    #[test]
    fn test_parse_key_val_with_single_quotes() {
        assert_eq!(
            parse_key_val("key='value with spaces'").unwrap(),
            ("key".to_owned(), "value with spaces".to_owned())
        );
    }

    #[test]
    fn test_parse_key_val_no_value() {
        assert!(parse_key_val("key=").is_ok());
        assert_eq!(
            parse_key_val("key=").unwrap(),
            ("key".to_owned(), String::new())
        );
    }

    #[test]
    fn test_parse_key_val_empty_value_with_quotes() {
        assert_eq!(
            parse_key_val("key=\"\"").unwrap(),
            ("key".to_owned(), String::new())
        );
    }

    #[test]
    fn test_parse_key_val_no_equal_sign() {
        assert!(parse_key_val("invalidkeyvalue").is_err());
        assert_eq!(
            parse_key_val("invalidkeyvalue").unwrap_err(),
            "Invalid key=value pair: invalidkeyvalue"
        );
    }

    #[test]
    fn test_parse_key_val_multiple_equals() {
        assert_eq!(
            parse_key_val("key=value=another").unwrap(),
            ("key".to_owned(), "value=another".to_owned())
        );
    }

    #[test]
    fn test_parse_key_val_with_numbers() {
        assert_eq!(
            parse_key_val("count=123").unwrap(),
            ("count".to_owned(), "123".to_owned())
        );
    }

    #[test]
    fn test_parse_key_val_with_boolean() {
        assert_eq!(
            parse_key_val("enabled=true").unwrap(),
            ("enabled".to_owned(), "true".to_owned())
        );
    }
}
