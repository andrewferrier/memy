use clap::builder::PossibleValuesParser;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "memy",
    version = option_env!("GIT_VERSION"),
    author = "Andrew Ferrier",
    about = "Track and recall frequently and recently used files or directories.",
    subcommand_required = true,
    override_usage = r#"
  memy note <FILES...> - note some files
  memy list            - list noted files in frecency order"#)]
pub struct Cli {
    /// Enable verbose logging (add multiple times for more verbosity)
    #[arg(display_order = 100, short, long, global = true, action = clap::ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,

    /// Override advanced configuration options normally set in memy.toml
    #[arg(display_order = 101, short, long, global = true, value_parser = parse_key_val, value_name("OPTION=VALUE"), number_of_values = 1)]
    pub config: Vec<(String, String)>,

    /// Output colorization
    #[arg(long, default_value = "automatic", value_name = "WHEN", alias="colour", value_parser = PossibleValuesParser::new(["always", "automatic", "never"]))]
    pub color: String,

    #[command(subcommand)]
    pub command: Commands,
}

fn parse_key_val(key_value: &str) -> Result<(String, String), String> {
    let (key_str, value_str) = key_value
        .split_once('=')
        .ok_or_else(|| format!("Invalid key=value pair: missing '=' in '{key_value}'"))?;

    let key = key_str.trim().to_owned();
    let mut value = value_str.trim().to_owned();

    if let Some(stripped) = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        value = stripped.to_owned();
    } else if let Some(stripped) = value.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        value = stripped.to_owned();
    }

    Ok((key, value))
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Note usage of (add to database) one or more paths
    Note(NoteArgs),
    /// List paths by frecency score
    List(ListArgs),
    /// Show statistics about noted paths
    Stats(StatsArgs),
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

    /// Only show paths noted after a specific time. Accepts duration strings (e.g., '4d', '3h', '4d3h')
    /// or ISO-8601 date/time strings (e.g., '2025-01-01', '2025-01-01T12:00:00')
    #[arg(long, value_name = "TIME")]
    pub newer_than: Option<String>,
}

#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Output format
    #[arg(long, default_value = "plain", value_name = "FORMAT", value_parser = PossibleValuesParser::new(["plain", "json"]))]
    pub format: String,
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_parse_key_val_no_equal_sign() {
        assert!(parse_key_val("invalidkeyvalue").is_err());
        assert_eq!(
            parse_key_val("invalidkeyvalue").unwrap_err(),
            "Invalid key=value pair: missing '=' in 'invalidkeyvalue'"
        );
    }

    #[test]
    fn test_parse_key_val_padded() {
        assert_eq!(
            parse_key_val(" a = b ").unwrap(),
            ("a".to_owned(), "b".to_owned())
        );
        assert_eq!(
            parse_key_val(" a = \"b \" ").unwrap(),
            ("a".to_owned(), "b ".to_owned())
        );
        assert_eq!(
            parse_key_val(" a = ' b' ").unwrap(),
            ("a".to_owned(), " b".to_owned())
        );
    }

    proptest! {
        #[test]
        fn proptest_parse_key_val(
            key_first in prop::sample::select(
                ('a'..='z').chain('A'..='Z').chain(core::iter::once('_')).collect::<Vec<_>>()
            ),
            key_rest in prop::collection::vec(
                prop::sample::select(
                    ('a'..='z')
                        .chain('A'..='Z')
                        .chain('0'..='9')
                        .chain(vec!['_','-'].into_iter())
                        .collect::<Vec<_>>()
                ),
                0..=10
            ),
            value_raw in prop::collection::vec(
                prop::sample::select(
                    ('a'..='z')
                        .chain('A'..='Z')
                        .chain('0'..='9')
                        .chain(vec![' ','_','-','.','='].into_iter())
                        .collect::<Vec<_>>()
                ),
                0..=20
            ),
            quote_type in prop_oneof![Just("\""), Just("'"), Just("none")],
            whitespace in prop::collection::vec(
                prop::sample::select(vec![' ', '\t']),
                0..=5
            )
        )
        {
            let value_raw_string: String = value_raw.into_iter().collect();
            let whitespace_string: String = whitespace.into_iter().collect();

            prop_assume!(
                !(value_raw_string.chars().any(char::is_whitespace) && quote_type == "none")
            );

            let input_value_str = match quote_type{
                "\"" => format!("\"{value_raw_string}\""),
                "'" => format!("'{value_raw_string}'"),
                "none" => value_raw_string.clone(),
                _ => String::new(),
            };

            let key: String = core::iter::once(key_first).chain(key_rest.into_iter()).collect();

            let input_str = format!("{whitespace_string}{key}{whitespace_string}={whitespace_string}{input_value_str}{whitespace_string}");

            let (parsed_key, parsed_value) = parse_key_val(&input_str).unwrap_or_else(|_| panic!("Parsing failed for input: \"{input_str}\""));

            assert_eq!(parsed_key, key, "Key mismatch for input: \"{input_str}\"");
            assert_eq!(parsed_value, value_raw_string, "Value mismatch for input: \"{input_str}\"");
        }
    }
}
