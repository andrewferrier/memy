use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "memy")]
#[command(version = option_env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, action = clap::ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,

    /// Override configuration options in config.toml
    #[arg(short, long, value_parser = parse_key_val, value_name("OPTION=VALUE"), number_of_values = 1)]
    pub config: Vec<(String, String)>,

    #[command(subcommand)]
    pub command: Commands,
}

#[allow(clippy::unnecessary_wraps)]
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
    /// Generate a template memy.toml config file
    GenerateConfig {
        /// Optional output filename for the generated config
        #[arg(value_name = "FILENAME")]
        filename: Option<String>,
    },
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

    /// Output format: 'plain', 'csv', or 'json'
    #[arg(long, default_value = "plain", value_name = "FORMAT", value_parser = validate_format)]
    pub format: String,
}

fn validate_format(value: &str) -> Result<String, String> {
    match value {
        "plain" | "csv" | "json" => Ok(value.to_string()),
        _ => Err(String::from(
            "Invalid value for --format. Allowed values are 'plain', 'csv', or 'json'.",
        )),
    }
}
