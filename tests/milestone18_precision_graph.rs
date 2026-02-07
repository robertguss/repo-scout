mod common;

use rusqlite::Connection;
use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn write_disambiguated_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/a.rs",
        include_str!("fixtures/phase4/ambiguity/disambiguated/src/a.rs"),
    );
    common::write_file(
        repo,
        "src/b.rs",
        include_str!("fixtures/phase4/ambiguity/disambiguated/src/b.rs"),
    );
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase4/ambiguity/disambiguated/src/lib.rs"),
    );
}

fn write_ambiguous_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/a.rs",
        include_str!("fixtures/phase4/ambiguity/ambiguous/src/a.rs"),
    );
    common::write_file(
        repo,
        "src/b.rs",
        include_str!("fixtures/phase4/ambiguity/ambiguous/src/b.rs"),
    );
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase4/ambiguity/ambiguous/src/lib.rs"),
    );
}

fn read_call_edges(db_path: &std::path::Path) -> Vec<(String, String, String, String, String)> {
    let connection = Connection::open(db_path).expect("index db should open");
    let mut statement = connection
        .prepare(
            "SELECT fs.file_path, fs.symbol, ts.file_path, ts.symbol, e.edge_kind
             FROM symbol_edges_v2 e
             JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
             JOIN symbols_v2 ts ON ts.symbol_id = e.to_symbol_id
             ORDER BY fs.file_path ASC, fs.symbol ASC, ts.file_path ASC, ts.symbol ASC",
        )
        .expect("edge query should prepare");
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .expect("edge query should execute");

    rows.map(|row| row.expect("edge row should deserialize"))
        .collect()
}

#[test]
fn milestone18_disambiguates_duplicate_rust_call_targets() {
    let repo = common::temp_repo();
    write_disambiguated_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let edges = read_call_edges(&db_path);

    let mut run_targets = edges
        .iter()
        .filter(|(from_file, from_symbol, _to_file, to_symbol, edge_kind)| {
            from_file == "src/lib.rs"
                && from_symbol == "entry"
                && to_symbol == "run"
                && edge_kind == "calls"
        })
        .map(|(_from_file, _from_symbol, to_file, _to_symbol, _edge_kind)| to_file.clone())
        .collect::<Vec<_>>();
    run_targets.sort();
    run_targets.dedup();

    assert_eq!(
        run_targets,
        vec!["src/a.rs".to_string(), "src/b.rs".to_string()],
        "entry should call both disambiguated run targets"
    );
}

#[test]
fn milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target() {
    let repo = common::temp_repo();
    write_disambiguated_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/b.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "entry"
            && item["file_path"] == "src/lib.rs"
            && item["relationship"] == "called_by"
            && item["distance"] == 1
    }));
}

#[test]
fn milestone18_ambiguous_unqualified_call_does_not_cross_link() {
    let repo = common::temp_repo();
    write_ambiguous_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let edges = read_call_edges(&db_path);

    let ambiguous_run_targets = edges
        .iter()
        .filter(|(from_file, from_symbol, _to_file, to_symbol, edge_kind)| {
            from_file == "src/lib.rs"
                && from_symbol == "entry"
                && to_symbol == "run"
                && edge_kind == "calls"
        })
        .count();

    assert_eq!(
        ambiguous_run_targets, 0,
        "ambiguous unqualified call should not resolve to arbitrary duplicate symbol"
    );
}
