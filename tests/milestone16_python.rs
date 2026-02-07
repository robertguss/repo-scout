mod common;

use rusqlite::Connection;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
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
        symbol == "CONSTANT" && kind == "constant" && language == "python"
    }));
    assert!(symbols
        .iter()
        .any(|(symbol, kind, language, _)| symbol == "Runner" && kind == "class" && language == "python"));
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
fn milestone16_python_references_calls_imports() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/util.py", "def helper():\n    return 1\n");
    common::write_file(
        repo.path(),
        "src/app.py",
        "from util import helper as call_helper\n\ndef invoke():\n    return call_helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let refs_out = run_stdout(&["refs", "call_helper", "--repo", repo.path().to_str().unwrap()]);
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

    let find_out = run_stdout(&[
        "find",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let find_payload: serde_json::Value =
        serde_json::from_str(&find_out).expect("find json should parse");
    let find_results = find_payload["results"]
        .as_array()
        .expect("find results should be an array");
    assert!(!find_results.is_empty());
    assert_eq!(find_results[0]["file_path"], "src/util.py");

    let refs_out = run_stdout(&[
        "refs",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let refs_payload: serde_json::Value =
        serde_json::from_str(&refs_out).expect("refs json should parse");
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("refs results should be an array");
    assert!(refs_results
        .iter()
        .any(|item| item["why_matched"] == "ast_reference"));

    let diff_out_1 = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/util.py",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let diff_out_2 = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/util.py",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let diff_payload_1: serde_json::Value =
        serde_json::from_str(&diff_out_1).expect("diff-impact json should parse");
    let diff_payload_2: serde_json::Value =
        serde_json::from_str(&diff_out_2).expect("diff-impact json should parse");
    assert_eq!(diff_payload_1, diff_payload_2);
    let diff_results = diff_payload_1["results"]
        .as_array()
        .expect("results should be an array");
    assert!(diff_results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "call_helper"
            && item["relationship"] == "imported_by"
            && item["language"] == "python"
    }));

    let explain_out_1 = run_stdout(&[
        "explain",
        "Runner",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let explain_out_2 = run_stdout(&[
        "explain",
        "Runner",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let explain_payload_1: serde_json::Value =
        serde_json::from_str(&explain_out_1).expect("explain json should parse");
    let explain_payload_2: serde_json::Value =
        serde_json::from_str(&explain_out_2).expect("explain json should parse");
    assert_eq!(explain_payload_1, explain_payload_2);
    let explain_results = explain_payload_1["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!explain_results.is_empty());
    assert_eq!(explain_results[0]["language"], "python");
    assert_eq!(explain_results[0]["outbound"]["contains"], 1);
}
