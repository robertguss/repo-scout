mod common;

use serde_json::Value;

fn setup_refactor_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn api() -> i32 {
    helper()
}

fn helper() -> i32 {
    41 + 1
}

fn dead_fn() -> i32 {
    5
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/other.rs",
        r#"
pub fn call_api() -> i32 {
    crate::lib::api()
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn api_works() {
    assert_eq!(42, crate::lib::api());
}
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
fn milestone97_anatomy_command_reports_file_symbols() {
    let repo = setup_refactor_repo();
    let out = common::run_stdout(&[
        "anatomy",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        out.contains("Anatomy of src/lib.rs"),
        "missing anatomy header:\n{out}"
    );
    assert!(out.contains("api"), "missing known symbol:\n{out}");
}

#[test]
fn milestone97_coupling_command_json_is_deterministic() {
    let repo = setup_refactor_repo();
    let args = [
        "coupling",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ];
    let out1 = common::run_stdout(&args);
    let out2 = common::run_stdout(&args);
    assert_eq!(out1, out2, "coupling json should be deterministic");
    let json: Value = serde_json::from_str(&out1).expect("coupling output should be json");
    assert_eq!(json["command"], "coupling");
}

#[test]
fn milestone97_dead_command_finds_unreferenced_symbols() {
    let repo = setup_refactor_repo();
    let out = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        out.contains("dead_fn"),
        "expected dead symbol dead_fn in output:\n{out}"
    );
}

#[test]
fn milestone97_test_gaps_reports_coverage_summary() {
    let repo = setup_refactor_repo();
    let out = common::run_stdout(&[
        "test-gaps",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        out.contains("Test gap analysis for src/lib.rs"),
        "missing test-gap header:\n{out}"
    );
    assert!(out.contains("SUMMARY"), "missing summary section:\n{out}");
}
