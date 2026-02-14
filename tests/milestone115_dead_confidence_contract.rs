mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn api() -> i32 { helper() }
fn helper() -> i32 { 1 }
fn dead_leaf() -> i32 { 2 }
"#,
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone115_dead_reports_confidence_reason_and_mode_contract() {
    let repo = setup_repo();
    let repo_path = repo.path().to_str().expect("repo path utf-8");

    let conservative: Value = serde_json::from_str(&common::run_stdout(&[
        "dead",
        "--repo",
        repo_path,
        "--json",
        "--mode",
        "conservative",
    ]))
    .expect("dead json");

    assert_eq!(conservative["command"], "dead");
    assert_eq!(conservative["mode"], "conservative");
    assert!(conservative["results"].is_array());
    if let Some(first) = conservative["results"]
        .as_array()
        .and_then(|rows| rows.first())
    {
        assert!(first["confidence"].is_string());
        assert!(first["reason"].is_string());
    }

    let aggressive: Value = serde_json::from_str(&common::run_stdout(&[
        "dead",
        "--repo",
        repo_path,
        "--json",
        "--mode",
        "aggressive",
    ]))
    .expect("dead json");

    assert_eq!(aggressive["mode"], "aggressive");
    let conservative_len = conservative["results"]
        .as_array()
        .map_or(0, |rows| rows.len());
    let aggressive_len = aggressive["results"]
        .as_array()
        .map_or(0, |rows| rows.len());
    assert!(
        aggressive_len >= conservative_len,
        "aggressive mode should be at least as inclusive"
    );
}
