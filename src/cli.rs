use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "memy")]
#[command(version = option_env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Enable verbose (info) logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable debug (very detailed) logging
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
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

#[derive(Args)]
pub struct NoteArgs {
    /// One or more paths to note
    #[arg(value_name = "PATHS")]
    pub paths: Vec<String>,
}

#[derive(Args)]
pub struct ListArgs {
    /// Show only files in the list
    #[arg(short, long)]
    pub files_only: bool,

    /// Show only directories in the list
    #[arg(short, long)]
    pub directories_only: bool,
}
