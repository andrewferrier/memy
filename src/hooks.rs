use crate::hooks_generated;
use core::error::Error;
use std::io::{self, Write as _};
use tracing::instrument;

fn get_hook_content(name: &str) -> Option<&'static str> {
    hooks_generated::HOOKS.get(name).copied()
}

#[instrument(level = "trace")]
pub fn command(
    hook_name: Option<String>,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    let mut stdout_handle = io::stdout().lock();

    let result = (|| -> io::Result<()> {
        if let Some(actual_hook_name) = hook_name {
            if let Some(content) = get_hook_content(&actual_hook_name) {
                write!(stdout_handle, "{content}")?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Hook not found: {actual_hook_name}"),
                ));
            }
        } else {
            writeln!(stdout_handle, "Available hooks:")?;
            for (k, _) in hooks_generated::HOOKS.iter() {
                writeln!(stdout_handle, "{k}")?;
            }
        }
        Ok(())
    })();

    match result {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e.into()),
        Ok(()) => Ok(()),
    }
}
