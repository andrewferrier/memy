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

include!("src/utils/cli.rs");
include!("src/utils/denylist_default.rs");

fn get_git_version() {
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            format!(
                "v{}",
                std::env::var("CARGO_PKG_VERSION").unwrap_or_default()
            )
        });

    println!("cargo:rustc-env=GIT_VERSION={version}");
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

    println!("cargo:rerun-if-changed=src/utils/cli.rs");

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

    for subcmd in main_cmd.get_subcommands().filter(|s| !s.is_hide_set()) {
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

    println!("cargo:rerun-if-changed=src/utils/cli.rs");

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
    build_man_pages(&build_root_dir).expect("Failed to build man page");
    generate_completions(&build_root_dir).expect("Failed to generate shell completions");
}
