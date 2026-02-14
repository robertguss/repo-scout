mod common;

use serde_json::Value;

#[test]
fn milestone119_refactor_plan_returns_ranked_actions_and_risk_confidence() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn api() -> i32 { helper() }\nfn helper() -> i32 { 41 }\n",
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);

    let out = common::run_stdout(&[
        "refactor-plan",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("refactor-plan json");

    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "refactor-plan");
    assert!(json["data"]["actions"].is_array());
    let first = json["data"]["actions"]
        .as_array()
        .and_then(|rows| rows.first())
        .expect("at least one action expected");
    assert!(first["risk"].is_string());
    assert!(first["confidence"].is_string());
}
