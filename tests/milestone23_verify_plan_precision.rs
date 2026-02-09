mod common;

use common::run_stdout;
use serde_json::Value;
use std::path::Path;

fn verify_plan_json(repo_root: &Path, max_targeted: Option<&str>) -> Value {
    let repo_arg = repo_root.to_str().expect("repo path should be utf-8");
    let mut args = vec![
        "verify-plan",
        "--changed-file",
        "src/main.rs",
        "--repo",
        repo_arg,
    ];
    if let Some(max_targeted) = max_targeted {
        args.extend_from_slice(&["--max-targeted", max_targeted]);
    }
    args.push("--json");
    let out = run_stdout(&args);
    serde_json::from_str(&out).expect("verify-plan json should parse")
}

fn verify_plan_results(payload: &Value) -> &[Value] {
    payload["results"]
        .as_array()
        .expect("results should be array")
}

fn count_scope(results: &[Value], scope: &str) -> usize {
    results.iter().filter(|row| row["scope"] == scope).count()
}

#[test]
fn milestone23_verify_plan_downranks_generic_changed_symbols() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.rs",
        "use std::path::Path;\n\npub fn output() -> usize {\n    1\n}\n\npub fn verify_signal() -> usize {\n    output()\n}\n\npub fn use_path(value: &Path) -> bool {\n    value.exists()\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/generic_output.rs",
        "#[test]\nfn generic_output() {\n    let _ = output();\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/generic_path.rs",
        "#[test]\nfn generic_path() {\n    let _ = std::path::Path::new(\".\");\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/specific_verify_signal.rs",
        "#[test]\nfn specific_verify_signal() {\n    let _ = verify_signal();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/main.rs",
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
            .any(|row| row["step"] == "cargo test --test specific_verify_signal")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["step"] == "cargo test --test generic_output")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["step"] == "cargo test --test generic_path")
    );
}

#[test]
fn milestone23_verify_plan_applies_targeted_cap_deterministically() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.rs",
        "pub fn alpha() -> usize { 1 }\npub fn beta() -> usize { 2 }\npub fn gamma() -> usize { 3 }\npub fn delta() -> usize { 4 }\n",
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
    common::write_file(
        repo.path(),
        "tests/gamma_case.rs",
        "#[test]\nfn gamma_case() { let _ = gamma(); }\n",
    );
    common::write_file(
        repo.path(),
        "tests/delta_case.rs",
        "#[test]\nfn delta_case() { let _ = delta(); }\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let capped_payload = verify_plan_json(repo.path(), Some("2"));
    let capped_again_payload = verify_plan_json(repo.path(), Some("2"));
    assert_eq!(capped_payload, capped_again_payload);

    let capped_results = verify_plan_results(&capped_payload);
    assert_eq!(count_scope(capped_results, "targeted"), 2);
    assert!(
        capped_results
            .iter()
            .any(|row| row["scope"] == "full_suite" && row["step"] == "cargo test")
    );

    let zero_payload = verify_plan_json(repo.path(), Some("0"));
    let zero_results = verify_plan_results(&zero_payload);
    assert_eq!(count_scope(zero_results, "targeted"), 0);
    assert_eq!(count_scope(zero_results, "full_suite"), 1);
}

#[test]
fn milestone23_verify_plan_keeps_short_meaningful_symbols() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.rs",
        "pub fn api() -> usize {\n    1\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/api_contract.rs",
        "#[test]\nfn api_contract() {\n    let _ = api();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/main.rs",
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
            .any(|row| row["step"] == "cargo test --test api_contract")
    );
}

#[test]
fn milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate() {
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
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-targeted",
        "0",
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
