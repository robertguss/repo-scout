mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn api() -> i32 { private_dead() }\nfn private_dead() -> i32 { 1 }\n",
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        "#[test] fn test_api() { assert_eq!(1, crate::lib::api()); }\n",
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone109_refactor_quality_command_contracts_hold() {
    let repo = setup_repo();

    let dead = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--scope",
        "production",
        "--json",
    ]);
    let dead_json: Value = serde_json::from_str(&dead).expect("dead json");
    assert!(dead_json["results"].is_array(), "output: {dead}");

    let boundary = common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--public-only",
        "--json",
    ]);
    let boundary_json: Value = serde_json::from_str(&boundary).expect("boundary json");
    assert!(
        boundary_json["report"]["internal_symbols"]
            .as_array()
            .expect("internal symbols")
            .is_empty(),
        "output: {boundary}"
    );

    let gaps = common::run_stdout(&[
        "test-gaps",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let gaps_json: Value = serde_json::from_str(&gaps).expect("test-gaps json");
    assert!(
        gaps_json["report"]["analysis_state"].is_string(),
        "output: {gaps}"
    );

    let coupling = common::run_stdout(&[
        "coupling",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let coupling_json: Value = serde_json::from_str(&coupling).expect("coupling json");
    assert!(coupling_json["results"].is_array(), "output: {coupling}");

    let rename = common::run_stdout(&[
        "rename-check",
        "api",
        "--to",
        "run",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let rename_json: Value = serde_json::from_str(&rename).expect("rename json");
    assert!(
        rename_json["semantic_impacts"].is_object(),
        "output: {rename}"
    );
    assert!(
        rename_json["lexical_impacts"].is_object(),
        "output: {rename}"
    );
}
