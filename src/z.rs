use core::error::Error;
use std::io::{Write as _, stdout};
use tracing::debug;
use tracing::instrument;

use crate::utils;
use crate::utils::cli::ZArgs;
use crate::utils::db;
use crate::utils::path;
use crate::utils::query;

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

#[instrument(level = "trace")]
fn db_search(args: &ZArgs) -> Result<(), Box<dyn Error>> {
    let db_connection = db::open()?;
    let matches = query::build_sorted_matches(&db_connection, |row, meta| {
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
        let output =
            utils::output::format_paths_colored(matches.iter().map(|m| (m.path.as_str(), true)));
        let selected = utils::output::pipe_through_filter(&output, None)?;
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
    if !args.interactive {
        if args.keywords.is_empty() {
            let home = std::env::home_dir().ok_or("Cannot determine home directory")?;
            let normalized_home = path::normalize_path(&home);
            let mut stdout_handle = stdout().lock();
            writeln!(stdout_handle, "{}", normalized_home.to_string_lossy())?;
            return Ok(());
        }

        if args.keywords.len() == 1 && args.keywords[0] == "-" {
            return Err("z -: cannot determine previous directory from within memy; use 'cd -' directly in your shell".into());
        }

        if args.keywords.len() == 1
            && let Some(resolved) = path::resolve_existing_dir(&args.keywords[0])
        {
            debug!(
                "Returning directory as specified on command line: {}",
                resolved.to_string_lossy()
            );
            let mut stdout_handle = stdout().lock();
            writeln!(stdout_handle, "{}", resolved.to_string_lossy())?;
            return Ok(());
        }
    }

    db_search(args)
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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

    proptest! {
        #[test]
        fn prop_keyword_in_last_component_matches(
            prefix in "[a-z]{1,8}",
            keyword in "[a-z]{2,8}",
            suffix in "[a-z]{0,5}",
        ) {
            // Construct a path where the keyword is embedded in the last component.
            let path = format!("/{prefix}/{keyword}{suffix}");
            prop_assert!(matches_zoxide(&path, core::slice::from_ref(&keyword)),
                "path={path} should match keyword={keyword}");
        }

        #[test]
        fn prop_keyword_only_in_non_last_component_doesnt_match(
            keyword in "[a-z]{4,8}",
            last_component in "[a-z]{4,8}",
        ) {
            // The last component must not contain the keyword.
            prop_assume!(!last_component.contains(keyword.as_str()));
            let path = format!("/{keyword}/{last_component}");
            prop_assert!(!matches_zoxide(&path, core::slice::from_ref(&keyword)),
                "keyword={keyword} should not match when absent from last component of path={path}");
        }
    }
}
