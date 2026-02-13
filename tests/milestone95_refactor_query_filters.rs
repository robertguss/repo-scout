mod common;

use common::run_stdout;
use serde_json::Value;

fn write_fixture(repo: &std::path::Path) {
    common::write_file(repo, "src/main.rs", "fn run() {}\nfn helper() {}\n");
    common::write_file(repo, "src/worker.py", "def run():\n    pass\n");
    common::write_file(repo, "tests/unit_test.rs", "fn run() {}\n");
    common::write_file(repo, "tests/fixtures/sample.rs", "fn run() {}\n");
}

#[test]
fn milestone95_find_scope_and_exclude_glob_filters_noise() {
    let repo = common::temp_repo();
    write_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "find",
        "run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--scope",
        "production",
        "--exclude-glob",
        "tests/fixtures/**",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("find json should parse");
    let results = payload["results"].as_array().expect("results should be array");

    assert!(
        results
            .iter()
            .all(|row| row["file_path"] == "src/main.rs" || row["file_path"] == "src/worker.py"),
        "production scope + fixture glob should keep only src paths"
    );
}

#[test]
fn milestone95_find_lang_and_file_filters_and_qualified_preference() {
    let repo = common::temp_repo();
    write_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let lang_out = run_stdout(&[
        "find",
        "run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--lang",
        "python",
        "--json",
    ]);
    let lang_payload: Value = serde_json::from_str(&lang_out).expect("find json should parse");
    let lang_results = lang_payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        !lang_results.is_empty()
            && lang_results
                .iter()
                .all(|row| row["file_path"] == "src/worker.py"),
        "--lang python should keep python rows only"
    );

    let file_out = run_stdout(&[
        "find",
        "run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--file",
        "src/main.rs",
        "--json",
    ]);
    let file_payload: Value = serde_json::from_str(&file_out).expect("find json should parse");
    let file_results = file_payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        !file_results.is_empty() && file_results.iter().all(|row| row["file_path"] == "src/main.rs"),
        "--file src/main.rs should keep only that file"
    );

    let qualified_out = run_stdout(&[
        "find",
        "rust:src/main.rs::run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let qualified_payload: Value =
        serde_json::from_str(&qualified_out).expect("find json should parse");
    let qualified_results = qualified_payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(
        qualified_results.first().and_then(|row| row["file_path"].as_str()),
        Some("src/main.rs"),
        "qualified query should prioritize matching file"
    );
}

#[test]
fn milestone95_find_penalizes_fixture_paths_unless_opted_in() {
    let repo = common::temp_repo();
    write_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let default_out = run_stdout(&[
        "find",
        "run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let default_payload: Value =
        serde_json::from_str(&default_out).expect("default find json should parse");
    let default_results = default_payload["results"]
        .as_array()
        .expect("results should be array");
    let fixture_default = default_results
        .iter()
        .find(|row| row["file_path"] == "tests/fixtures/sample.rs")
        .expect("fixture path should be present by default");

    let include_out = run_stdout(&[
        "find",
        "run",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-fixtures",
        "--json",
    ]);
    let include_payload: Value =
        serde_json::from_str(&include_out).expect("include-fixtures find json should parse");
    let include_results = include_payload["results"]
        .as_array()
        .expect("results should be array");
    let fixture_included = include_results
        .iter()
        .find(|row| row["file_path"] == "tests/fixtures/sample.rs")
        .expect("fixture path should remain present when include-fixtures is set");

    let default_score = fixture_default["score"]
        .as_f64()
        .expect("fixture default score should be numeric");
    let include_score = fixture_included["score"]
        .as_f64()
        .expect("fixture include score should be numeric");

    assert!(
        default_score < include_score,
        "fixture rows should be penalized by default and restored by --include-fixtures"
    );
}
