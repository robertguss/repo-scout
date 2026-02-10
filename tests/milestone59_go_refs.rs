mod common;

use common::run_stdout;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

fn write_go_refs_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/app/main.go",
        include_str!("fixtures/phase12/go_refs/src/app/main.go"),
    );
    common::write_file(
        repo,
        "src/util/util.go",
        include_str!("fixtures/phase12/go_refs/src/util/util.go"),
    );
    common::write_file(
        repo,
        "src/other/other.go",
        include_str!("fixtures/phase12/go_refs/src/other/other.go"),
    );
}

fn run_json(repo: &Path, command: &[&str]) -> Value {
    let repo_arg = repo.to_str().expect("repo path should be utf-8");
    let mut args = command.to_vec();
    args.push("--repo");
    args.push(repo_arg);
    args.push("--json");
    let output = run_stdout(&args);
    serde_json::from_str(&output).expect("json output should parse")
}

fn has_diff_impact_called_by_row(payload: &Value, symbol: &str, file_path: &str) -> bool {
    payload["results"]
        .as_array()
        .expect("results should be an array")
        .iter()
        .any(|item| {
            item["result_kind"] == "impacted_symbol"
                && item["relationship"] == "called_by"
                && item["symbol"] == symbol
                && item["file_path"] == file_path
        })
}

fn has_impact_called_by_row(payload: &Value, symbol: &str, file_path: &str) -> bool {
    payload["results"]
        .as_array()
        .expect("results should be an array")
        .iter()
        .any(|item| {
            item["relationship"] == "called_by"
                && item["symbol"] == symbol
                && item["file_path"] == file_path
        })
}

#[test]
fn milestone59_go_refs_are_ast_backed_for_selector_calls() {
    let repo = common::temp_repo();
    write_go_refs_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let refs_payload = run_json(repo.path(), &["refs", "Helper"]);
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("refs results should be an array");
    assert!(
        refs_results.iter().any(|item| {
            item["file_path"] == "src/app/main.go"
                && item["symbol"] == "Helper"
                && item["why_matched"] == "ast_reference"
                && item["confidence"] == "ast_likely"
        }),
        "expected selector call helper reference in src/app/main.go to be AST-backed"
    );
}

#[test]
fn milestone59_go_diff_impact_prefers_import_alias_target_for_duplicate_functions() {
    let repo = common::temp_repo();
    write_go_refs_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = run_json(
        repo.path(),
        &["diff-impact", "--changed-file", "src/util/util.go"],
    );

    assert!(
        has_diff_impact_called_by_row(&payload, "Run", "src/app/main.go"),
        "expected utilpkg.Helper() call to map to src/util/util.go::Helper despite duplicate Helper symbol in src/other/other.go"
    );
}

#[test]
fn milestone59_go_impact_and_refs_json_are_deterministic() {
    let repo = common::temp_repo();
    write_go_refs_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let refs_a = run_json(repo.path(), &["refs", "Helper"]);
    let refs_b = run_json(repo.path(), &["refs", "Helper"]);
    assert_eq!(refs_a, refs_b);

    let impact_a = run_json(repo.path(), &["impact", "SayHello"]);
    let impact_b = run_json(repo.path(), &["impact", "SayHello"]);
    assert_eq!(impact_a, impact_b);
    assert!(
        has_impact_called_by_row(&impact_a, "Run", "src/app/main.go"),
        "expected impact SayHello to include Run caller from src/app/main.go"
    );
}

#[test]
fn milestone59_go_type_kinds_capture_interface_and_alias_definitions() {
    let repo = common::temp_repo();
    write_go_refs_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("index db should open");
    let speaker_kind: String = connection
        .query_row(
            "SELECT kind
             FROM symbols_v2
             WHERE file_path = 'src/util/util.go' AND symbol = 'Speaker'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("Speaker symbol row should exist");
    assert_eq!(speaker_kind, "interface");

    let alias_kind: String = connection
        .query_row(
            "SELECT kind
             FROM symbols_v2
             WHERE file_path = 'src/util/util.go' AND symbol = 'SpeakerAlias'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .expect("SpeakerAlias symbol row should exist");
    assert_eq!(alias_kind, "type_alias");
}
