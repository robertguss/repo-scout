mod common;

use common::run_stdout;
use serde_json::Value;

#[test]
fn milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn c_changed() {}\n\npub fn b_calls_c() {\n    c_changed();\n}\n\npub fn a_calls_b() {\n    b_calls_c();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:1",
        "--max-distance",
        "2",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("diff-impact json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == "a_calls_b"
            && row["distance"] == 2
    }));
}

#[test]
fn milestone25_diff_impact_respects_max_distance_bound() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn c_changed() {}\n\npub fn b_calls_c() {\n    c_changed();\n}\n\npub fn a_calls_b() {\n    b_calls_c();\n}\n\npub fn d_calls_a() {\n    a_calls_b();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:1",
        "--max-distance",
        "1",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("diff-impact json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(!results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol" && row["distance"].as_u64().unwrap_or(0) > 1
    }));
}

#[test]
fn milestone25_diff_impact_handles_cycles_without_duplicate_growth() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_c() {\n    b();\n}\n\npub fn b() {\n    a();\n}\n\npub fn a() {\n    changed_c();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let first = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:1",
        "--max-distance",
        "3",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let second = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:1",
        "--max-distance",
        "3",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(first, second);

    let payload: Value = serde_json::from_str(&first).expect("diff-impact json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(!results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == "changed_c"
            && row["distance"].as_u64().unwrap_or(0) > 0
    }));
}
