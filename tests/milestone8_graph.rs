mod common;

use rusqlite::Connection;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone8_symbol_upsert_stable_ids() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );
    common::write_file(repo.path(), "src/extra.rs", "pub fn extra_symbol() {}\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let db_path = repo.path().join(".repo-scout").join("index.db");
    let first_id = symbol_id_for(&db_path, "start_engine");

    common::write_file(
        repo.path(),
        "src/lib.rs",
        &("pub fn aaa_new_symbol() {}\n".to_owned()
            + include_str!("fixtures/phase2/rust_symbols/src/lib.rs")
            + "\n// trigger reindex and reorder insertion keys\n"),
    );
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let second_id = symbol_id_for(&db_path, "start_engine");

    assert_eq!(
        first_id, second_id,
        "symbol_id should remain stable for the same symbol identity across reindex"
    );
}

fn symbol_id_for(db_path: &std::path::Path, symbol: &str) -> i64 {
    let connection = Connection::open(db_path).expect("index db should open");
    connection
        .query_row(
            "SELECT symbol_id
             FROM symbols_v2
             WHERE symbol = ?1
             ORDER BY file_path ASC, start_line ASC, start_column ASC
             LIMIT 1",
            [symbol],
            |row| row.get(0),
        )
        .expect("symbol should exist")
}

#[test]
fn milestone8_call_and_contains_edges() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let db_path = repo.path().join(".repo-scout").join("index.db");
    let edges = read_edges(&db_path);

    assert!(
        edges.contains(&(
            "run".to_string(),
            "start_engine".to_string(),
            "calls".to_string()
        )),
        "expected call edge run -> start_engine"
    );
    assert!(
        edges.contains(&(
            "Launcher".to_string(),
            "run".to_string(),
            "contains".to_string()
        )),
        "expected containment edge Launcher -> run"
    );
}

#[test]
fn milestone8_imports_and_implements_edges() {
    let repo = common::temp_repo();
    let source = format!(
        "{}\nuse launch::Launcher as LocalLauncher;\n",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs")
    );
    common::write_file(repo.path(), "src/lib.rs", &source);

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let db_path = repo.path().join(".repo-scout").join("index.db");
    let edges = read_edges(&db_path);

    assert!(
        edges.contains(&(
            "LocalLauncher".to_string(),
            "Launcher".to_string(),
            "imports".to_string()
        )),
        "expected imports edge LocalLauncher -> Launcher"
    );
    assert!(
        edges.contains(&(
            "Launcher".to_string(),
            "Runnable".to_string(),
            "implements".to_string()
        )),
        "expected implements edge Launcher -> Runnable"
    );
}

fn read_edges(db_path: &std::path::Path) -> Vec<(String, String, String)> {
    let connection = Connection::open(db_path).expect("index db should open");
    let mut statement = connection
        .prepare(
            "SELECT fs.symbol, ts.symbol, e.edge_kind
             FROM symbol_edges_v2 e
             JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
             JOIN symbols_v2 ts ON ts.symbol_id = e.to_symbol_id
             ORDER BY fs.symbol ASC, ts.symbol ASC, e.edge_kind ASC",
        )
        .expect("edge query should prepare");
    let rows = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("edge query should execute");
    rows.map(|row| row.expect("edge row should deserialize"))
        .collect()
}
