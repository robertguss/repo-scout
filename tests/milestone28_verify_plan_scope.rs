mod common;

use common::run_stdout;
use serde_json::Value;

#[test]
fn milestone28_verify_plan_changed_line_limits_targeted_symbol_set() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn alpha() -> usize { 1 }\n\npub fn beta() -> usize { 2 }\n",
    );
    common::write_file(
        repo.path(),
        "tests/alpha_case.rs",
        "#[test]\nfn alpha_case() { let _ = alpha(); }\n",
    );
    common::write_file(
        repo.path(),
        "tests/beta_case.rs",
        "#[test]\nfn beta_case() { let _ = beta(); }\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:1",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("verify-plan json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["step"] == "cargo test --test alpha_case")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["step"] == "cargo test --test beta_case")
    );
}

#[test]
fn milestone28_verify_plan_changed_symbol_filters_targeted_recommendations() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn alpha() -> usize { 1 }\n\npub fn beta() -> usize { 2 }\n",
    );
    common::write_file(
        repo.path(),
        "tests/alpha_case.rs",
        "#[test]\nfn alpha_case() { let _ = alpha(); }\n",
    );
    common::write_file(
        repo.path(),
        "tests/beta_case.rs",
        "#[test]\nfn beta_case() { let _ = beta(); }\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--changed-symbol",
        "beta",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("verify-plan json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["step"] == "cargo test --test beta_case")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["step"] == "cargo test --test alpha_case")
    );
}

#[test]
fn milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "tests/changed_focus.rs",
        "#[test]\nfn changed_focus() {\n    assert_eq!(changed_focus_helper(), 1);\n}\n\nfn changed_focus_helper() -> usize {\n    1\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "tests/changed_focus.rs",
        "--changed-line",
        "tests/changed_focus.rs:1:1",
        "--changed-symbol",
        "does_not_exist",
        "--max-targeted",
        "0",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("verify-plan json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["step"] == "cargo test --test changed_focus")
    );
    assert!(
        results
            .iter()
            .any(|row| row["step"] == "cargo test" && row["scope"] == "full_suite")
    );
}
