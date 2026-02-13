mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn public_api() -> i32 {
    41
}

fn private_dead() -> i32 {
    7
}

fn tested_fn() -> i32 {
    10
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn test_tested_fn() {
    assert_eq!(10, crate::lib::tested_fn());
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
fn milestone101_dead_default_is_conservative_in_production_scope() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "dead",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--scope",
        "production",
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("dead json");
    let results = json["results"].as_array().expect("results array");
    assert!(
        results
            .iter()
            .any(|entry| entry["symbol"] == "private_dead"),
        "expected private_dead in output: {out}"
    );
    assert!(
        results.iter().all(|entry| entry["symbol"] != "public_api"),
        "public_api should not be a default dead candidate: {out}"
    );
}

#[test]
fn milestone101_boundary_public_only_json_hides_internal_symbols() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--public-only",
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("boundary json");
    assert!(
        json["report"]["internal_symbols"]
            .as_array()
            .expect("internal symbols array")
            .is_empty(),
        "--public-only json should suppress internal symbols: {out}"
    );
}

#[test]
fn milestone101_test_gaps_unknown_target_is_explicit() {
    let repo = setup_repo();
    let out = common::run_stdout(&[
        "test-gaps",
        "missing_symbol",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("test-gaps json");
    assert_eq!(json["report"]["analysis_state"], "unknown", "output: {out}");
}
