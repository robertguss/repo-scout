mod common;

use serde_json::Value;

#[test]
fn context_scoring_differentiates_direct_vs_tangential() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn index_file(path: &str) -> bool { true }\n\
         pub fn validate_schema() { /* mentions index in comment */ }\n\
         pub fn index_repository(path: &str) { index_file(path); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "context", "--task", "understand how indexing works",
        "--repo", repo.path().to_str().unwrap(), "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    let results = payload["results"].as_array().unwrap();

    assert!(
        results.len() >= 2,
        "expected at least 2 results, got {}: {results:?}",
        results.len()
    );
    let scores: Vec<f64> = results.iter()
        .map(|r| r["score"].as_f64().unwrap())
        .collect();
    let unique_scores: std::collections::HashSet<u64> = scores.iter()
        .map(|s| (s * 1000.0) as u64)
        .collect();
    assert!(
        unique_scores.len() > 1,
        "scores should be differentiated, got: {scores:?}"
    );
}
