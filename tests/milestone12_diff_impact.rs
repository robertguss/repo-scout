mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone12_diff_impact_changed_files_normalization() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    let absolute = repo.path().join("src/lib.rs");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let terminal_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "./src/lib.rs",
        "--changed-file",
        absolute.to_str().unwrap(),
        "--changed-file",
        "src\\lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(terminal_out.contains("changed_files: 1"));
    assert!(terminal_out.contains("src/lib.rs"));

    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "./src/lib.rs",
        "--changed-file",
        absolute.to_str().unwrap(),
        "--changed-file",
        "src\\lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    assert_eq!(payload["schema_version"], 3);
    assert_eq!(payload["command"], "diff-impact");
    assert_eq!(
        payload["changed_files"].as_array().expect("changed_files"),
        &vec![Value::String("src/lib.rs".to_string())]
    );
}

#[test]
fn milestone12_diff_impact_graph_neighbors() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_entry() {\n    helper();\n}\n\npub fn helper() {}\n",
    );
    common::write_file(
        repo.path(),
        "src/other.rs",
        "pub fn watcher() {\n    changed_entry();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(results.iter().any(|item| {
        item["symbol"] == "changed_entry"
            && item["relationship"] == "changed_symbol"
            && item["distance"] == 0
    }));
    assert!(results.iter().any(|item| {
        item["symbol"] == "watcher"
            && item["relationship"] == "called_by"
            && item["distance"] == 1
            && item["confidence"] == "graph_likely"
            && item["provenance"] == "call_resolution"
    }));
}

#[test]
fn milestone12_diff_impact_includes_tests() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n    compute_plan();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(results.iter().any(|item| {
        item["result_kind"] == "test_target"
            && item["target"] == "tests/plan_test.rs"
            && item["target_kind"] == "integration_test_file"
            && item["confidence"].is_string()
            && item["provenance"] == "text_fallback"
    }));
}

#[test]
fn milestone12_diff_impact_exclude_tests_hides_test_targets() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n    compute_plan();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--exclude-tests",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    assert_eq!(payload["include_tests"], false);
    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        !results
            .iter()
            .any(|item| item["result_kind"] == "test_target")
    );
}

#[test]
fn milestone12_diff_impact_deterministic_ordering() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn changed_entry() {\n    helper();\n}\n\npub fn helper() {}\n",
    );
    common::write_file(
        repo.path(),
        "src/other.rs",
        "pub fn watcher() {\n    changed_entry();\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/impact_test.rs",
        "fn changed_entry() {}\n\n#[test]\nfn covers_changed_entry() {\n    changed_entry();\n    changed_entry();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out_a = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let out_b = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out_a, out_b);

    let payload_a: Value =
        serde_json::from_str(&out_a).expect("diff-impact --json should produce valid json");
    let results_a = payload_a["results"]
        .as_array()
        .expect("results should be array");
    assert!(results_a.iter().any(|item| item["distance"] == 1));
    for item in results_a {
        if let Some(confidence) = item["confidence"].as_str() {
            assert!(
                [
                    "graph_exact",
                    "graph_likely",
                    "context_high",
                    "context_medium",
                    "context_low"
                ]
                .contains(&confidence)
            );
        }
        if let Some(provenance) = item["provenance"].as_str() {
            assert!(
                [
                    "ast_definition",
                    "ast_reference",
                    "import_resolution",
                    "call_resolution",
                    "text_fallback"
                ]
                .contains(&provenance)
            );
        }
    }

    let max_zero_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--max-distance",
        "0",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload_zero: Value =
        serde_json::from_str(&max_zero_out).expect("diff-impact --json should produce valid json");
    let results_zero = payload_zero["results"]
        .as_array()
        .expect("results should be array");
    assert!(!results_zero.iter().any(|item| item["distance"] == 1));
}
