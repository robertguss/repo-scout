mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn run() -> i32 { helper() }\nfn helper() -> i32 { 1 }\n",
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        "#[test] fn t() { assert_eq!(1, crate::lib::helper()); }\n",
    );
    common::write_file(
        repo.path(),
        "tests/fixtures/lib_fixture.rs",
        "pub fn fixture() -> &'static str { \"helper\" }\n",
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone108_rename_check_reports_semantic_and_lexical_impacts() {
    let repo = setup_repo();

    let out = common::run_stdout(&[
        "rename-check",
        "helper",
        "--to",
        "helper_v2",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("rename-check json");
    assert!(json["semantic_impacts"].is_object(), "output: {out}");
    assert!(json["lexical_impacts"].is_object(), "output: {out}");

    let include_tests_out = common::run_stdout(&[
        "rename-check",
        "helper",
        "--to",
        "helper_v2",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--include-tests",
        "--json",
    ]);
    let include_tests_json: Value =
        serde_json::from_str(&include_tests_out).expect("rename-check json");

    assert!(
        include_tests_json["semantic_impacts"]["reported"]
            .as_u64()
            .unwrap_or_default()
            >= json["semantic_impacts"]["reported"]
                .as_u64()
                .unwrap_or_default(),
        "--include-tests should not reduce semantic impacts\ndefault: {out}\ninclude-tests: {include_tests_out}"
    );
}
