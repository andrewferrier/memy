use chrono::{DateTime, Utc};
use clap::CommandFactory;
use clap::{Args, Parser, Subcommand};
use env_logger::{Builder, Env};
use log::{info, warn, LevelFilter};
use rusqlite::{params, Connection};
use std::{fs, path::Path};

mod config;
use config::DeniedFilesOnList;
mod db;
mod hooks_generated;
mod utils;

#[derive(Parser)]
#[command(name = "memy")]
#[command(version = env!("GIT_VERSION"))]
#[command(author = "Andrew Ferrier")]
#[command(about = "Track and recall frequently and recently used files or directories.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Enable verbose (info) logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug (very detailed) logging
    #[arg(long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Note usage of one or more paths
    Note(NoteArgs),
    /// List paths by frecency score
    List(ListArgs),
    /// Generate a template memy.toml config file
    GenerateConfig {
        /// Optional output filename for the generated config
        #[arg(value_name = "FILENAME")]
        filename: Option<String>,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for (e.g. bash, zsh)
        #[arg(value_enum)]
        shell: Option<clap_complete::Shell>,
    },
    /// Show contents of a memy hook
    Hook {
        #[arg(value_enum)]
        hook_name: Option<String>,
    },
}

#[derive(Args)]
struct NoteArgs {
    /// One or more paths to note
    #[arg(value_name = "PATHS")]
    paths: Vec<String>,
}

#[derive(Args)]
struct ListArgs {
    /// Show only files in the list
    #[arg(short, long)]
    files_only: bool,

    /// Show only directories in the list
    #[arg(short, long)]
    directories_only: bool,

    /// Include frecency score in output
    #[arg(long)]
    include_frecency_score: bool,
}

const RECENCY_BIAS: f64 = 20000.0;

fn set_logging_level(cli: &Cli) {
    let level = match &cli.command {
        Commands::GenerateConfig { .. } => {
            if cli.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        }
        _ => {
            if cli.debug {
                LevelFilter::Debug
            } else if cli.verbose {
                LevelFilter::Info
            } else {
                LevelFilter::Warn
            }
        }
    };

    Builder::from_env(Env::default().default_filter_or(level.to_string()))
        .target(env_logger::Target::Stderr)
        .init();
}

fn normalize_path_if_needed(path: &Path, normalize: bool) -> String {
    if normalize {
        std::fs::canonicalize(path).ok().map_or_else(
            || path.to_string_lossy().into_owned(),
            |p| p.to_string_lossy().into_owned(),
        )
    } else {
        path.to_string_lossy().into_owned()
    }
}

fn note_path(conn: &Connection, raw_path: &str) {
    let path = Path::new(raw_path);

    if !path.exists() {
        if config::get_missing_files_warn_on_note() {
            warn!("Path {raw_path} does not exist.");
        }

        return;
    }

    let config_normalize = config::get_normalize_symlinks_on_note();
    let clean_path = normalize_path_if_needed(path, config_normalize);

    let matcher = config::get_denylist_matcher();
    if let ignore::Match::Ignore(_matched_pat) = matcher.matched(&clean_path, false) {
        if config::get_denied_files_warn_on_note() {
            warn!("Path denied by denylist pattern.");
        }
        return;
    }

    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, 1, ?2) \
            ON CONFLICT(path) DO UPDATE SET \
                noted_count = noted_count + 1, \
                last_noted_timestamp = excluded.last_noted_timestamp",
        params![clean_path, now],
    )
    .unwrap();
    info!("Path {raw_path} noted");
}

fn calculate_frecency(now: DateTime<Utc>, count: i64, last_noted_timestamp: i64) -> f64 {
    let last_dt = DateTime::from_timestamp(last_noted_timestamp, 0).expect("invalid timestamp");
    let age_secs = now.signed_duration_since(last_dt).num_seconds() as f64;
    count as f64 * (1.0 / (1.0 + age_secs / RECENCY_BIAS))
}

fn list_paths(conn: &Connection, args: &ListArgs) {
    let mut stmt = conn
        .prepare("SELECT path, noted_count, last_noted_timestamp FROM paths")
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let path: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            let last_noted_timestamp: i64 = row.get(2)?;
            Ok((path, count, last_noted_timestamp))
        })
        .unwrap();

    let now = Utc::now();

    let mut results: Vec<(String, f64)> = vec![];

    let list_denied_action = config::get_denied_files_on_list();
    let matcher = config::get_denylist_matcher();

    for (path, count, last_noted_timestamp) in rows.into_iter().flatten() {
        if let ignore::Match::Ignore(_matched_pat) = matcher.matched(&path, false) {
            match list_denied_action {
                DeniedFilesOnList::SkipSilently => {
                    continue;
                }
                DeniedFilesOnList::Warn => {
                    warn!("Path {path} is denied, remaining in database.");
                    continue;
                }
                DeniedFilesOnList::Delete => {
                    conn.execute("DELETE FROM paths WHERE path = ?", params![path.clone()])
                        .unwrap();
                    info!("Path {path} is denied, deleted from database.");
                    continue;
                }
            }
        }

        let Ok(metadata) = fs::metadata(&path) else {
            conn.execute("DELETE FROM paths WHERE path = ?", params![path.clone()])
                .unwrap();
            info!("Path {path} no longer exists, deleted from database.");
            continue;
        };

        if args.files_only && !metadata.is_file() {
            continue;
        }

        if args.directories_only && !metadata.is_dir() {
            continue;
        }

        let frecency = calculate_frecency(now, count, last_noted_timestamp);
        results.push((path, frecency));
    }

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (path, score) in results {
        if args.include_frecency_score {
            println!("{path}\t{score}");
        } else {
            println!("{path}");
        }
    }
}

fn completions(shell: Option<clap_complete::Shell>) {
    let shell = shell
        .or_else(utils::detect_shell)
        .expect("Could not determine shell. Specify one explicitly.");
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}

fn hook_show(hook_name: Option<String>) {
    if let Some(hook_name) = hook_name {
        if let Some(content) = hooks_generated::get_hook_content(&hook_name) {
            print!("{content}");
        } else {
            eprintln!("Hook not found: {hook_name}");
            std::process::exit(1);
        }
    } else {
        println!("Available hooks:");
        for hook in hooks_generated::get_hook_list() {
            println!("{hook}");
        }
    }
}

fn main() {
    let cli = Cli::parse();

    set_logging_level(&cli);

    match cli.command {
        Commands::Note(note_args) => {
            let db_connection = db::open_db();
            for path in note_args.paths {
                note_path(&db_connection, &path);
            }
        }
        Commands::List(list_args) => {
            let db_connection = db::open_db();
            list_paths(&db_connection, &list_args);
        }
        Commands::GenerateConfig { filename } => {
            config::generate_config(filename.as_deref());
        }
        Commands::Completions { shell } => {
            completions(shell);
        }
        Commands::Hook { hook_name } => {
            hook_show(hook_name);
        }
    }
}
