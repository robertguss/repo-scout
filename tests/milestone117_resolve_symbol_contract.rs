mod common;

use serde_json::Value;

#[test]
fn milestone117_resolve_returns_candidates_and_ambiguity_metadata() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn run() {}\npub fn runner() { run(); }\n",
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);

    let out = common::run_stdout(&[
        "resolve",
        "run",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("resolve json");

    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "resolve");
    assert!(json["data"]["candidates"].is_array());
    assert!(
        json["data"]["recommended_symbol_id"].is_number()
            || json["data"]["recommended_symbol_id"].is_null()
    );
}
