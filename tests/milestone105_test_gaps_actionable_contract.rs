mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn covered_fn() -> i32 {
    1
}

pub fn uncovered_fn() -> i32 {
    2
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn test_covered_fn() {
    assert_eq!(1, crate::lib::covered_fn());
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
fn milestone105_test_gaps_has_explicit_status_fields() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "test-gaps",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("test-gaps json");

    assert!(
        json["report"]["analysis_state"].is_string(),
        "output: {out}"
    );

    let covered = json["report"]["covered"].as_array().expect("covered array");
    if let Some(entry) = covered.first() {
        assert_eq!(entry["coverage_status"], "covered", "output: {out}");
    }

    let uncovered = json["report"]["uncovered"]
        .as_array()
        .expect("uncovered array");
    if let Some(entry) = uncovered.first() {
        assert_eq!(entry["coverage_status"], "uncovered", "output: {out}");
    }
}
