use core::error::Error;
use rust_embed::Embed;
use std::io::{self, Write as _};
use tracing::instrument;

#[derive(Embed)]
#[folder = "hooks/"]
struct Hooks;

#[instrument(level = "trace")]
pub fn command(
    hook_name: Option<String>,
) -> core::result::Result<(), std::boxed::Box<dyn Error + 'static>> {
    let mut stdout_handle = io::stdout().lock();

    let result = (|| -> io::Result<()> {
        if let Some(actual_hook_name) = hook_name {
            if let Some(content) = Hooks::get(&actual_hook_name) {
                stdout_handle.write_all(&content.data)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Hook not found: {actual_hook_name}"),
                ));
            }
        } else {
            writeln!(stdout_handle, "Available hooks:")?;
            for name in Hooks::iter() {
                writeln!(stdout_handle, "{name}")?;
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
