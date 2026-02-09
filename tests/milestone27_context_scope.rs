mod common;

use std::path::Path;

use common::run_stdout;
use rusqlite::Connection;
use serde_json::Value;

const CONTEXT_SCOPE_SYMBOL: &str = concat!(
    "update_verify_plan_recommendation_quality_for_changed_files_",
    "and_reduce_noisy_test_selection"
);
const CONTEXT_SCOPE_TASK: &str = concat!(
    "update verify plan recommendation quality for changed files ",
    "and reduce noisy test selection"
);
const CONTEXT_SCOPE_TEST_SOURCE: &str = concat!(
    "pub fn update_verify_plan_recommendation_quality_for_changed_files_",
    "and_reduce_noisy_test_selection() {}\n"
);

fn is_test_like_path(path: &str) -> bool {
    path.starts_with("tests/") || path.contains("/tests/") || path.ends_with("_test.rs")
}

fn is_code_path(path: &str) -> bool {
    path.ends_with(".rs")
        || path.ends_with(".ts")
        || path.ends_with(".tsx")
        || path.ends_with(".py")
}

fn insert_symbol(
    db_path: &Path,
    file_path: &str,
    symbol: &str,
    kind: &str,
    language: &str,
    start_line: u32,
    end_line: u32,
) {
    let connection = Connection::open(db_path).expect("index db should open");
    connection
        .execute(
            "INSERT INTO symbols_v2(
                file_path, symbol, kind, language, qualified_symbol, container,
                start_line, start_column, end_line, end_column, signature
             ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, 1, ?7, 1, NULL)",
            rusqlite::params![
                file_path,
                symbol,
                kind,
                language,
                format!("{language}:{file_path}::{symbol}"),
                start_line,
                end_line
            ],
        )
        .expect("symbol insert should succeed");
}

fn context_results<'a>(payload: &'a Value, label: &str) -> &'a [Value] {
    payload[label].as_array().expect("results should be array")
}

fn run_context_json(repo_root: &Path, extra_flags: &[&str]) -> Value {
    let repo_arg = repo_root.to_str().expect("repo path should be utf-8");
    let mut args = vec![
        "context",
        "--task",
        CONTEXT_SCOPE_TASK,
        "--repo",
        repo_arg,
        "--budget",
        "1200",
    ];
    args.extend_from_slice(extra_flags);
    args.push("--json");
    let out = run_stdout(&args);
    serde_json::from_str(&out).expect("context json should parse")
}

fn write_scope_fixture(repo_root: &Path) {
    common::write_file(
        repo_root,
        "src/lib.rs",
        "pub fn verify_plan_for_changed_files() {}\n",
    );
    common::write_file(
        repo_root,
        "tests/context_scope_test.rs",
        CONTEXT_SCOPE_TEST_SOURCE,
    );
}

fn assert_scope_results_are_code_only(results: &[Value]) {
    assert!(results.iter().all(|row| {
        row["file_path"]
            .as_str()
            .is_some_and(is_code_path)
    }));
}

#[test]
fn milestone27_context_exclude_tests_omits_test_paths() {
    let repo = common::temp_repo();
    write_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline = run_context_json(repo.path(), &[]);
    let baseline_results = context_results(&baseline, "results");
    assert!(
        baseline_results
            .iter()
            .any(|row| { row["file_path"].as_str().is_some_and(is_test_like_path) })
    );

    let scoped = run_context_json(repo.path(), &["--exclude-tests"]);
    let scoped_results = context_results(&scoped, "results");

    assert!(
        scoped_results
            .iter()
            .all(|row| !row["file_path"].as_str().is_some_and(is_test_like_path))
    );
    assert!(
        scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/lib.rs")
    );
}

#[test]
fn milestone27_context_code_only_restricts_to_code_extensions() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn verify_plan_for_changed_files() {}\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    insert_symbol(
        &db_path,
        "docs/context_notes.md",
        CONTEXT_SCOPE_SYMBOL,
        "function",
        "unknown",
        1,
        1,
    );

    let baseline = run_context_json(repo.path(), &[]);
    let baseline_results = context_results(&baseline, "results");
    assert!(
        baseline_results
            .iter()
            .any(|row| row["file_path"] == "docs/context_notes.md")
    );

    let scoped = run_context_json(repo.path(), &["--code-only"]);
    let scoped_results = context_results(&scoped, "results");

    assert_scope_results_are_code_only(scoped_results);
    assert!(
        scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/lib.rs")
    );
}

#[test]
fn milestone27_context_scope_flags_preserve_deterministic_json() {
    let repo = common::temp_repo();
    write_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    insert_symbol(
        &db_path,
        "docs/context_notes.md",
        CONTEXT_SCOPE_SYMBOL,
        "function",
        "unknown",
        1,
        1,
    );
    insert_symbol(
        &db_path,
        "src/lib.rs",
        "verify_plan_for_changed_files",
        "type_alias",
        "rust",
        1,
        1,
    );
    let connection = Connection::open(&db_path).expect("index db should open");
    let duplicate_rows: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM symbols_v2
             WHERE file_path = 'src/lib.rs'
               AND symbol = 'verify_plan_for_changed_files'",
            [],
            |row| row.get(0),
        )
        .expect("symbol rows should be queryable");
    assert_eq!(duplicate_rows, 2);

    let scope_flags = ["--code-only", "--exclude-tests"];
    let first = run_context_json(repo.path(), &scope_flags).to_string();
    let second = run_context_json(repo.path(), &scope_flags).to_string();

    assert_eq!(first, second);

    let payload: Value = serde_json::from_str(&first).expect("context json should parse");
    let results = context_results(&payload, "results");
    assert_scope_results_are_code_only(results);
    assert!(
        results
            .iter()
            .all(|row| !row["file_path"].as_str().is_some_and(is_test_like_path))
    );
    assert!(results.iter().any(|row| row["file_path"] == "src/lib.rs"));

    let scoped_kinds = results
        .iter()
        .filter(|row| {
            row["file_path"] == "src/lib.rs" && row["symbol"] == "verify_plan_for_changed_files"
        })
        .map(|row| row["kind"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        scoped_kinds,
        vec!["function".to_string(), "type_alias".to_string()]
    );
}
