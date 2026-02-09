mod common;

use rusqlite::Connection;
use std::fs;

/// Executes repo-scout with the provided arguments and returns stdout as a `String`.
///
/// The returned string contains the command's standard output decoded as UTF-8.
///
/// # Examples
///
/// ```
/// let out = run_stdout(&["status", "--repo", "/tmp/repo"]);
/// assert!(out.contains("schema_version"));
/// ```
fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

/// Create a v1-format repository index and return the path to its SQLite database.
///
/// This creates a `.repo-scout` directory, initializes `index.db` with the v1 schema,
/// and seeds representative rows (`schema_version`, one indexed file, one text
/// occurrence, one AST definition, and one AST reference).
///
/// # Examples
///
/// ```
/// use std::fs;
/// use std::path::Path;
///
/// let repo = std::env::temp_dir().join("repo_scout_example");
/// let _ = fs::remove_dir_all(&repo); // ignore errors from previous runs
/// fs::create_dir_all(&repo).unwrap();
/// let db_path = build_v1_index(&repo);
/// assert!(db_path.exists());
/// ```
fn build_v1_index(repo_path: &std::path::Path) -> std::path::PathBuf {
    let index_dir = repo_path.join(".repo-scout");
    fs::create_dir_all(&index_dir).expect("index directory should exist");
    let db_path = index_dir.join("index.db");

    let connection = Connection::open(&db_path).expect("v1 db should open");
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
            INSERT INTO meta(key, value) VALUES ('schema_version', '1');
            INSERT INTO indexed_files(file_path, content_hash) VALUES ('src/lib.rs', 'hash-v1');
            INSERT INTO text_occurrences(file_path, symbol, line, column)
                VALUES ('src/lib.rs', 'legacy_symbol', 1, 1);
            INSERT INTO ast_definitions(file_path, symbol, kind, line, column)
                VALUES ('src/lib.rs', 'legacy_fn', 'function', 1, 1);
            INSERT INTO ast_references(file_path, symbol, line, column)
                VALUES ('src/lib.rs', 'legacy_fn', 3, 5);
            "#,
        )
        .expect("v1 schema fixture should be created");
    db_path
}

#[test]
fn milestone6_schema_v1_upgrades_to_v3_without_data_loss() {
    let repo = common::temp_repo();
    let db_path = build_v1_index(repo.path());

    let status = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        status.contains("schema_version: 3"),
        "store bootstrap should migrate v1 dbs to schema v3"
    );

    let connection = Connection::open(db_path).expect("db should remain readable");
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM text_occurrences WHERE symbol = 'legacy_symbol'",
            [],
            |row| row.get(0),
        )
        .expect("legacy rows should survive migration");
    assert_eq!(count, 1);

    let symbols_table_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'symbols_v2'",
            [],
            |row| row.get(0),
        )
        .expect("sqlite_master query should work");
    assert_eq!(symbols_table_exists, 1);
}

/// Verifies that applying the schema migration is idempotent and that migrated artifacts persist.
///
/// Runs the status command twice to ensure the repository index is migrated to schema version 3 on
/// repeated runs, then opens the index database to assert that the `meta` table contains
/// `schema_version = "3"` and that the `symbol_edges_v2` table exists.
#[test]
fn milestone6_schema_migration_is_idempotent() {
    let repo = common::temp_repo();
    let db_path = build_v1_index(repo.path());

    let first = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    let second = run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);
    assert!(first.contains("schema_version: 3"));
    assert!(second.contains("schema_version: 3"));

    let connection = Connection::open(db_path).expect("db should remain readable");
    let version: String = connection
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .expect("meta schema_version should exist");
    assert_eq!(version, "3");

    let edge_table_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'symbol_edges_v2'",
            [],
            |row| row.get(0),
        )
        .expect("sqlite_master query should work");
    assert_eq!(edge_table_exists, 1);
}

#[test]
fn milestone6_schema_upgrade_preserves_v1_find_refs_behavior() {
    let repo = common::temp_repo();
    build_v1_index(repo.path());

    run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);

    let find = run_stdout(&["find", "legacy_fn", "--repo", repo.path().to_str().unwrap()]);
    assert!(find.contains("command: find"));
    assert!(find.contains("results: 1"));
    assert!(find.contains("[ast_definition ast_exact]"));

    let refs = run_stdout(&["refs", "legacy_fn", "--repo", repo.path().to_str().unwrap()]);
    assert!(refs.contains("command: refs"));
    assert!(refs.contains("results: 1"));
    assert!(refs.contains("[ast_reference ast_likely]"));
}
