use core::error::Error;
use std::path::Path;
use tracing::instrument;

use crate::utils::cli::OpenArgs;
use crate::utils::path::expand_tilde_in_path;

#[instrument(level = "trace")]
pub fn command(args: &OpenArgs) -> Result<(), Box<dyn Error>> {
    let expanded = expand_tilde_in_path(Path::new(&args.path));
    let path = expanded.as_ref();

    if !path.exists() {
        return Err(format!("'{}' does not exist", path.display()).into());
    }

    if path.is_dir() {
        return Err(format!("'{}' is a directory, cannot open", path.display()).into());
    }

    open::that_detached(path).map_err(|err| {
        format!(
            "Failed to open '{}' with the system default application: {err}",
            path.display()
        )
        .into()
    })
}
