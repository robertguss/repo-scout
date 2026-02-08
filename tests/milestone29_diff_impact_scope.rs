mod common;

use common::run_stdout;
use serde_json::Value;

#[test]
fn milestone29_diff_impact_changed_symbol_filters_seed_rows() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_alpha() {}\n\npub fn changed_beta() {}\n\npub fn caller_alpha() {\n    changed_alpha();\n}\n\npub fn caller_beta() {\n    changed_beta();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-symbol",
        "changed_beta",
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

    let changed_rows = results
        .iter()
        .filter(|row| {
            row["result_kind"] == "impacted_symbol" && row["relationship"] == "changed_symbol"
        })
        .collect::<Vec<_>>();
    assert!(!changed_rows.is_empty());
    assert!(
        changed_rows
            .iter()
            .all(|row| row["symbol"] == "changed_beta")
    );

    assert!(results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == "caller_beta"
            && row["distance"] == 1
    }));
    assert!(!results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == "caller_alpha"
            && row["distance"] == 1
    }));
}

#[test]
fn milestone29_diff_impact_exclude_changed_hides_distance_zero_rows() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_beta() {}\n\npub fn caller_beta() {\n    changed_beta();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-symbol",
        "changed_beta",
        "--exclude-changed",
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

    assert!(
        !results
            .iter()
            .any(|row| row["result_kind"] == "impacted_symbol"
                && row["relationship"] == "changed_symbol")
    );
    assert!(results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == "caller_beta"
            && row["distance"] == 1
    }));
}

#[test]
fn milestone29_diff_impact_max_results_caps_deterministically() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_alpha() {}\n\npub fn changed_beta() {}\n\npub fn changed_gamma() {}\n\npub fn caller_alpha() {\n    changed_alpha();\n}\n\npub fn caller_beta() {\n    changed_beta();\n}\n\npub fn caller_gamma() {\n    changed_gamma();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let first = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--max-distance",
        "1",
        "--max-results",
        "2",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let second = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--max-distance",
        "1",
        "--max-results",
        "2",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(first, second);

    let payload: Value = serde_json::from_str(&first).expect("diff-impact json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(results.len(), 2);
}
