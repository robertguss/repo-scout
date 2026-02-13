mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn exported_api() -> i32 {
    42
}

fn local_dead_leaf() -> i32 {
    5
}

fn local_used() -> i32 {
    11
}
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
fn milestone103_dead_json_includes_confidence_and_reason() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--scope",
        "production",
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("dead json");
    let rows = json["results"].as_array().expect("results array");
    let dead_entry = rows
        .iter()
        .find(|entry| entry["symbol"] == "local_dead_leaf")
        .expect("expected local dead leaf in output");
    assert!(dead_entry["confidence"].is_string(), "output: {out}");
    assert!(dead_entry["reason"].is_string(), "output: {out}");
}
