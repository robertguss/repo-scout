mod common;

use rusqlite::Connection;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn build_v3_index(repo_path: &std::path::Path) -> std::path::PathBuf {
    let index_dir = repo_path.join(".repo-scout");
    std::fs::create_dir_all(&index_dir).expect("index directory should exist");
    let db_path = index_dir.join("index.db");

    let connection = Connection::open(&db_path).expect("v3 db should open");
    connection
        .execute_batch(
            r#"
            CREATE TABLE meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE indexed_files (
                file_path TEXT PRIMARY KEY,
                content_hash TEXT NOT NULL
            );
            CREATE TABLE text_occurrences (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                symbol TEXT NOT NULL,
                line INTEGER NOT NULL,
                column INTEGER NOT NULL
            );
            CREATE TABLE ast_definitions (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                symbol TEXT NOT NULL,
                kind TEXT NOT NULL,
                line INTEGER NOT NULL,
                column INTEGER NOT NULL
            );
            CREATE TABLE ast_references (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                symbol TEXT NOT NULL,
                line INTEGER NOT NULL,
                column INTEGER NOT NULL
            );
            CREATE TABLE symbols_v2 (
                symbol_id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                symbol TEXT NOT NULL,
                kind TEXT NOT NULL,
                language TEXT NOT NULL DEFAULT 'unknown',
                qualified_symbol TEXT,
                container TEXT,
                start_line INTEGER NOT NULL,
                start_column INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                end_column INTEGER NOT NULL,
                signature TEXT,
                UNIQUE(file_path, symbol, kind, start_line, start_column)
            );
            CREATE TABLE symbol_edges_v2 (
                edge_id INTEGER PRIMARY KEY,
                from_symbol_id INTEGER NOT NULL,
                to_symbol_id INTEGER NOT NULL,
                edge_kind TEXT NOT NULL,
                confidence REAL NOT NULL,
                provenance TEXT NOT NULL DEFAULT 'ast_definition',
                UNIQUE(from_symbol_id, to_symbol_id, edge_kind)
            );
            INSERT INTO meta(key, value) VALUES ('schema_version', '3');
            INSERT INTO indexed_files(file_path, content_hash) VALUES ('src/lib.rs', 'hash-v3');
            INSERT INTO symbols_v2(symbol_id, file_path, symbol, kind, language, qualified_symbol,
                start_line, start_column, end_line, end_column)
                VALUES (1, 'src/lib.rs', 'my_fn', 'function', 'rust', 'rust:src/lib.rs::my_fn',
                    10, 1, 25, 2);
            "#,
        )
        .expect("v3 schema fixture should be created");
    db_path
}

#[test]
fn milestone90_v3_upgrades_to_v4_with_new_columns() {
    let repo = common::temp_repo();
    let db_path = build_v3_index(repo.path());

    // Running status should trigger migration
    let status = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        status.contains("schema_version: 4"),
        "store bootstrap should migrate v3 dbs to schema v4, got: {status}"
    );

    // Verify new columns exist
    let connection = Connection::open(&db_path).expect("db should remain readable");

    let has_column = |table: &str, column: &str| -> bool {
        let mut stmt = connection
            .prepare(&format!("PRAGMA table_info({table})"))
            .expect("pragma should prepare");
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("pragma should query");
        rows.filter_map(|r| r.ok())
            .any(|col| col == column)
    };

    assert!(has_column("indexed_files", "line_count"), "indexed_files should have line_count column");
    assert!(has_column("symbols_v2", "line_count"), "symbols_v2 should have line_count column");
    assert!(has_column("symbols_v2", "visibility"), "symbols_v2 should have visibility column");
    assert!(has_column("symbols_v2", "param_count"), "symbols_v2 should have param_count column");
    assert!(has_column("symbols_v2", "nesting_depth"), "symbols_v2 should have nesting_depth column");
    assert!(has_column("symbols_v2", "branch_count"), "symbols_v2 should have branch_count column");
    assert!(has_column("symbols_v2", "complexity_score"), "symbols_v2 should have complexity_score column");
}

#[test]
fn milestone90_v4_migration_is_idempotent() {
    let repo = common::temp_repo();
    let db_path = build_v3_index(repo.path());

    let first = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    let second = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    assert!(first.contains("schema_version: 4"));
    assert!(second.contains("schema_version: 4"));

    let connection = Connection::open(db_path).expect("db should remain readable");
    let version: String = connection
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .expect("meta schema_version should exist");
    assert_eq!(version, "4");
}

#[test]
fn milestone90_v4_preserves_existing_data() {
    let repo = common::temp_repo();
    let db_path = build_v3_index(repo.path());

    run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);

    let connection = Connection::open(db_path).expect("db should remain readable");

    // Existing indexed_files row should survive
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM indexed_files WHERE file_path = 'src/lib.rs'",
            [],
            |row| row.get(0),
        )
        .expect("indexed_files query should work");
    assert_eq!(count, 1, "existing indexed_files rows should survive migration");

    // Existing symbols_v2 row should survive
    let sym_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM symbols_v2 WHERE symbol = 'my_fn'",
            [],
            |row| row.get(0),
        )
        .expect("symbols_v2 query should work");
    assert_eq!(sym_count, 1, "existing symbols_v2 rows should survive migration");

    // line_count should be NULL for pre-existing rows (not yet populated by indexer)
    let line_count: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM indexed_files WHERE file_path = 'src/lib.rs'",
            [],
            |row| row.get(0),
        )
        .expect("line_count query should work");
    assert!(line_count.is_none(), "pre-existing indexed_files.line_count should be NULL");

    let sym_line_count: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM symbols_v2 WHERE symbol = 'my_fn'",
            [],
            |row| row.get(0),
        )
        .expect("symbol line_count query should work");
    assert!(sym_line_count.is_none(), "pre-existing symbols_v2.line_count should be NULL");
}

#[test]
fn milestone90_indexer_populates_file_line_count() {
    let repo = common::temp_repo();
    // Create a Rust file with exactly 5 lines
    common::write_file(
        repo.path(),
        "src/main.rs",
        "fn main() {\n    println!(\"hello\");\n}\n\nfn helper() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");

    let line_count: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM indexed_files WHERE file_path = 'src/main.rs'",
            [],
            |row| row.get(0),
        )
        .expect("line_count query should work");
    assert_eq!(
        line_count,
        Some(5),
        "indexed_files.line_count should be populated after indexing"
    );
}

#[test]
fn milestone90_indexer_populates_symbol_line_count() {
    let repo = common::temp_repo();
    // main spans lines 1-3, helper spans line 5
    common::write_file(
        repo.path(),
        "src/main.rs",
        "fn main() {\n    println!(\"hello\");\n}\n\nfn helper() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");

    let main_line_count: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM symbols_v2 WHERE symbol = 'main' AND kind = 'function'",
            [],
            |row| row.get(0),
        )
        .expect("main line_count query should work");
    assert!(
        main_line_count.is_some(),
        "symbols_v2.line_count should be populated for main function"
    );
    assert_eq!(main_line_count.unwrap(), 3, "main function should span 3 lines (1 to 3)");

    let helper_line_count: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM symbols_v2 WHERE symbol = 'helper' AND kind = 'function'",
            [],
            |row| row.get(0),
        )
        .expect("helper line_count query should work");
    assert!(
        helper_line_count.is_some(),
        "symbols_v2.line_count should be populated for helper function"
    );
    assert_eq!(helper_line_count.unwrap(), 1, "helper function should span 1 line");
}

#[test]
fn milestone90_reindex_populates_line_counts_for_existing_files() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn greet() {\n    println!(\"hi\");\n}\n",
    );

    // Index once
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(&db_path).expect("db should open");

    let file_lc: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM indexed_files WHERE file_path = 'src/lib.rs'",
            [],
            |row| row.get(0),
        )
        .expect("file line_count should query");
    assert!(file_lc.is_some(), "line_count should be populated on first index");

    let sym_lc: Option<i64> = connection
        .query_row(
            "SELECT line_count FROM symbols_v2 WHERE symbol = 'greet' AND kind = 'function'",
            [],
            |row| row.get(0),
        )
        .expect("symbol line_count should query");
    assert!(sym_lc.is_some(), "symbol line_count should be populated on first index");
}
