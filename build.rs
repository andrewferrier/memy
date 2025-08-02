use std::process::Command;
use std::{fs, path::Path};

fn embed_hooks() {
    let hooks_dir = Path::new("hooks");

    let mut entries = Vec::new();

    for entry in fs::read_dir(hooks_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy();
            let content = fs::read_to_string(&path).expect("Failed to read hook file");
            let escaped = content.escape_default().to_string(); // escape for safe inclusion in code
            entries.push(format!("map.insert(\"{filename}\", \"{escaped}\");"));
        }
    }

    let generated_code = format!(
        r"use std::collections::HashMap;

static HOOKS: std::sync::LazyLock<HashMap<&'static str, &'static str>> = std::sync::LazyLock::new(|| {{
    let mut map = HashMap::new();
    {}
    map
}});

pub fn get_hook_content(name: &str) -> Option<&'static str> {{
    HOOKS.get(name).copied()
}}

pub fn get_hook_list() -> Vec<&'static str> {{
    HOOKS.keys().copied().collect()
}}",
        entries.join("\n")
    );

    let dest_path = Path::new("src/hooks_generated.rs");
    fs::write(dest_path, generated_code).expect("Failed to write generated code");

    println!("cargo:rerun-if-changed=plugins");
}

fn main() {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .expect("Failed to execute git");

    let git_version = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_VERSION={}", git_version.trim());

    // Invalidate the build if HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");

    embed_hooks();
}
