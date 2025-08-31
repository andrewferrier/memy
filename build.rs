use clap::CommandFactory as _;
use clap_mangen::Man;
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, io, path::Path};

include!("src/cli.rs");

fn read_dir_sorted<P: AsRef<Path>>(path: P) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries: Vec<fs::DirEntry> = fs::read_dir(path)?.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);
    Ok(entries)
}

fn embed_hooks() {
    let hooks_dir = Path::new("hooks");

    let mut entries = Vec::new();

    for entry in read_dir_sorted(hooks_dir).expect("Cannot read hook dir") {
        let path = entry.path();
        if path.is_file() {
            let filename = path
                .file_name()
                .expect("Cannot read hook filename")
                .to_string_lossy();
            let content = fs::read_to_string(&path).expect("Failed to read hook file");
            let escaped = content.escape_default().to_string(); // escape for safe inclusion in code
            entries.push(format!("map.insert(\"{filename}\", \"{escaped}\");"));
        }
    }

    let generated_code = format!(
        r"use std::collections::HashMap;

pub static HOOKS: std::sync::LazyLock<HashMap<&'static str, &'static str>> = std::sync::LazyLock::new(|| {{
    let mut map = HashMap::new();
    {}
    map
}});",
        entries.join("\n")
    );

    let dest_path = Path::new("src/hooks_generated.rs");
    fs::write(dest_path, generated_code).expect("Failed to write generated code");

    println!("cargo:rerun-if-changed=plugins");
}

fn get_git_version() {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .expect("Failed to execute git");

    let git_version = String::from_utf8(output.stdout).expect("Cannot get git version");
    println!("cargo:rustc-env=GIT_VERSION={}", git_version.trim());

    // Invalidate the build if HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");
}

fn build_man_pages() -> std::io::Result<()> {
    let build_root_dir = std::env::var("CARGO_MANIFEST_DIR")
        .ok() // converts Result -> Option
        .map_or_else(
            || std::env::current_dir().expect("Failed to get current dir"),
            PathBuf::from,
        );

    let man_dir = build_root_dir.join("target/man");

    create_dir_all(&man_dir)?;

    let main_cmd = Cli::command();

    {
        let file = File::create(man_dir.join("memy.1"))?;
        let mut writer = BufWriter::new(file);
        Man::new(main_cmd.clone()).render(&mut writer)?;
    };

    for subcmd in main_cmd.get_subcommands() {
        let file = File::create(man_dir.join(format!("memy-{}.1", subcmd.get_name())))?;
        let mut writer = BufWriter::new(file);
        let sub = main_cmd
            .find_subcommand(subcmd.get_name())
            .expect("Subcommand not found")
            .clone();
        Man::new(sub).render(&mut writer)?;
    }

    Ok(())
}

fn main() {
    get_git_version();
    embed_hooks();
    build_man_pages().expect("Failed to build man page");
}
