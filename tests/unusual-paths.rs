mod support;
use support::*;

#[test]
fn test_file_with_space_in_filename() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let filename = "file with space.txt";
    let file_path = create_test_file(&working_path, filename, "test content");

    note_path(&db_path, None, file_path.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &[]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], file_path.to_str().unwrap());
}

#[test]
fn test_file_with_emoji_in_filename() {
    let (_db_temp, db_path) = temp_dir();
    let (_working_temp, working_path) = temp_dir();

    let filename = "file_😀.txt";
    let file_path = create_test_file(&working_path, filename, "test content");

    note_path(&db_path, None, file_path.to_str().unwrap(), 1, false);

    let lines = list_paths(&db_path, None, &[]);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], file_path.to_str().unwrap());
}
