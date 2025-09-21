mod cli;
mod config;
mod db;
mod denylist_default;
mod hooks;
mod hooks_generated;
mod import;
mod list;
mod logging;
mod note;
mod types;
mod utils;

use clap::CommandFactory as _;
use clap::Parser as _;
use cli::{Cli, Commands};
use core::error::Error;
use log::{debug, error, warn};
use std::io::{self, stdout, Write as _};
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

#[instrument(level = "trace")]
fn hook_show(
    hook_name: Option<String>,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    let mut stdout_handle = io::stdout().lock();

    if let Some(actual_hook_name) = hook_name {
        if let Some(content) = hooks::get_hook_content(&actual_hook_name) {
            write!(stdout_handle, "{content}")?;
        } else {
            return Err(format!("Hook not found: {actual_hook_name}").into());
        }
    } else {
        writeln!(stdout_handle, "Available hooks:")?;
        for hook in hooks::get_hook_list() {
            writeln!(stdout_handle, "{hook}")?;
        }
    }

    Ok(())
}

fn handle_cli_command(
    command: Commands,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    match command {
        Commands::Note(note_args) => Ok(note::note_paths(note_args)?),
        Commands::List(list_args) => Ok(list::list_paths(&list_args)?),
        Commands::GenerateConfig {} => Ok(config::output_template_config()?),
        Commands::Completions { shell } => {
            completions(shell);
            Ok(())
        }
        Commands::Hook { hook_name } => Ok(hook_show(hook_name)?),
    }
}

fn main() {
    let cli = Cli::parse();

    config::set_config_overrides(cli.config.clone());
    logging::configure_logging_and_tracing(cli.verbose);

    debug!("Memy version {}", env!("GIT_VERSION"));
    debug!("CLI params parsed: {cli:?}");

    match handle_cli_command(cli.command) {
        Ok(()) => {}
        Err(err) => {
            error!("{err}");
            std::process::exit(1);
        }
    }
}
