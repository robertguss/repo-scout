mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn public_api() -> i32 { helper() }
fn helper() -> i32 { 41 }
fn no_tests() -> i32 { 2 }
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn api_works() { assert_eq!(41, crate::lib::public_api()); }
"#,
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone116_precision_contracts_hold_for_refactor_diagnostics() {
    let repo = setup_repo();
    let repo_path = repo.path().to_str().expect("repo path utf-8");

    let gaps: Value = serde_json::from_str(&common::run_stdout(&[
        "test-gaps",
        "src/lib.rs",
        "--repo",
        repo_path,
        "--json",
    ]))
    .expect("test-gaps json");
    assert!(
        gaps["analysis_state"].is_string()
            || gaps["data"]["analysis_state"].is_string()
            || gaps["report"]["analysis_state"].is_string(),
        "analysis_state should be present in test-gaps json"
    );

    let boundary: Value = serde_json::from_str(&common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo_path,
        "--json",
        "--public-only",
    ]))
    .expect("boundary json");
    assert!(
        boundary["internal_symbols"]
            .as_array()
            .or_else(|| boundary["report"]["internal_symbols"].as_array())
            .expect("internal symbols array")
            .is_empty(),
        "public-only boundary should exclude internal symbols"
    );

    let coupling: Value = serde_json::from_str(&common::run_stdout(&[
        "coupling", "--repo", repo_path, "--json",
    ]))
    .expect("coupling json");
    assert_eq!(coupling["command"], "coupling");

    let rename: Value = serde_json::from_str(&common::run_stdout(&[
        "rename-check",
        "public_api",
        "--to",
        "public_api_v2",
        "--repo",
        repo_path,
        "--json",
    ]))
    .expect("rename-check json");

    assert!(rename["semantic_impacts"].is_object());
    assert!(rename["lexical_impacts"].is_object());
}
