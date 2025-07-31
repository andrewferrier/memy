mod support;
use support::*;

#[test]
fn test_plugin_lfrc() {
    let (_db_temp, db_path) = temp_dir();
    let mut cmd = memy_cmd(&db_path, None, &["plugin", "lfrc"]);

    cmd.assert().success().stdout(
        r"cmd on-cd &{{
    memy note ${PWD} &
}}
",
    );
}
