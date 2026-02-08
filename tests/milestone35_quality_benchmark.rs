mod common;

use serde_json::Value;
use std::path::Path;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn write_semantic_precision_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/util_a.ts",
        include_str!("fixtures/phase7/semantic_precision/src/util_a.ts"),
    );
    common::write_file(
        repo,
        "src/util_b.ts",
        include_str!("fixtures/phase7/semantic_precision/src/util_b.ts"),
    );
    common::write_file(
        repo,
        "src/app.ts",
        include_str!("fixtures/phase7/semantic_precision/src/app.ts"),
    );
    common::write_file(
        repo,
        "src/pkg_a/util.py",
        include_str!("fixtures/phase7/semantic_precision/src/pkg_a/util.py"),
    );
    common::write_file(
        repo,
        "src/pkg_b/util.py",
        include_str!("fixtures/phase7/semantic_precision/src/pkg_b/util.py"),
    );
    common::write_file(
        repo,
        "src/py_app.py",
        include_str!("fixtures/phase7/semantic_precision/src/py_app.py"),
    );
    common::write_file(
        repo,
        "tests/semantic_helper_test.rs",
        "#[test]\nfn helper_smoke() {\n    helper();\n    helper();\n}\n",
    );
}

fn diff_impact_payload(repo: &Path, changed_file: &str) -> Value {
    let output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        changed_file,
        "--repo",
        repo.to_str().expect("repo path should be utf-8"),
        "--json",
    ]);
    serde_json::from_str(&output).expect("diff-impact json should parse")
}

fn impact_payload(repo: &Path, symbol: &str) -> Value {
    let output = run_stdout(&[
        "impact",
        symbol,
        "--repo",
        repo.to_str().expect("repo path should be utf-8"),
        "--json",
    ]);
    serde_json::from_str(&output).expect("impact json should parse")
}

#[test]
fn milestone35_diff_impact_semantic_confidence_ranking() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be array");

    let semantic_row = results
        .iter()
        .find(|item| {
            item["result_kind"] == "impacted_symbol"
                && item["symbol"] == "run"
                && item["relationship"] == "called_by"
        })
        .expect("expected semantic impacted caller row for run");
    let semantic_score = semantic_row["score"]
        .as_f64()
        .expect("semantic row score should be f64");
    assert!(
        semantic_score >= 0.96,
        "semantic caller rows should receive calibrated high confidence score"
    );

    for fallback_row in results
        .iter()
        .filter(|item| item["result_kind"] == "test_target")
    {
        let fallback_score = fallback_row["score"]
            .as_f64()
            .expect("fallback score should be f64");
        assert!(
            semantic_score > fallback_score,
            "semantic caller row should outrank text fallback test-target rows"
        );
    }
}

#[test]
fn milestone35_impact_semantic_rows_rank_deterministically() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload_a = impact_payload(repo.path(), "helper");
    let payload_b = impact_payload(repo.path(), "helper");
    assert_eq!(payload_a, payload_b);

    let results = payload_a["results"]
        .as_array()
        .expect("impact results should be array");
    assert!(
        results.len() >= 2,
        "expected TypeScript and Python caller rows"
    );
    assert!(results.iter().any(|item| item["symbol"] == "run"));
    assert!(results.iter().any(|item| item["symbol"] == "run_py"));
    assert!(results.iter().all(|item| {
        item["relationship"] == "called_by"
            && item["score"].as_f64().is_some_and(|score| score >= 0.96)
    }));
}

#[test]
fn milestone35_fixture_quality_benchmark_is_stable() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let ts_diff_a = diff_impact_payload(repo.path(), "src/util_a.ts");
    let ts_diff_b = diff_impact_payload(repo.path(), "src/util_a.ts");
    assert_eq!(ts_diff_a, ts_diff_b);

    let py_diff = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let impact = impact_payload(repo.path(), "helper");

    let mut semantic_scores = Vec::new();
    for payload in [&ts_diff_a, &py_diff] {
        let rows = payload["results"]
            .as_array()
            .expect("diff-impact results should be array");
        for row in rows {
            if row["result_kind"] == "impacted_symbol" && row["relationship"] == "called_by" {
                semantic_scores.push(
                    row["score"]
                        .as_f64()
                        .expect("called_by score should be f64"),
                );
            }
        }
    }
    assert!(
        semantic_scores.len() >= 2,
        "benchmark fixture should surface both TypeScript and Python semantic caller rows"
    );
    let average_score = semantic_scores.iter().sum::<f64>() / semantic_scores.len() as f64;
    assert!(
        average_score >= 0.96,
        "benchmark semantic caller average score should stay in calibrated high-confidence band"
    );

    let impact_rows = impact["results"]
        .as_array()
        .expect("impact results should be array");
    assert!(impact_rows.iter().any(|row| row["symbol"] == "run"));
    assert!(impact_rows.iter().any(|row| row["symbol"] == "run_py"));
}
