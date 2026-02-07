mod common;

use common::run_stdout;
use serde_json::Value;

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

    let capped_out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/main.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-targeted",
        "2",
        "--json",
    ]);
    let capped_again_out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/main.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-targeted",
        "2",
        "--json",
    ]);
    assert_eq!(capped_out, capped_again_out);

    let capped_payload: Value =
        serde_json::from_str(&capped_out).expect("verify-plan json should parse");
    let capped_results = capped_payload["results"]
        .as_array()
        .expect("results should be array");
    let capped_targeted = capped_results
        .iter()
        .filter(|row| row["scope"] == "targeted")
        .count();
    assert_eq!(capped_targeted, 2);
    assert!(
        capped_results
            .iter()
            .any(|row| row["scope"] == "full_suite" && row["step"] == "cargo test")
    );

    let zero_out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/main.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-targeted",
        "0",
        "--json",
    ]);
    let zero_payload: Value =
        serde_json::from_str(&zero_out).expect("verify-plan json should parse");
    let zero_results = zero_payload["results"]
        .as_array()
        .expect("results should be array");

    assert_eq!(
        zero_results
            .iter()
            .filter(|row| row["scope"] == "targeted")
            .count(),
        0
    );
    assert_eq!(
        zero_results
            .iter()
            .filter(|row| row["scope"] == "full_suite")
            .count(),
        1
    );
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
