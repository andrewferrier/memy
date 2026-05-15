use assert_cmd::Command;
use core::time::Duration;
use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

#[path = "../tests/support.rs"]
mod support;

fn find_x_real_entries(
    x: usize,
    kind: &str,
    predicate: impl Fn(&walkdir::DirEntry) -> bool,
) -> Vec<String> {
    let mut list = Vec::new();

    if Path::new("/usr").exists() {
        for entry in walkdir::WalkDir::new("/usr")
            .into_iter()
            .filter_map(core::result::Result::ok)
        {
            if predicate(&entry) {
                list.push(entry.path().to_string_lossy().to_string());
                if list.len() >= x {
                    break;
                }
            }
        }
    }

    assert_eq!(list.len(), x, "Cannot find {x} real {kind}s");
    list
}

fn find_x_real_files(x: usize) -> Vec<String> {
    find_x_real_entries(x, "file", |e| e.file_type().is_file())
}

fn find_x_real_dirs(x: usize) -> Vec<String> {
    find_x_real_entries(x, "dir", |e| e.file_type().is_dir())
}

/// Initialises a fresh memy SQLite DB and populates it with `paths` using
/// deterministic `noted_count` and `last_noted_timestamp` values.
fn setup_list_bench_db(db_dir: &Path, paths: &[String]) {
    let mut conn = rusqlite::Connection::open(db_dir.join("memy.sqlite3"))
        .expect("Failed to open SQLite database");

    conn.execute(
        "CREATE TABLE paths (
            path TEXT PRIMARY KEY,
            noted_count INTEGER NOT NULL,
            last_noted_timestamp INTEGER NOT NULL
        )",
        [],
    )
    .expect("Failed to create paths table");
    conn.execute("PRAGMA user_version = 1;", [])
        .expect("Failed to set user_version");

    let base_timestamp: i64 = 1_700_000_000;
    let tx = conn.transaction().expect("Failed to start transaction");
    for (i, path) in paths.iter().enumerate() {
        let count = i64::try_from(i % 100).expect("index fits i64") + 1;
        let timestamp = base_timestamp - i64::try_from(i).expect("index fits i64") * 3600;
        tx.execute(
            "INSERT INTO paths (path, noted_count, last_noted_timestamp) VALUES (?1, ?2, ?3)",
            rusqlite::params![path, count, timestamp],
        )
        .expect("Failed to insert path");
    }
    tx.commit().expect("Failed to commit transaction");
}

fn get_entries_in_sqlite(db_dir: &Path) -> usize {
    let conn = rusqlite::Connection::open(db_dir.join("memy.sqlite3"))
        .expect("Failed to open SQLite database");
    conn.query_row("SELECT COUNT(*) FROM paths;", [], |row| row.get(0))
        .expect("Failed to query paths table")
}

fn benchmark_note_command(c: &mut Criterion) {
    let file_count = 5000;

    let file_list = find_x_real_files(file_count);

    c.bench_function(format!("memy note {file_count} files").as_str(), |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;

            for _ in 0..iters {
                let temp_dir = tempdir().expect("Failed to create temp dir");
                let db_dir = temp_dir.path();

                // START TIMING
                let start = Instant::now();
                let mut cmd = Command::cargo_bin("memy").expect("Failed to find memy binary");
                cmd.arg("note")
                    .arg("--config")
                    .arg("import_on_first_use=false")
                    .args(&file_list)
                    .env("MEMY_DB_DIR", db_dir)
                    .assert()
                    .success();
                let elapsed = start.elapsed();
                // STOP TIMING

                let entry_count = get_entries_in_sqlite(db_dir);
                assert_eq!(
                    entry_count, file_count,
                    "Expected {file_count} records in paths table, but found {entry_count}"
                );

                temp_dir.close().expect("Failed to clean up temp dir");

                total += elapsed;
            }
            total
        });
    });
}

fn benchmark_import_fasd(c: &mut Criterion) {
    let file_count = 5000;

    let temp_dir_cache = tempdir().expect("Failed to create temp dir");
    let cache_dir = temp_dir_cache.path();

    let temp_dir_filesystem = tempdir().expect("Failed to create temp dir");
    let filesystem_dir = temp_dir_filesystem.path();

    let mut fasd_entries = Vec::new();
    for i in 0..file_count {
        let test_file_path =
            support::create_test_file(filesystem_dir, &format!("test_file_{i}"), "test content");
        let entry = format!(
            "{}|{}|{}",
            test_file_path.to_string_lossy(),
            10.0 + i as f64,
            1_633_036_800 + i * 86400
        );
        fasd_entries.push(entry);
    }

    let fasd_state_file = cache_dir.join("fasd");
    fs::write(&fasd_state_file, fasd_entries.join("\n"))
        .expect("Failed to write mock fasd state file");

    c.bench_function(
        format!("import fasd state file with {file_count} entries").as_str(),
        |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;

                for _ in 0..iters {
                    let temp_dir_db = tempdir().expect("Failed to create temp dir");
                    let db_dir = temp_dir_db.path();

                    let mut cmd = Command::cargo_bin("memy").expect("Failed to find memy binary");

                    // START TIMING
                    let start = Instant::now();
                    cmd.arg("list")
                        .env("MEMY_DB_DIR", db_dir)
                        .env("XDG_CACHE_HOME", cache_dir)
                        .env("XDG_DATA_HOME", cache_dir)
                        .env("_ZO_DATA_DIR", cache_dir)
                        .assert()
                        .success();
                    let elapsed = start.elapsed();

                    // STOP TIMING
                    total += elapsed;

                    let entry_count = get_entries_in_sqlite(temp_dir_db.path());
                    assert_eq!(
                        entry_count, file_count,
                        "{entry_count} doesn't match {file_count}"
                    );
                    temp_dir_db.close().expect("Failed to clean up temp dir");
                }
                total
            });
        },
    );
}

fn benchmark_list_command(c: &mut Criterion) {
    let file_count = 48_000;
    let dir_count = 2_000;
    let total = file_count + dir_count;

    let mut paths = find_x_real_files(file_count);
    paths.extend(find_x_real_dirs(dir_count));

    let temp_dir_db = tempdir().expect("Failed to create temp dir for DB");
    let db_dir = temp_dir_db.path().to_path_buf();

    setup_list_bench_db(&db_dir, &paths);

    let entry_count = get_entries_in_sqlite(&db_dir);
    assert_eq!(
        entry_count, total,
        "Expected {total} entries in DB, found {entry_count}"
    );

    c.bench_function(format!("memy list {total} entries").as_str(), |b| {
        b.iter_custom(|iters| {
            let mut total_time = Duration::ZERO;
            for _ in 0..iters {
                let start = Instant::now();
                Command::cargo_bin("memy")
                    .expect("Failed to find memy binary")
                    .arg("list")
                    .arg("--config")
                    .arg("import_on_first_use=false")
                    .env("MEMY_DB_DIR", &db_dir)
                    .assert()
                    .success();
                total_time += start.elapsed();
            }
            total_time
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(60))
        .sample_size(10);
    targets = benchmark_note_command, benchmark_import_fasd, benchmark_list_command
}

criterion_main!(benches);
