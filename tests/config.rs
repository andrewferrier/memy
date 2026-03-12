mod support;
use support::*;

#[test]
fn test_invalid_config_key() {
    let ctx = TestContext::new();

    create_config_file(&ctx.config_path, "foo=\"bar\"");

    let output = memy_cmd(None, Some(&ctx.config_path), &["list"], vec![]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown field"));
    assert!(!output.status.success());
}

#[test]
fn test_invalid_config_datatype() {
    let ctx = TestContext::new();

    create_config_file(&ctx.config_path, "normalize_symlinks_on_note=100");

    let output = memy_cmd(None, Some(&ctx.config_path), &["list"], vec![]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid type"));
    assert!(!output.status.success());
}
