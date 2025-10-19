#![allow(clippy::unwrap_used, reason = "unwrap() OK inside build")]

use clap::CommandFactory as _;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, Zsh},
};
use clap_mangen::Man;
use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::Command;
use std::{fs, path::Path};
use tera::{Context, Tera};

include!("src/cli.rs");
include!("src/denylist_default.rs");

fn embed_hooks() {
    let mut entries = Vec::new();

    let read_dir = fs::read_dir(Path::new("hooks"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path());

    for path in read_dir {
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy();
            let content = fs::read_to_string(&path).expect("Failed to read hook file");
            let escaped = content.escape_default().to_string(); // escape for safe inclusion in code
            entries.push(format!("map.insert(\"{filename}\", \"{escaped}\");"));
        }
    }

    let generated_code = format!(
        r"use std::collections::BTreeMap;

pub static HOOKS: std::sync::LazyLock<BTreeMap<&'static str, &'static str>> = std::sync::LazyLock::new(|| {{
    let mut map = BTreeMap::new();
{}
    map
}});",
        entries.join("\n")
    );

    let dest_path = Path::new("src/hooks_generated.rs");
    fs::write(dest_path, generated_code).expect("Failed to write generated code");

    println!("cargo:rerun-if-changed=hooks/");
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

fn generate_completions(build_root_dir: &Path) -> std::io::Result<()> {
    let completions_dir = build_root_dir.join("target/completions");
    create_dir_all(&completions_dir)?;

    let mut cmd = Cli::command();

    generate_to(Bash, &mut cmd, "memy", &completions_dir)?;
    generate_to(Zsh, &mut cmd, "memy", &completions_dir)?;
    generate_to(Fish, &mut cmd, "memy", &completions_dir)?;

    println!("cargo:rerun-if-changed=src/cli.rs");

    Ok(())
}

fn write_config_man_page(man_dir: &Path) {
    let config_contents =
        fs::read_to_string("config/template-memy.toml").expect("Can't read rendered template");

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

fn build_man_pages(build_root_dir: &Path) -> std::io::Result<()> {
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

fn render_config_template() -> Result<(), Box<dyn core::error::Error>> {
    let template_path = "config/template-memy.toml.tmpl";
    let output_path = "config/template-memy.toml";

    let mut tera = Tera::default();
    tera.add_template_file(template_path, Some("memy"))?;

    let mut ctx = Context::new();
    ctx.insert("builtins", &DEFAULT_DENYLIST);

    let rendered = tera.render("memy", &ctx)?;

    if let Some(parent) = Path::new(output_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, rendered)?;

    println!("cargo:rerun-if-changed={template_path}");

    Ok(())
}

fn main() {
    let build_root_dir = std::env::var("CARGO_MANIFEST_DIR").ok().map_or_else(
        || std::env::current_dir().expect("Failed to get current dir"),
        PathBuf::from,
    );

    get_git_version();
    render_config_template().expect("Failed to render config template");
    embed_hooks();
    build_man_pages(&build_root_dir).expect("Failed to build man page");
    generate_completions(&build_root_dir).expect("Failed to generate shell completions");
}
