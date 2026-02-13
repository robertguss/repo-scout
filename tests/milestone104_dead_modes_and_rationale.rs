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

fn private_dead() -> i32 {
    3
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
fn milestone104_dead_aggressive_expands_results() {
    let repo = setup_repo();

    let default_out = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let aggressive_out = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--aggressive",
        "--json",
    ]);

    let default_json: Value = serde_json::from_str(&default_out).expect("default dead json");
    let aggressive_json: Value =
        serde_json::from_str(&aggressive_out).expect("aggressive dead json");

    let default_rows = default_json["results"].as_array().expect("default rows");
    let aggressive_rows = aggressive_json["results"]
        .as_array()
        .expect("aggressive rows");

    assert!(
        default_rows
            .iter()
            .all(|entry| entry["symbol"] != "exported_api"),
        "default mode should suppress exported_api: {default_out}"
    );
    assert!(
        aggressive_rows
            .iter()
            .any(|entry| entry["symbol"] == "exported_api"),
        "aggressive mode should include exported_api: {aggressive_out}"
    );
}
