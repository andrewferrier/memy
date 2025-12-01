mod cli;
mod config;
mod db;
mod denylist_default;
mod frecency;
mod hooks;
mod hooks_generated;
mod import;
mod list;
mod logging;
mod note;
mod stats;
mod types;
mod utils;

use clap::CommandFactory as _;
use clap::Parser as _;
use cli::{Cli, Commands};
use core::error::Error;
use log::{debug, error, warn};
use std::io::stdout;
use tracing::instrument;

#[instrument(level = "trace")]
fn completions(shell: Option<clap_complete::Shell>) {
    let actual_shell = shell
        .or_else(utils::detect_shell)
        .expect("Could not determine shell. Specify one explicitly.");
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_owned();
    clap_complete::generate(actual_shell, &mut cmd, bin_name, &mut stdout());
}

fn handle_cli_command(
    command: Commands,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    match command {
        Commands::Note(note_args) => Ok(note::command(note_args)?),
        Commands::List(list_args) => Ok(list::command(&list_args)?),
        Commands::GenerateConfig {} => Ok(config::output_template_config()?),
        Commands::Completions { shell } => {
            completions(shell);
            Ok(())
        }
        Commands::Hook { hook_name } => Ok(hooks::command(hook_name)?),
        Commands::Stats(stats_args) => Ok(stats::command(&stats_args)?),
    }
}

fn configure_color(color: &str) -> Result<Option<bool>, String> {
    match color {
        "always" => {
            colored::control::set_override(true);
            Ok(Some(true))
        }
        "never" => {
            colored::control::set_override(false);
            Ok(Some(false))
        }
        "automatic" => Ok(None),
        _ => Err(format!("Invalid value for color: {color}")),
    }
}

fn main() {
    let cli = Cli::parse();

    let color_option = match configure_color(&cli.color) {
        Ok(option) => option,
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    };

    logging::configure_logging_and_tracing(cli.verbose, color_option);
    config::load_config(cli.config.clone());

    debug!("Memy version {}", env!("GIT_VERSION"));
    debug!("CLI params parsed: {cli:?}");

    if let Err(err) = handle_cli_command(cli.command) {
        error!("{err}");
        std::process::exit(1);
    }
}
