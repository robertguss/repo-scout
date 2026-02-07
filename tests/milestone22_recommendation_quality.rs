mod common;

use common::run_stdout;
use serde_json::Value;

#[test]
fn milestone22_tests_for_excludes_support_paths_by_default() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn phase22_token() -> &'static str {\n    \"phase22token\"\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/milestone22_target.rs",
        "#[test]\nfn phase22_test_target() {\n    let _ = \"phase22token\";\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/common/mod.rs",
        "pub fn shared() -> &'static str {\n    \"phase22token phase22token\"\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/helpers/support.rs",
        "pub const SUPPORT: &str = \"phase22token\";\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "phase22token",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("tests-for json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["target"] == "tests/milestone22_target.rs")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["target"] == "tests/common/mod.rs")
    );
    assert!(
        !results
            .iter()
            .any(|row| row["target"] == "tests/helpers/support.rs")
    );
}

#[test]
fn milestone22_tests_for_prefers_runnable_targets() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn phase22_rank_token() -> &'static str {\n    \"phase22rank\"\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/milestone22_preferred.rs",
        "#[test]\nfn runnable_target() {\n    let _ = \"phase22rank\";\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/common/mod.rs",
        "pub const A: &str = \"phase22rank\";\npub const B: &str = \"phase22rank\";\npub const C: &str = \"phase22rank\";\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "phase22rank",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-support",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("tests-for json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    let first = results.first().expect("expected at least one result");
    assert_eq!(first["target"], "tests/milestone22_preferred.rs");
    assert_eq!(first["target_kind"], "integration_test_file");
    assert!(
        results
            .iter()
            .any(|row| row["target"] == "tests/common/mod.rs")
    );
}

#[test]
fn milestone22_tests_for_include_support_restores_paths() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn phase22_support_token() -> &'static str {\n    \"phase22support\"\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/milestone22_default.rs",
        "#[test]\nfn runnable_target() {\n    let _ = \"phase22support\";\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/common/mod.rs",
        "pub const SHARED: &str = \"phase22support\";\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let default_out = run_stdout(&[
        "tests-for",
        "phase22support",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let default_payload: Value =
        serde_json::from_str(&default_out).expect("default tests-for json should parse");
    let default_results = default_payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        !default_results
            .iter()
            .any(|row| row["target"] == "tests/common/mod.rs")
    );

    let include_support_out = run_stdout(&[
        "tests-for",
        "phase22support",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-support",
        "--json",
    ]);
    let include_support_payload: Value = serde_json::from_str(&include_support_out)
        .expect("include-support tests-for json should parse");
    let include_support_results = include_support_payload["results"]
        .as_array()
        .expect("results should be array");

    let support_row = include_support_results
        .iter()
        .find(|row| row["target"] == "tests/common/mod.rs")
        .expect("support path should be restored when include-support is set");
    assert_eq!(support_row["target_kind"], "support_test_file");
    assert!(
        support_row["why_included"]
            .as_str()
            .expect("why_included should be string")
            .contains("support path")
    );
}
