mod common;

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
    let find_out = run_stdout(&["find", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(find_out.contains("[ast_definition ast_exact]"));

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

    let refs_out = run_stdout(&["refs", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(refs_out.contains("[ast_reference ast_likely]"));

    let impact_out = run_stdout(&["impact", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(impact_out.contains("called_by"));
    assert!(impact_out.contains("invoke"));
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

    let impact_out = run_stdout(&["impact", "helper", "--repo", repo.path().to_str().unwrap()]);
    assert!(impact_out.contains("imported_by"));

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
