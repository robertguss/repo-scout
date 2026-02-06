mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone9_impact_terminal() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/graph/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "impact",
        "validate",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(out.contains("command: impact"));
    assert!(out.contains("query: validate"));
    assert!(out.contains("called_by"));
    assert!(out.contains("prepare"));
}

#[test]
fn milestone9_impact_json_schema() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/graph/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "impact",
        "persist",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("impact --json should be valid json");

    assert_eq!(payload["schema_version"], 2);
    assert_eq!(payload["command"], "impact");
    assert_eq!(payload["query"], "persist");
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["relationship"], "called_by");
    assert_eq!(results[0]["confidence"], "graph_likely");
}

#[test]
fn milestone9_context_budgeted_terminal() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "context",
        "--task",
        "modify start_engine and update callers",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "200",
    ]);

    assert!(out.contains("command: context"));
    assert!(out.contains("budget: 200"));
    assert!(out.contains("results: 1"));
    assert!(out.contains("why:"));
}

#[test]
fn milestone9_context_json_schema() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "context",
        "--task",
        "modify start_engine and update callers",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "400",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("context --json should be valid json");

    assert_eq!(payload["schema_version"], 2);
    assert_eq!(payload["command"], "context");
    assert_eq!(payload["budget"], 400);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert!(results[0]["why_included"].is_string());
    assert!(results[0]["confidence"].is_string());
}
