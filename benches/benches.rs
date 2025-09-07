use assert_cmd::Command;
use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::time::Instant;
use tempfile::tempdir;

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

    let mut group = c.benchmark_group("benchmark_note_command");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    let file_list = find_x_real_files(file_count);

    group.bench_function(format!("memy note {file_count} files"), |b| {
        b.iter(|| {
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

            elapsed
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_note_command);
criterion_main!(benches);
