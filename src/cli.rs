use clap::builder::PossibleValuesParser;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "memy")]
#[command(version = option_env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
pub struct Cli {
    /// Enable verbose logging (can be added multiple times to add more verbosity)
    #[arg(short, long, global = true, action = clap::ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,

    /// Override advanced configuration options normally set in memy.toml
    #[arg(short, long, global = true, value_parser = parse_key_val, value_name("OPTION=VALUE"), number_of_values = 1)]
    pub config: Vec<(String, String)>,

    #[command(subcommand)]
    pub command: Commands,
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "The wrap is required by the clap framework"
)]
#[allow(
    clippy::print_stderr,
    reason = "This function is evaluated before logging framework is configured"
)]
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').unwrap_or_else(|| {
        eprintln!("Invalid key=value pair: {s}");
        std::process::exit(1);
    });
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
    /// Note usage of one or more paths
    Note(NoteArgs),
    /// List paths by frecency score
    List(ListArgs),
    /// Generate a default memy.toml config file on stdout
    GenerateConfig {},
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for (e.g. bash, zsh)
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,
    },
    /// Show contents of a memy hook
    Hook {
        #[arg(value_enum)]
        hook_name: Option<String>,
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
    #[arg(short, long)]
    pub files_only: bool,

    /// Show only directories in the list
    #[arg(short, long)]
    pub directories_only: bool,

    /// Output format
    #[arg(long, default_value = "plain", value_name = "FORMAT", value_parser = PossibleValuesParser::new(["plain", "csv", "json"]))]
    pub format: String,

    /// Output colorization
    #[arg(long, default_value = "automatic", value_name = "WHEN", alias="colour", value_parser = PossibleValuesParser::new(["always", "automatic", "never"]))]
    pub color: String,
}
