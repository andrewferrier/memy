use core::error::Error;
use std::io::{Write as _, stdout};
use std::path::PathBuf;
use tracing::instrument;

use crate::cli::ZArgs;
use crate::db;
use crate::query;
use crate::utils;

/// Normalizes a `PathBuf` by stripping trailing slashes (by re-collecting components).
fn normalize_path(path: &std::path::Path) -> PathBuf {
    path.components().collect()
}

/// Returns true if `path` matches all `keywords` using the zoxide matching algorithm:
/// * All terms must be present within the path, in order.
/// * The last component of the last keyword must be contained in the last component of the path.
#[must_use]
pub fn matches_zoxide(path: &str, keywords: &[String]) -> bool {
    if keywords.is_empty() {
        return true;
    }

    let path_lower = path.to_lowercase();
    let mut search_start = 0;

    for keyword in keywords {
        let kw_lower = keyword.to_lowercase();
        if let Some(found_offset) = path_lower[search_start..].find(&kw_lower) {
            search_start += found_offset + kw_lower.len();
        } else {
            return false;
        }
    }

    let last_keyword = keywords.last().expect("keywords is non-empty");
    let last_kw_lower = last_keyword.to_lowercase();
    let kw_last_component = last_kw_lower
        .split('/')
        .next_back()
        .unwrap_or(&last_kw_lower);
    let path_last_component = path_lower.split('/').next_back().unwrap_or(&path_lower);

    path_last_component.contains(kw_last_component)
}

/// Returns `Some(absolute_path)` if the resolved path is an existing directory, `None` otherwise.
fn resolve_existing_dir(arg: &str) -> Option<PathBuf> {
    let expanded = utils::expand_tilde_in_path(arg);
    let resolved = if expanded.is_absolute() {
        expanded.into_owned()
    } else {
        let cwd = std::env::current_dir().ok()?;
        cwd.join(&expanded)
    };

    if resolved.is_dir() {
        Some(normalize_path(&resolved))
    } else {
        None
    }
}

#[instrument(level = "trace")]
fn db_search(args: &ZArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open()?;
    let query::SortedMatches { matches, .. } =
        query::build_sorted_matches(&db_connection, |row, meta| {
            if meta.is_dir() && matches_zoxide(&row.path, &args.keywords) {
                query::FilterResult::Include
            } else {
                query::FilterResult::Exclude
            }
        })?;
    db::close(db_connection)?;

    if matches.is_empty() {
        return Err("no match found".into());
    }

    if args.interactive {
        let output: String = matches.iter().fold(String::new(), |mut acc, m| {
            use core::fmt::Write as _;
            let _ = writeln!(acc, "{}", utils::format_path_colored(&m.path, true));
            acc
        });

        let filter_cmd = utils::get_output_filter_command(None).ok_or(
            "No output filter command found. Set MEMY_OUTPUT_FILTER environment variable, \
             memy_output_filter in config, or install fzf.",
        )?;

        let selected = utils::run_output_filter(&output, &filter_cmd)?;
        let mut stdout_handle = stdout().lock();
        stdout_handle.write_all(selected.as_bytes())?;
    } else {
        let best = matches.into_iter().next_back().expect("matches non-empty");
        let mut stdout_handle = stdout().lock();
        writeln!(stdout_handle, "{}", best.path)?;
    }

    Ok(())
}

#[instrument(level = "trace")]
pub fn command(args: &ZArgs) -> Result<(), Box<dyn Error>> {
    if args.interactive {
        return db_search(args);
    }

    if args.keywords.is_empty() {
        let home = std::env::home_dir().ok_or("Cannot determine home directory")?;
        let normalized_home = normalize_path(&home);
        let mut stdout_handle = stdout().lock();
        writeln!(stdout_handle, "{}", normalized_home.to_string_lossy())?;
        return Ok(());
    }

    if args.keywords.len() == 1 && args.keywords[0] == "-" {
        return Err("z -: cannot determine previous directory from within memy; use 'cd -' directly in your shell".into());
    }

    if args.keywords.len() == 1
        && let Some(resolved) = resolve_existing_dir(&args.keywords[0])
    {
        let mut stdout_handle = stdout().lock();
        writeln!(stdout_handle, "{}", resolved.to_string_lossy())?;
        return Ok(());
    }

    db_search(args)
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_zoxide_empty_keywords() {
        assert!(matches_zoxide("/foo/bar", &[]));
    }

    #[test]
    fn test_matches_zoxide_basic() {
        assert!(matches_zoxide("/foo/bar", &["bar".to_owned()]));
    }

    #[test]
    fn test_matches_zoxide_last_component_rule() {
        // "bar" must appear in last component of path
        assert!(!matches_zoxide("/bar/foo", &["bar".to_owned()]));
    }

    #[test]
    fn test_matches_zoxide_case_insensitive() {
        assert!(matches_zoxide("/FOO/BAR", &["bar".to_owned()]));
        assert!(matches_zoxide("/foo/bar", &["BAR".to_owned()]));
    }

    #[test]
    fn test_matches_zoxide_multiple_keywords_ordered() {
        assert!(matches_zoxide(
            "/foo/bar",
            &["fo".to_owned(), "ba".to_owned()]
        ));
        // reversed order should not match
        assert!(!matches_zoxide(
            "/foo/bar",
            &["ba".to_owned(), "fo".to_owned()]
        ));
    }

    #[test]
    fn test_matches_zoxide_slash_in_keyword() {
        // z foo/bar matches /foo/bar but not /foo/bar/baz
        assert!(matches_zoxide("/foo/bar", &["foo/bar".to_owned()]));
        assert!(!matches_zoxide("/foo/bar/baz", &["foo/bar".to_owned()]));
    }

    #[test]
    fn test_matches_zoxide_slash_separated_keywords() {
        // z fo / ba matches /foo/bar but not /foobar
        assert!(matches_zoxide(
            "/foo/bar",
            &["fo".to_owned(), "/".to_owned(), "ba".to_owned()]
        ));
        assert!(!matches_zoxide(
            "/foobar",
            &["fo".to_owned(), "/".to_owned(), "ba".to_owned()]
        ));
    }
}
