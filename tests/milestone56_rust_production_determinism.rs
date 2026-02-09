mod common;

use common::run_stdout;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

fn write_rust_production_corpus(repo: &Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase11/rust_production/corpus/src/lib.rs"),
    );
    common::write_file(
        repo,
        "src/util/mod.rs",
        include_str!("fixtures/phase11/rust_production/corpus/src/util/mod.rs"),
    );
    common::write_file(
        repo,
        "src/support.rs",
        include_str!("fixtures/phase11/rust_production/corpus/src/support.rs"),
    );
    common::write_file(
        repo,
        "src/nested/mod.rs",
        include_str!("fixtures/phase11/rust_production/corpus/src/nested/mod.rs"),
    );
    common::write_file(
        repo,
        "src/nested/child.rs",
        include_str!("fixtures/phase11/rust_production/corpus/src/nested/child.rs"),
    );
    common::write_file(
        repo,
        "tests/rust_production.rs",
        include_str!("fixtures/phase11/rust_production/corpus/tests/rust_production.rs"),
    );
}

fn run_json(repo_arg: &str, command: &[&str]) -> String {
    let mut args = command.to_vec();
    args.push("--repo");
    args.push(repo_arg);
    args.push("--json");
    run_stdout(&args)
}

fn call_edge_count(db_path: &Path) -> i64 {
    let connection = Connection::open(db_path).expect("index db should open");
    connection
        .query_row(
            "SELECT COUNT(*) FROM symbol_edges_v2 WHERE edge_kind = 'calls'",
            [],
            |row| row.get(0),
        )
        .expect("call edge count query should succeed")
}

#[test]
fn milestone56_rust_json_outputs_are_repeatable_across_runs() {
    let repo = common::temp_repo();
    write_rust_production_corpus(repo.path());

    let repo_arg = repo.path().to_str().expect("repo path should be utf-8");
    run_stdout(&["index", "--repo", repo_arg]);

    for command in [
        vec!["find", "helper"],
        vec!["refs", "helper"],
        vec!["impact", "helper"],
        vec!["diff-impact", "--changed-file", "src/util/mod.rs"],
    ] {
        let first = run_json(repo_arg, &command);
        let second = run_json(repo_arg, &command);
        assert_eq!(
            first, second,
            "expected deterministic json output for command {:?}",
            command
        );

        let payload: Value = serde_json::from_str(&first).expect("json output should parse");
        assert!(
            payload["results"].is_array(),
            "json output should contain results array for command {:?}",
            command
        );
    }
}

#[test]
fn milestone56_rust_module_resolution_does_not_duplicate_edges_unboundedly() {
    let repo = common::temp_repo();
    write_rust_production_corpus(repo.path());

    let repo_arg = repo.path().to_str().expect("repo path should be utf-8");
    run_stdout(&["index", "--repo", repo_arg]);
    let db_path = repo.path().join(".repo-scout").join("index.db");
    let first_call_edge_count = call_edge_count(&db_path);

    run_stdout(&["index", "--repo", repo_arg]);
    let second_call_edge_count = call_edge_count(&db_path);

    assert_eq!(
        first_call_edge_count, second_call_edge_count,
        "call edge count should remain stable after repeat indexing"
    );
    assert!(
        second_call_edge_count <= 64,
        "call edge growth must remain bounded for phase11 rust corpus"
    );
}
