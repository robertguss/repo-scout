mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/a.rs",
        r#"
pub fn run() -> i32 {
    crate::b::helper()
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/b.rs",
        r#"
pub fn helper() -> i32 {
    crate::a::run()
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/a_test.rs",
        r#"
#[test]
fn test_run() {
    assert_eq!(42, crate::a::run());
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/fixtures/helper_fixture.rs",
        r#"
pub fn fixture_call() -> i32 {
    crate::b::helper()
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
fn milestone102_coupling_default_is_production_first() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "coupling",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("coupling json");
    let rows = json["results"].as_array().expect("results array");
    assert!(
        rows.iter().all(|row| {
            !row["file_a"]
                .as_str()
                .unwrap_or_default()
                .starts_with("tests/")
                && !row["file_b"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("tests/")
        }),
        "default coupling should suppress tests/fixtures: {out}"
    );
}
