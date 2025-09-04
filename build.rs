use clap::CommandFactory as _;
use clap_mangen::Man;
use std::fs::{create_dir_all, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};

include!("src/cli.rs");

fn embed_hooks() {
    let mut entries = Vec::new();

    let read_dir: Vec<fs::DirEntry> = fs::read_dir(Path::new("hooks"))
        .expect("Can't read hooks dir")
        .filter_map(Result::ok)
        .collect();

    for entry in read_dir {
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

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");
}

fn write_config_man_page(man_dir: &Path) {
    let config_contents = include_str!("config/template-memy.toml");

    let preamble = r#".TH memy 5 "" "memy Manual"
.SH NAME
memy.toml \- configuration file for memy

.SH DESCRIPTION
The configuration file for memy is written in TOML format and by default is read from ~/.config/memy/memy.toml.
Below is a template with all available options and their defaults.
You can use this as a starting point to create your own configuration.

"#;

    let body = format!(".SH CONFIGURATION TEMPLATE\n.nf\n{config_contents}\n.fi\n");
    let manpage = format!("{preamble}{body}");

    let manpage_path = man_dir.join("memy.toml.5");
    fs::write(&manpage_path, manpage).expect("Failed to write manpage");

    println!("cargo:rerun-if-changed=config/template-memy.toml");
}

fn write_man_page(
    man_dir: &Path,
    file_base_name: String,
    cmd: clap::Command,
) -> std::io::Result<()> {
    let file = File::create(man_dir.join(file_base_name))?;
    let mut writer = BufWriter::new(file);
    Man::new(cmd).render(&mut writer)?;

    Ok(())
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

    write_man_page(&man_dir, "memy.1".to_owned(), main_cmd.clone())?;

    for subcmd in main_cmd.get_subcommands() {
        let full_subcmd_name = format!("memy {}", subcmd.get_name());

        let sub = main_cmd
            .find_subcommand(subcmd.get_name())
            .expect("Subcommand not found")
            .clone()
            .display_name(&full_subcmd_name)
            .bin_name(&full_subcmd_name);
        write_man_page(&man_dir, format!("memy-{}.1", subcmd.get_name()), sub)?;
    }

    write_config_man_page(&man_dir);

    println!("cargo:rerun-if-changed=src/cli.rs");

    Ok(())
}

fn main() {
    get_git_version();
    embed_hooks();
    build_man_pages().expect("Failed to build man page");
}
