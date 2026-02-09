mod common;

use common::run_stdout;
use rusqlite::Connection;
use serde_json::Value;

fn write_go_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/main.go",
        include_str!("fixtures/phase10/go_find/src/main.go"),
    );
    common::write_file(
        repo,
        "src/app.ts",
        include_str!("fixtures/phase10/go_find/src/app.ts"),
    );
}

#[test]
fn milestone50_go_find_definitions_are_ast_backed() {
    let repo = common::temp_repo();
    write_go_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "find",
        "SayHello",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("find json should parse");
    assert_eq!(payload["schema_version"], 1);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty(), "expected at least one find result");
    let first = &results[0];
    assert_eq!(first["file_path"], "src/main.go");
    assert_eq!(first["symbol"], "SayHello");
    assert_eq!(first["why_matched"], "ast_definition");
    assert_eq!(first["confidence"], "ast_exact");
}

#[test]
fn milestone50_go_find_persists_language_metadata() {
    let repo = common::temp_repo();
    write_go_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("index db should open");
    let language: String = connection
        .query_row(
            "SELECT language
             FROM symbols_v2
             WHERE file_path = 'src/main.go' AND symbol = 'SayHello'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("go symbol row should exist");
    assert_eq!(language, "go");
}

#[test]
fn milestone50_go_find_json_is_deterministic() {
    let repo = common::temp_repo();
    write_go_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out_a = run_stdout(&[
        "find",
        "Greeter",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let out_b = run_stdout(&[
        "find",
        "Greeter",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(out_a, out_b);

    let payload: Value = serde_json::from_str(&out_a).expect("find json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty(), "expected at least one find result");
    assert_eq!(results[0]["why_matched"], "ast_definition");
}

#[test]
fn milestone50_go_find_scope_flags_do_not_regress_existing_languages() {
    let repo = common::temp_repo();
    write_go_fixture(repo.path());
    common::write_file(repo.path(), "docs/guide.md", "helperTs helperTs\n");
    common::write_file(repo.path(), "src/app.test.ts", "helperTs helperTs\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "find",
        "helperTs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("find json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty(), "expected at least one find result");
    assert_eq!(results[0]["file_path"], "src/app.ts");
    assert_eq!(results[0]["why_matched"], "ast_definition");
}
