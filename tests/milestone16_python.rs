mod common;

use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn run_repo_json(repo_root: &Path, args: &[&str]) -> Value {
    let repo_arg = repo_root.to_str().expect("repo path should be utf-8");
    let mut full_args = Vec::with_capacity(args.len() + 3);
    full_args.extend_from_slice(args);
    full_args.extend_from_slice(&["--repo", repo_arg, "--json"]);
    let out = run_stdout(&full_args);
    serde_json::from_str(&out).expect("command json should parse")
}

fn run_repo_json_deterministic(repo_root: &Path, args: &[&str]) -> Value {
    let first = run_repo_json(repo_root, args);
    let second = run_repo_json(repo_root, args);
    assert_eq!(first, second);
    first
}

fn payload_results<'a>(payload: &'a Value) -> &'a [Value] {
    payload["results"]
        .as_array()
        .expect("results should be an array")
}

fn assert_diff_impact_reports_imported_by(payload: &Value) {
    let results = payload_results(payload);
    assert!(results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "call_helper"
            && item["relationship"] == "imported_by"
            && item["language"] == "python"
    }));
}

fn assert_explain_reports_python(payload: &Value) {
    let results = payload_results(payload);
    assert!(!results.is_empty());
    assert_eq!(results[0]["language"], "python");
    assert_eq!(results[0]["outbound"]["contains"], 1);
}

#[test]
fn milestone16_python_definitions() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/app.py",
        "CONSTANT = 1\n\nclass Runner:\n    def run(self):\n        helper()\n\ndef helper():\n    return CONSTANT\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let find_out = run_stdout(&["find", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(find_out.contains("[ast_definition ast_exact]"));

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");
    let mut statement = connection
        .prepare(
            "SELECT symbol, kind, language, container
             FROM symbols_v2
             WHERE file_path = 'src/app.py'
             ORDER BY symbol ASC, kind ASC",
        )
        .expect("symbol query should prepare");

    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .expect("symbol rows should be queryable");

    let mut symbols = Vec::new();
    for row in rows {
        symbols.push(row.expect("row should decode"));
    }

    assert!(symbols.iter().any(|(symbol, kind, language, _)| {
        symbol == "CONSTANT" && kind == "const" && language == "python"
    }));
    assert!(
        symbols
            .iter()
            .any(|(symbol, kind, language, _)| symbol == "Runner"
                && kind == "class"
                && language == "python")
    );
    assert!(symbols.iter().any(|(symbol, kind, language, _)| {
        symbol == "helper" && kind == "function" && language == "python"
    }));
    assert!(symbols.iter().any(|(symbol, kind, language, container)| {
        symbol == "run"
            && kind == "method"
            && language == "python"
            && container.as_deref() == Some("Runner")
    }));
}

#[test]
fn milestone16_python_dotted_imports_bind_top_level_package() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/package/module.py", "VALUE = 1\n");
    common::write_file(
        repo.path(),
        "src/app.py",
        "import package.module\n\ndef invoke():\n    return package.module.VALUE\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");
    let mut statement = connection
        .prepare(
            "SELECT symbol
             FROM symbols_v2
             WHERE file_path = 'src/app.py' AND kind = 'import'
             ORDER BY symbol ASC",
        )
        .expect("import symbol query should prepare");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("import symbol rows should be queryable");
    let mut import_symbols = Vec::new();
    for row in rows {
        import_symbols.push(row.expect("row should decode"));
    }

    assert!(import_symbols.contains(&"package".to_string()));
    assert!(!import_symbols.contains(&"module".to_string()));

    let refs_out = run_stdout(&[
        "refs",
        "package",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let refs_payload: serde_json::Value =
        serde_json::from_str(&refs_out).expect("refs json should parse");
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("refs results should be an array");
    assert!(
        refs_results
            .iter()
            .any(|item| item["file_path"] == "src/app.py")
    );
}

#[test]
fn milestone16_python_references_calls_imports() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/util.py", "def helper():\n    return 1\n");
    common::write_file(
        repo.path(),
        "src/app.py",
        "from util import helper as call_helper\n\ndef invoke():\n    return call_helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let refs_out = run_stdout(&[
        "refs",
        "call_helper",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(refs_out.contains("[ast_reference ast_likely]"));

    let impact_helper = run_stdout(&["impact", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(impact_helper.contains("imported_by"));
    assert!(impact_helper.contains("call_helper"));

    let impact_import = run_stdout(&[
        "impact",
        "call_helper",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(impact_import.contains("called_by"));
    assert!(impact_import.contains("invoke"));
}

#[test]
fn milestone16_python_edges_and_queries() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/util.py", "def helper():\n    return 1\n");
    common::write_file(
        repo.path(),
        "src/app.py",
        "from util import helper as call_helper\n\nclass Runner:\n    def run(self):\n        return call_helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let find_payload = run_repo_json(repo.path(), &["find", "helper"]);
    let find_results = payload_results(&find_payload);
    assert!(!find_results.is_empty());
    assert_eq!(find_results[0]["file_path"], "src/util.py");

    let refs_payload = run_repo_json(repo.path(), &["refs", "helper"]);
    let refs_results = payload_results(&refs_payload);
    assert!(
        refs_results
            .iter()
            .any(|item| item["why_matched"] == "ast_reference")
    );

    let diff_payload = run_repo_json_deterministic(
        repo.path(),
        &[
            "diff-impact",
            "--changed-file",
            "src/util.py",
        ],
    );
    assert_diff_impact_reports_imported_by(&diff_payload);

    let explain_payload = run_repo_json_deterministic(
        repo.path(),
        &[
            "explain",
            "Runner",
        ],
    );
    assert_explain_reports_python(&explain_payload);
}

#[test]
fn milestone16_python_import_edges_skip_import_targets_after_deferred_resolution() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/a.py",
        "from missing_a import helper as call_helper\n",
    );
    common::write_file(repo.path(), "src/b.py", "from missing_b import helper\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");
    let invalid_edge_count: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM symbol_edges_v2 e
             JOIN symbols_v2 ts ON ts.symbol_id = e.to_symbol_id
             WHERE e.edge_kind IN ('imports', 'implements')
               AND ts.kind = 'import'",
            [],
            |row| row.get(0),
        )
        .expect("edge count query should execute");

    assert_eq!(invalid_edge_count, 0);
}
