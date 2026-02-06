mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone10_tests_for_direct_matches() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "tests-for",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("command: tests-for"));
    assert!(out.contains("query: compute_plan"));
    assert!(out.contains("results: 1"));
    assert!(out.contains("tests/plan_test.rs"));
}

#[test]
fn milestone10_tests_for_dedup_confidence() {
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
    let out = run_stdout(&[
        "tests-for",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("results: 1"));
    assert_eq!(out.matches("tests/plan_test.rs").count(), 1);
    assert!(out.contains("[graph_likely"));
}

#[test]
fn milestone10_verify_plan_changed_files() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("command: verify-plan"));
    assert!(out.contains("changed_files: 1"));
    assert!(out.contains("cargo test --test plan_test"));
    assert!(out.contains("cargo test"));
}

#[test]
fn milestone10_verify_plan_skips_non_runnable_test_modules() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/common/mod.rs",
        "pub fn helper() {\n    compute_plan();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(out.contains("cargo test --test plan_test"));
    assert!(!out.contains("cargo test --test mod"));
}

#[test]
fn milestone10_verify_plan_deterministic_recommendations() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );
    common::write_file(
        repo.path(),
        "tests/plan_test.rs",
        "#[test]\nfn compute_plan_smoke_test() {\n    compute_plan();\n    normalize_input();\n    normalize_input();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out_a = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--changed-file",
        "./src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let out_b = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "./src/lib.rs",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out_a, out_b);

    let payload: Value =
        serde_json::from_str(&out_a).expect("verify-plan --json should be valid json");

    assert_eq!(payload["schema_version"], 2);
    assert_eq!(payload["command"], "verify-plan");
    let changed_files = payload["changed_files"]
        .as_array()
        .expect("changed_files should be an array");
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0], "src/lib.rs");

    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["step"], "cargo test --test plan_test");
    assert_eq!(results[0]["scope"], "targeted");
    assert_eq!(results[0]["confidence"], "graph_likely");
    assert_eq!(results[1]["step"], "cargo test");
    assert_eq!(results[1]["scope"], "full_suite");
}
