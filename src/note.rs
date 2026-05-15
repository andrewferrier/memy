use core::error::Error;
use ignore::gitignore::Gitignore;
use rayon::prelude::*;
use rusqlite::Transaction;
use rusqlite::params;
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::instrument;
use tracing::{info, warn};

use crate::utils;
use crate::utils::cli;
use crate::utils::config;
use crate::utils::db;
use crate::utils::types::UnixTimestamp;

fn normalize_path_if_needed(path: Cow<'_, Path>) -> std::io::Result<Cow<'_, Path>> {
    let normalize = config::get_normalize_symlinks_on_note();

    if normalize {
        Ok(Cow::Owned(fs::canonicalize(&*path)?))
    } else {
        Ok(path)
    }
}

/// Returns `Ok(None)` when the path should be silently skipped, `Ok(Some(path))`
/// when it should be inserted, or `Err` on an unexpected I/O failure.
fn preprocess_path(raw_path: &str, matcher: &Gitignore) -> std::io::Result<Option<PathBuf>> {
    let path = utils::path::expand_tilde_in_path(raw_path);

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }
        return Ok(None);
    }

    let clean_path = normalize_path_if_needed(path)?;

    if let ignore::Match::Ignore(_) = matcher.matched_path_or_any_parents(&clean_path, false) {
        if config::get_denied_files_warn_on_note() {
            warn!("Path {} denied by denylist pattern.", clean_path.display());
        }
        return Ok(None);
    }

    Ok(Some(clean_path.into_owned()))
}

fn insert_path(tx: &Transaction, path: &Path, now: UnixTimestamp) {
    tx.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![path.to_string_lossy(), now],
    )
    .expect("Insert failed");

    info!("Path {} noted", path.display());
}

#[instrument(level = "trace")]
pub fn command(note_args: cli::NoteArgs) -> Result<(), Box<dyn Error>> {
    if note_args.paths.is_empty() {
        return Err("You must specify some paths to note".into());
    }

    let matcher = config::get_denylist_matcher();

    let preprocessed: Vec<Option<PathBuf>> = note_args
        .paths
        .into_par_iter()
        .map(|raw_path| preprocess_path(&raw_path, &matcher))
        .collect::<Result<_, _>>()?;

    let now = utils::get_unix_timestamp();
    let mut db_connection = db::open().expect("Could not open memy database");
    let tx = db_connection
        .transaction()
        .expect("Cannot start DB transaction");

    for clean_path in preprocessed.into_iter().flatten() {
        insert_path(&tx, &clean_path, now);
    }

    tx.commit().expect("Cannot commit transaction");
    db::close(db_connection)?;

    Ok(())
}
