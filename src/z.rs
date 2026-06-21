use core::error::Error;
use tracing::{instrument, warn};

use crate::utils::cli::{ListArgs, ZArgs};

#[instrument(level = "trace")]
pub fn command(args: &ZArgs) -> Result<(), Box<dyn Error>> {
    warn!("memy z is deprecated; use 'memy list' instead");

    let list_args = ListArgs {
        directories_only: true,
        files_only: false,
        keywords: args.keywords.clone(),
        zoxide_compatible: true,
        output_filter: args.interactive,
        output_filter_command: None,
        head: if args.interactive { None } else { Some(1) },
        format: "plain".to_owned(),
        newer_than: None,
    };
    crate::list::command(&list_args)
}
