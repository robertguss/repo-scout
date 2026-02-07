mod common;

use common::run_stdout;
use serde_json::Value;

#[test]
fn milestone24_context_matches_relevant_symbols_for_paraphrased_task() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn verify_plan_for_changed_files() {}\n\npub fn tests_for_symbol() {}\n\npub fn files() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "context",
        "--task",
        "update verify plan recommendation quality for changed files and reduce noisy test selection",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "1200",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("context json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["symbol"] == "verify_plan_for_changed_files")
    );
    assert!(
        results
            .iter()
            .any(|row| row["symbol"] == "tests_for_symbol")
    );
}

#[test]
fn milestone24_context_prioritizes_definitions_over_incidental_tokens() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn verify_plan_for_changed_files() {}\n\npub fn verify_plan_orchestrator() {\n    verify_plan_for_changed_files();\n}\n\npub fn plan() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "context",
        "--task",
        "improve verify plan workflow",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "400",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("context json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    let first = results.first().expect("expected at least one result");
    assert_eq!(first["symbol"], "verify_plan_for_changed_files");

    let top_symbols = results
        .iter()
        .take(2)
        .map(|row| row["symbol"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert!(!top_symbols.contains(&"plan".to_string()));
}

#[test]
fn milestone24_context_json_is_stable_with_relevance_scoring() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn verify_plan_for_changed_files() {}\n\npub fn verify_plan_orchestrator() {\n    verify_plan_for_changed_files();\n}\n\npub fn tests_for_symbol() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let first = run_stdout(&[
        "context",
        "--task",
        "improve verify plan workflow and reduce noisy test selection",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "1200",
        "--json",
    ]);
    let second = run_stdout(&[
        "context",
        "--task",
        "improve verify plan workflow and reduce noisy test selection",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "1200",
        "--json",
    ]);

    assert_eq!(first, second);

    let payload: Value = serde_json::from_str(&first).expect("context json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(results.iter().any(|row| {
        row["why_included"]
            .as_str()
            .expect("why_included should be string")
            .contains("token-overlap relevance")
    }));
}
