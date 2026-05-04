mod hooks;
mod import;
mod list;
mod note;
mod stats;
mod utils;
mod z;

use clap::CommandFactory as _;
use core::error::Error;
use log::debug;
use std::io::stdout;
use tracing::instrument;
use utils::cli::{Cli, Commands};

#[instrument(level = "trace")]
fn completions(shell: Option<clap_complete::Shell>) -> Result<(), Box<dyn Error>> {
    let actual_shell = shell
        .or_else(clap_complete::Shell::from_env)
        .ok_or("Could not determine shell. Specify one explicitly.")?;
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_owned();
    clap_complete::generate(actual_shell, &mut cmd, bin_name, &mut stdout());
    Ok(())
}

fn handle_cli_command(
    command: Commands,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    match command {
        Commands::Note(note_args) => Ok(note::command(note_args)?),
        Commands::List(list_args) => Ok(list::command(&list_args)?),
        Commands::GenerateConfig {} => Ok(utils::config::output_template_config()?),
        Commands::Completions { shell } => Ok(completions(shell)?),
        Commands::Hook { hook_name } => Ok(hooks::command(hook_name)?),
        Commands::Stats(stats_args) => Ok(stats::command(&stats_args)?),
        Commands::Z(z_args) => Ok(z::command(&z_args)?),
    }
}

fn configure_color(color: &str) -> Result<Option<bool>, Box<dyn Error>> {
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
        _ => Err(format!("Invalid value for color: {color}").into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = utils::cli::parse();

    let color_option = configure_color(&cli.color)?;

    utils::logging::configure_logging_and_tracing(cli.verbose, color_option);
    utils::config::load_config(cli.config.clone());

    debug!("Memy version {}", env!("GIT_VERSION"));
    debug!("CLI params parsed: {cli:?}");

    handle_cli_command(cli.command)?;

    Ok(())
}
