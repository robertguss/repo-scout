mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/a.rs",
        "pub fn a() -> i32 { crate::b::b() }\n",
    );
    common::write_file(
        repo.path(),
        "src/b.rs",
        "pub fn b() -> i32 { crate::a::a() }\n",
    );
    common::write_file(
        repo.path(),
        "tests/a_test.rs",
        "#[test] fn t() { assert_eq!(0, crate::a::a()); }\n",
    );
    common::write_file(
        repo.path(),
        "tests/fixtures/fix.rs",
        "pub fn fx() -> i32 { crate::a::a() }\n",
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone107_coupling_filters_tests_by_default_and_has_opt_in_flags() {
    let repo = setup_repo();

    let default_out = common::run_stdout(&[
        "coupling",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let default_json: Value = serde_json::from_str(&default_out).expect("coupling json");
    let default_rows = default_json["results"].as_array().expect("rows");
    assert!(
        default_rows.iter().all(|row| {
            !row["file_a"]
                .as_str()
                .unwrap_or_default()
                .starts_with("tests/")
                && !row["file_b"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("tests/")
        }),
        "default should suppress tests: {default_out}"
    );

    let include_tests_out = common::run_stdout(&[
        "coupling",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--include-tests",
        "--json",
    ]);
    let include_tests_json: Value =
        serde_json::from_str(&include_tests_out).expect("coupling json");
    let include_rows = include_tests_json["results"].as_array().expect("rows");
    assert!(
        include_rows.len() >= default_rows.len(),
        "--include-tests should be at least as broad as default\n\
         default={default_out}\ninclude-tests={include_tests_out}"
    );
}
