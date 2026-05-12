use std::borrow::Cow;
use std::env::home_dir;
use std::path::{Component, Path, PathBuf};

/// Strips trailing slashes (by re-collecting components).
pub fn normalize_path(path: &Path) -> PathBuf {
    path.components().collect()
}

/// Returns `Some(absolute_path)` if the resolved path is an existing directory, `None` otherwise.
pub fn resolve_existing_dir(arg: &str) -> Option<PathBuf> {
    let expanded = expand_tilde_in_path(arg);
    let resolved = if expanded.is_absolute() {
        expanded.into_owned()
    } else {
        let cwd = std::env::current_dir().ok()?;
        cwd.join(&expanded)
    };

    if resolved.is_dir() {
        Some(resolved.components().collect())
    } else {
        None
    }
}

pub fn expand_tilde_in_path<P: AsRef<Path> + ?Sized>(path: &'_ P) -> Cow<'_, Path> {
    let p = path.as_ref();

    if let Some(Component::Normal(first)) = p.components().next()
        && first == "~"
        && let Some(home) = home_dir()
    {
        let mut comps = p.components();
        comps.next(); // skip "~"
        let rest = comps.as_path();
        let expanded = if rest.as_os_str().is_empty() {
            home
        } else {
            home.join(rest)
        };
        Cow::Owned(expanded)
    } else {
        Cow::Borrowed(p)
    }
}

fn expand_tilde_in_string(line: &str) -> Cow<'_, str> {
    match expand_tilde_in_path(Path::new(line)) {
        Cow::Owned(p) => Cow::Owned(p.to_string_lossy().into_owned()),
        Cow::Borrowed(_) => Cow::Borrowed(line),
    }
}

pub fn expand_tildes_in_multiline_string(text: &str) -> String {
    let had_trailing_newline = text.ends_with('\n');

    let mut expanded = text
        .lines()
        .map(expand_tilde_in_string)
        .collect::<Vec<_>>()
        .join("\n");

    if had_trailing_newline {
        expanded.push('\n');
    }

    expanded
}

pub fn collapse_to_tilde<P: AsRef<Path>>(path: P) -> String {
    let p = path.as_ref();

    if let Some(home) = home_dir()
        && let Ok(stripped) = p.strip_prefix(&home)
    {
        if stripped.as_os_str().is_empty() {
            return "~".to_owned();
        }

        return PathBuf::from("~")
            .join(stripped)
            .to_string_lossy()
            .into_owned();
    }

    p.to_string_lossy().into_owned()
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[cfg(test)]
mod tests {
    use super::*;
    use normalize_path::NormalizePath as _;
    use proptest::prelude::*;
    use proptest::strategy::Strategy;
    use proptest::string::string_regex;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_path_strips_trailing_slash() {
        assert_eq!(
            normalize_path(Path::new("/foo/bar/")),
            PathBuf::from("/foo/bar")
        );
    }

    #[test]
    fn test_normalize_path_no_change_without_trailing_slash() {
        assert_eq!(
            normalize_path(Path::new("/foo/bar")),
            PathBuf::from("/foo/bar")
        );
    }

    #[test]
    fn test_resolve_existing_dir_absolute_exists() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().canonicalize().unwrap();
        let result = resolve_existing_dir(dir.to_str().unwrap());
        assert_eq!(result, Some(normalize_path(&dir)));
    }

    #[test]
    fn test_resolve_existing_dir_nonexistent_returns_none() {
        assert_eq!(
            resolve_existing_dir("/this/path/absolutely/does/not/exist/xyz_abc"),
            None
        );
    }

    #[test]
    fn test_resolve_existing_dir_file_returns_none() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.txt");
        std::fs::write(&file, "content").unwrap();
        assert_eq!(resolve_existing_dir(file.to_str().unwrap()), None);
    }

    #[test]
    fn test_resolve_existing_dir_tilde_expands_to_home() {
        let home = home_dir().expect("HOME must be set for this test");
        if home.is_dir() {
            let result = resolve_existing_dir("~");
            assert_eq!(result, Some(normalize_path(&home)));
        }
    }

    fn generate_unix_path() -> impl Strategy<Value = String> {
        let component_char = r"[^/]+";
        let components = proptest::collection::vec(
            string_regex(component_char).expect("string_regex failed"),
            1..6,
        );
        let base_path = components.prop_map(|comps| comps.join("/"));
        base_path
            .prop_flat_map(|s| prop_oneof![Just(format!("~/{s}")), Just(format!("/{s}")), Just(s)])
    }

    proptest! {
        #[test]
        fn test_tilde_expand_collapse(path in generate_unix_path()) {
            let normalized_path = Path::new(&path).normalize();
            let expanded = expand_tilde_in_path(&normalized_path);
            let collapsed = collapse_to_tilde(&expanded);

            prop_assert_eq!(collapsed, normalized_path.to_string_lossy());
        }
    }

    #[test]
    fn test_expand_tildes_in_multiline_string() {
        let home = home_dir().unwrap();
        let home_str = home.to_str().unwrap();

        assert_eq!(expand_tildes_in_multiline_string(""), "");
        assert_eq!(expand_tildes_in_multiline_string("~"), home_str);
        assert_eq!(expand_tildes_in_multiline_string("\n"), "\n");
        assert_eq!(
            expand_tildes_in_multiline_string("/etc/hosts"),
            "/etc/hosts"
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/config"),
            format!("{home_str}/config")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file1\n~/dir/file2"),
            format!("{home_str}/file1\n{home_str}/dir/file2")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("/absolute/path\nrelative/path"),
            "/absolute/path\nrelative/path"
        );
        assert_eq!(
            expand_tildes_in_multiline_string(
                "~/file1\n/absolute/path\n~/dir/file2\nrelative/path"
            ),
            format!("{home_str}/file1\n/absolute/path\n{home_str}/dir/file2\nrelative/path",)
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file~name"),
            format!("{home_str}/file~name")
        );
        assert_eq!(
            expand_tildes_in_multiline_string("~/file1\n/absolute/path\n"),
            format!("{home_str}/file1\n/absolute/path\n")
        );
    }
}
