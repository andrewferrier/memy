use core::error::Error;
use log::{info, warn};
use rusqlite::params;
use tracing::instrument;

use super::cli;
use super::config;
use super::db;
use super::utils;

use rusqlite::Transaction;
use std::borrow::Cow;
use std::fs;
use std::path::Path;

fn normalize_path_if_needed(path: &Path) -> std::io::Result<Cow<'_, Path>> {
    let normalize = config::get_normalize_symlinks_on_note();

    if normalize {
        Ok(Cow::Owned(fs::canonicalize(path)?))
    } else {
        Ok(Cow::Borrowed(path))
    }
}

fn note_path(tx: &Transaction, raw_path: &str) -> Result<(), Box<dyn Error + 'static>> {
    let pathbuf = utils::expand_tilde(raw_path);
    let path = pathbuf.as_path();

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }

        return Ok(());
    }

    let clean_path = normalize_path_if_needed(path)?;

    let matcher = config::get_denylist_matcher();
    if let ignore::Match::Ignore(_matched_pat) =
        matcher.matched_path_or_any_parents(&clean_path, false)
    {
        if config::get_denied_files_warn_on_note() {
            warn!("Path {} denied by denylist pattern.", clean_path.display());
        }

        return Ok(());
    }

    let now = utils::get_unix_timestamp();
    tx.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![clean_path.to_string_lossy(), now],
    )
    .expect("Insert failed");

    info!("Path {} noted", clean_path.display());
    Ok(())
}

#[instrument(level = "trace")]
pub fn note_paths(note_args: cli::NoteArgs) -> Result<(), Box<dyn Error>> {
    if note_args.paths.is_empty() {
        return Err("You must specify some paths to note".into());
    }

    let mut db_connection = db::open_db().expect("Could not open memy database");
    let tx = db_connection
        .transaction()
        .expect("Cannot start DB transaction");

    for path in note_args.paths {
        note_path(&tx, &path)?;
    }

    tx.commit().expect("Cannot commit transaction");

    Ok(())
}
