mod common;

use serde_json::Value;

#[test]
fn milestone112_schema_command_exposes_registry() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn run() {}\n");
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);

    let out = common::run_stdout(&[
        "schema",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("schema output json");

    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "schema");
    assert!(
        json["data"]["schemas"].is_array(),
        "schema output should expose command schema list"
    );

    let items = json["data"]["schemas"]
        .as_array()
        .expect("schemas array expected");
    assert!(
        items.iter().any(|entry| entry["command"] == "find"),
        "find schema should be listed"
    );
    assert!(
        items.iter().any(|entry| entry["command"] == "resolve"),
        "resolve schema should be listed"
    );
}
