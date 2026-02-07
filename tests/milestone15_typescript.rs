mod common;

use predicates::str::contains;
use rusqlite::Connection;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone15_typescript_definitions() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/app.ts",
        "export interface Engine {\n  run(): void;\n}\n\nexport enum Mode { Fast, Slow }\n\nexport type EngineId = string;\n\nexport class Runner implements Engine {\n  run(): void {\n    helper();\n  }\n}\n\nexport function helper(): void {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    common::repo_scout_cmd()
        .args(["find", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_definition ast_exact]"));

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");
    let mut statement = connection
        .prepare(
            "SELECT symbol, kind, language, container
             FROM symbols_v2
             WHERE file_path = 'src/app.ts'
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
        symbol == "helper" && kind == "function" && language == "typescript"
    }));
    assert!(symbols.iter().any(|(symbol, kind, language, _)| {
        symbol == "Runner" && kind == "class" && language == "typescript"
    }));
    assert!(symbols.iter().any(|(symbol, kind, language, _)| {
        symbol == "Engine" && kind == "interface" && language == "typescript"
    }));
    assert!(
        symbols
            .iter()
            .any(|(symbol, kind, language, _)| symbol == "Mode"
                && kind == "enum"
                && language == "typescript")
    );
    assert!(symbols.iter().any(|(symbol, kind, language, _)| {
        symbol == "EngineId" && kind == "type_alias" && language == "typescript"
    }));
    assert!(symbols.iter().any(|(symbol, kind, language, container)| {
        symbol == "run"
            && kind == "method"
            && language == "typescript"
            && container.as_deref() == Some("Runner")
    }));
}

#[test]
fn milestone15_typescript_references_and_calls() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/app.ts",
        "export function helper(): void {}\n\nexport const invoke = () => {\n  helper();\n};\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    common::repo_scout_cmd()
        .args(["refs", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_reference ast_likely]"));
    common::repo_scout_cmd()
        .args(["impact", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("called_by"))
        .stdout(contains("invoke"));
}

#[test]
fn milestone15_typescript_edges_and_queries() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/util.ts",
        "export function helper(): void {}\n",
    );
    common::write_file(
        repo.path(),
        "src/contracts.ts",
        "export interface Contract {\n  run(): void;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/app.ts",
        "import { helper as callHelper } from \"./util\";\nimport { Contract } from \"./contracts\";\n\nexport class Runner implements Contract {\n  run(): void {\n    callHelper();\n  }\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    common::repo_scout_cmd()
        .args(["impact", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("imported_by"));

    let diff_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/contracts.ts",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let diff_payload: serde_json::Value =
        serde_json::from_str(&diff_out).expect("diff-impact json should parse");
    let diff_results = diff_payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(diff_results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "Runner"
            && item["relationship"] == "implemented_by"
            && item["language"] == "typescript"
    }));

    let explain_out = run_stdout(&[
        "explain",
        "Runner",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let explain_payload: serde_json::Value =
        serde_json::from_str(&explain_out).expect("explain json should parse");
    let explain_results = explain_payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!explain_results.is_empty());
    assert_eq!(explain_results[0]["language"], "typescript");
    assert_eq!(explain_results[0]["outbound"]["implements"], 1);
    assert_eq!(explain_results[0]["outbound"]["contains"], 1);
}

#[test]
fn milestone15_typescript_default_and_named_import_hints_disambiguate_implements_target() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/contracts_a.ts",
        "export default interface Contract {\n  run(): void;\n}\n\nexport const TOKEN = 1;\n",
    );
    common::write_file(
        repo.path(),
        "src/contracts_b.ts",
        "export interface Contract {\n  run(): void;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/app.ts",
        "import Contract, { TOKEN } from \"./contracts_a\";\n\nvoid TOKEN;\n\nexport class Runner implements Contract {\n  run(): void {}\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let db_path = repo.path().join(".repo-scout").join("index.db");
    let connection = Connection::open(db_path).expect("db should open");
    let mut statement = connection
        .prepare(
            "SELECT ts.file_path
             FROM symbol_edges_v2 e
             JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
             JOIN symbols_v2 ts ON ts.symbol_id = e.to_symbol_id
             WHERE fs.symbol = 'Runner' AND e.edge_kind = 'implements'
             ORDER BY ts.file_path ASC",
        )
        .expect("edge query should prepare");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("edge rows should be queryable");
    let mut target_paths = Vec::new();
    for row in rows {
        target_paths.push(row.expect("row should decode"));
    }

    assert_eq!(target_paths, vec!["src/contracts_a.ts".to_string()]);
}
