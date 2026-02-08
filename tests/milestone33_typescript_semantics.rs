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

fn diff_impact_payload(repo: &Path, changed_file: &str) -> Value {
    let output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        changed_file,
        "--repo",
        repo.to_str().expect("repo path should be utf-8"),
        "--json",
    ]);
    serde_json::from_str(&output).expect("diff-impact json should parse")
}

fn has_called_by_row(results: &[Value], symbol: &str, file_path: &str) -> bool {
    results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == symbol
            && item["relationship"] == "called_by"
            && item["file_path"] == file_path
            && item["distance"] == 1
    })
}

#[test]
fn milestone33_typescript_namespace_alias_resolves_changed_callee() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/util_a.ts",
        "export function helper(): number {\n  return 1;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/util_b.ts",
        "export function helper(): number {\n  return 2;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/app.ts",
        "import * as utilA from \"./util_a\";\nimport * as utilB from \"./util_b\";\n\nexport function run(): number {\n  return utilA.helper() + utilB.helper();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(
        has_called_by_row(results, "run", "src/app.ts"),
        "expected namespace alias call to resolve changed callee with called_by row"
    );
}

#[test]
fn milestone33_typescript_member_call_prefers_import_context() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/util_a.ts",
        "export function helper(): number {\n  return 1;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/util_b.ts",
        "export function helper(): number {\n  return 2;\n}\n",
    );
    common::write_file(
        repo.path(),
        "src/app.ts",
        "import * as utilA from \"./util_a\";\n\nexport function run(): number {\n  return utilA.helper();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let util_a_payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    let util_a_results = util_a_payload["results"]
        .as_array()
        .expect("diff-impact results should be array");
    assert!(
        has_called_by_row(util_a_results, "run", "src/app.ts"),
        "expected utilA.helper() to resolve to util_a helper"
    );

    let util_b_payload = diff_impact_payload(repo.path(), "src/util_b.ts");
    let util_b_results = util_b_payload["results"]
        .as_array()
        .expect("diff-impact results should be array");
    assert!(
        !has_called_by_row(util_b_results, "run", "src/app.ts"),
        "run should not be impacted when util_b helper is unchanged in this fixture"
    );
}

#[test]
fn milestone33_typescript_semantics_preserve_existing_m15_behavior() {
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
