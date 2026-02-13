mod common;

use serde_json::Value;

fn setup_intelligence_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn public_api() -> i32 {
    internal_helper() + 1
}

fn internal_helper() -> i32 {
    41
}

pub fn long_public_entry() -> i32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    let f = 6;
    let g = 7;
    let h = 8;
    let i = 9;
    let j = 10;
    a + b + c + d + e + f + g + h + i + j
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn public_api_works() {
    assert_eq!(42, crate::lib::public_api());
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
fn milestone98_boundary_reports_public_surface() {
    let repo = setup_intelligence_repo();
    let out = common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        out.contains("Boundary analysis for src/lib.rs"),
        "missing boundary header:\n{out}"
    );
    assert!(
        out.contains("public_api"),
        "expected public_api in boundary report:\n{out}"
    );
}

#[test]
fn milestone98_suggest_json_returns_ranked_candidates() {
    let repo = setup_intelligence_repo();
    let out = common::run_stdout(&[
        "suggest",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("suggest output should be json");
    assert_eq!(json["command"], "suggest");
    assert!(
        json["results"].is_array(),
        "suggest json should include results array:\n{out}"
    );
}
