mod common;

use rusqlite::Connection;

/// Run the repo-scout command with the given arguments and return its standard output as a `String`.
///
/// Panics if the command exits with a non-zero status or if the command's stdout is not valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// let out = run_stdout(&["index", "--repo", "path/to/repo"]);
/// assert!(out.contains("results"));
/// ```
fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone7_struct_enum_trait_defs() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    for symbol in ["Launcher", "LaunchState", "Runnable"] {
        let out = run_stdout(&["find", symbol, "--repo", repo.path().to_str().unwrap()]);
        assert!(
            !out.contains("results: 0"),
            "{symbol} should resolve to AST definitions"
        );
        assert!(out.contains("src/lib.rs"));
        assert!(out.contains("[ast_definition ast_exact]"));
    }
}

/// Verifies that an `impl` method is indexed and its containing type is recorded.
///
/// Confirms the symbol `run` resolves to a single AST definition and that the persisted index entry for `run` has `kind` equal to `"function"` and `container` equal to `"Launcher"`.
///
/// # Examples
///
/// ```
/// // Run via `cargo test` to execute this integration test.
/// milestone7_impl_method_container();
/// ```
#[test]
fn milestone7_impl_method_container() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let find = run_stdout(&["find", "run", "--repo", repo.path().to_str().unwrap()]);
    assert!(find.contains("results: 1"));
    assert!(find.contains("[ast_definition ast_exact]"));

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("index db should open");
    let (kind, container): (String, Option<String>) = connection
        .query_row(
            "SELECT kind, container
             FROM symbols_v2
             WHERE symbol = 'run'
             ORDER BY file_path ASC, start_line ASC, start_column ASC
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("symbols_v2 should persist method metadata");

    assert_eq!(kind, "function");
    assert_eq!(container.as_deref(), Some("Launcher"));
}

#[test]
fn milestone7_module_alias_const_use() {
    let repo = common::temp_repo();
    let source = format!(
        "{}\nuse launch::Launcher as LocalLauncher;\n",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs")
    );
    common::write_file(repo.path(), "src/lib.rs", &source);

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    for symbol in ["launch", "LaunchId", "DEFAULT_RETRIES", "LocalLauncher"] {
        let out = run_stdout(&["find", symbol, "--repo", repo.path().to_str().unwrap()]);
        assert!(
            out.contains("results: 1"),
            "{symbol} should have one AST definition"
        );
        assert!(out.contains("[ast_definition ast_exact]"));
    }
}

/// Verifies that symbol spans and function signature summaries are persisted in the index database.
///
/// This test indexes a fixture Rust crate, opens `.repo-scout/index.db`, then queries `symbols_v2` for
/// the symbol `start_engine`. It asserts the recorded span extends beyond its start position and that
/// a function signature containing `fn start_engine` is present.
///
/// # Examples
///
/// ```
/// // After indexing a repository, query `symbols_v2` for `start_engine` and assert:
/// // - end_line >= start_line
/// // - end_column > start_column
/// // - signature contains "fn start_engine"
/// ```
#[test]
fn milestone7_spans_and_signatures_persist() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("index db should open");
    let (start_line, start_column, end_line, end_column, signature): (
        i64,
        i64,
        i64,
        i64,
        Option<String>,
    ) = connection
        .query_row(
            "SELECT start_line, start_column, end_line, end_column, signature
                 FROM symbols_v2
                 WHERE symbol = 'start_engine'
                 ORDER BY start_line ASC
                 LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("start_engine should be persisted in symbols_v2");

    assert!(end_line >= start_line);
    assert!(
        end_line > start_line || end_column > start_column,
        "span should either end on a later line or extend beyond the start column on the same line"
    );
    let signature = signature.expect("function signature summary should be persisted");
    assert!(signature.contains("fn start_engine"));
}
