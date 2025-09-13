use assert_cmd::Command;
use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

#[path = "../tests/support.rs"]
mod support;

fn find_x_real_files(x: usize) -> Vec<String> {
    let mut file_list = Vec::new();
    let usr_dir = Path::new("/usr");
    let mut count = 0;

    if usr_dir.exists() {
        for entry in walkdir::WalkDir::new(usr_dir)
            .into_iter()
            .filter_map(core::result::Result::ok)
        {
            if entry.file_type().is_file() {
                file_list.push(entry.path().to_string_lossy().to_string());
                count += 1;
                if count >= x {
                    break;
                }
            }
        }
    }

    assert_eq!(file_list.len(), x, "Cannot find {x} real files");

    file_list
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

    c.bench_function("import_fasd_state_file", |b| {
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
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(10))
        .measurement_time(Duration::from_secs(60))
        .sample_size(10);
    targets = benchmark_note_command, benchmark_import_fasd
}

criterion_main!(benches);
