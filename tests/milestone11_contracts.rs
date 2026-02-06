mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone11_diff_impact_json_contract() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&out).expect("diff-impact --json should produce valid json");
    assert_eq!(payload["schema_version"], 3);
    assert_eq!(payload["command"], "diff-impact");
    assert_eq!(payload["max_distance"], 2);
    assert_eq!(payload["include_tests"], true);

    let changed_files = payload["changed_files"]
        .as_array()
        .expect("changed_files should be an array");
    assert_eq!(
        changed_files,
        &vec![Value::String("src/lib.rs".to_string())]
    );

    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["result_kind"], "impacted_symbol");
    assert!(results[0]["qualified_symbol"].is_string());
    assert!(results[0]["provenance"].is_string());
}

#[test]
fn milestone11_explain_cli_contract() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "explain",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("command: explain"));
    assert!(out.contains("query: compute_plan"));
    assert!(out.contains("results:"));
}

#[test]
fn milestone11_explain_json_contract() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "explain",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
        "--include-snippets",
    ]);

    let payload: Value =
        serde_json::from_str(&out).expect("explain --json should produce valid json");
    assert_eq!(payload["schema_version"], 3);
    assert_eq!(payload["command"], "explain");
    assert_eq!(payload["query"], "compute_plan");
    assert_eq!(payload["include_snippets"], true);

    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert!(results[0]["language"].is_string());
    assert!(results[0]["qualified_symbol"].is_string());
    assert!(results[0]["inbound"].is_object());
    assert!(results[0]["outbound"].is_object());
    let snippet = results[0]["snippet"]
        .as_str()
        .expect("snippet should be present when include_snippets=true");
    assert!(snippet.contains("compute_plan"));
}

#[test]
fn milestone11_diff_impact_cli_contract() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("command: diff-impact"));
    assert!(out.contains("changed_files: 1"));
    assert!(out.contains("results:"));
}
