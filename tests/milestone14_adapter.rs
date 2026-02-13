mod common;

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

const V2_FIXTURE_SCHEMA_SQL: &str = r#"
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
        UNIQUE(from_symbol_id, to_symbol_id, edge_kind)
    );
    INSERT INTO meta(key, value) VALUES ('schema_version', '2');
    INSERT INTO indexed_files(file_path, content_hash) VALUES ('src/lib.rs', 'hash-v2');
    INSERT INTO symbols_v2(
        symbol_id, file_path, symbol, kind, container, start_line, start_column, end_line, end_column, signature
    ) VALUES (1, 'src/lib.rs', 'legacy_symbol', 'function', NULL, 1, 1, 1, 10, 'pub fn legacy_symbol()');
    INSERT INTO symbol_edges_v2(from_symbol_id, to_symbol_id, edge_kind, confidence)
        VALUES (1, 1, 'calls', 0.95);
"#;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn build_v2_index(repo_path: &Path) -> PathBuf {
    let index_dir = repo_path.join(".repo-scout");
    fs::create_dir_all(&index_dir).expect("index directory should exist");
    let db_path = index_dir.join("index.db");
    let connection = Connection::open(&db_path).expect("v2 db should open");
    connection
        .execute_batch(V2_FIXTURE_SCHEMA_SQL)
        .expect("v2 schema fixture should be created");
    db_path
}

fn table_has_column(connection: &Connection, table: &str, column: &str) -> bool {
    let query = format!("PRAGMA table_info({table})");
    let mut statement = connection
        .prepare(&query)
        .expect("table info query should prepare");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("table info should be queryable");

    for row in rows {
        if row.expect("column name row should decode") == column {
            return true;
        }
    }
    false
}

#[test]
fn milestone14_language_adapter_trait_migration() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mod_source = fs::read_to_string(root.join("src/indexer/languages/mod.rs"))
        .expect("languages adapter module should exist");
    let rust_source = fs::read_to_string(root.join("src/indexer/languages/rust.rs"))
        .expect("rust adapter module should exist");
    let indexer_source =
        fs::read_to_string(root.join("src/indexer/mod.rs")).expect("indexer module should load");

    assert!(mod_source.contains("pub trait LanguageAdapter"));
    assert!(mod_source.contains("pub struct ExtractionUnit"));
    assert!(mod_source.contains("pub struct ExtractedSymbol"));
    assert!(mod_source.contains("pub struct ExtractedEdge"));
    assert!(rust_source.contains("impl LanguageAdapter for RustLanguageAdapter"));
    assert!(rust_source.contains("fn language_id(&self) -> &'static str"));
    assert!(
        !indexer_source.contains("rust_ast::extract_rust_items"),
        "indexing should route language extraction through adapters"
    );
    assert!(indexer_source.contains("extract_with_adapter("));
}

#[test]
fn milestone14_rust_behavior_unchanged_through_adapter() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let indexer_source =
        fs::read_to_string(root.join("src/indexer/mod.rs")).expect("indexer module should load");
    assert!(indexer_source.contains("RustLanguageAdapter"));
    assert!(indexer_source.contains("extract("));

    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let find_out = run_stdout(&[
        "find",
        "start_engine",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(find_out.contains("[ast_definition ast_exact]"));

    let refs_out = run_stdout(&[
        "refs",
        "start_engine",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(refs_out.contains("[ast_reference ast_likely]"));

    let impact_out = run_stdout(&[
        "impact",
        "start_engine",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(impact_out.contains("called_by"));
}

#[test]
fn milestone14_schema_language_metadata_migration() {
    let repo = common::temp_repo();
    let db_path = build_v2_index(repo.path());

    let status_out = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    assert!(status_out.contains("schema_version: 4"));

    let connection = Connection::open(db_path).expect("db should remain readable");
    assert!(table_has_column(&connection, "symbols_v2", "language"));
    assert!(table_has_column(
        &connection,
        "symbols_v2",
        "qualified_symbol"
    ));
    assert!(table_has_column(
        &connection,
        "symbol_edges_v2",
        "provenance"
    ));

    let language_index_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_symbols_v2_language_symbol'",
            [],
            |row| row.get(0),
        )
        .expect("language index query should work");
    assert_eq!(language_index_exists, 1);

    let qualified_index_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_symbols_v2_qualified_symbol'",
            [],
            |row| row.get(0),
        )
        .expect("qualified symbol index query should work");
    assert_eq!(qualified_index_exists, 1);

    let legacy_symbol: String = connection
        .query_row(
            "SELECT symbol FROM symbols_v2 WHERE symbol_id = 1",
            [],
            |row| row.get(0),
        )
        .expect("legacy symbol row should survive migration");
    assert_eq!(legacy_symbol, "legacy_symbol");
}
