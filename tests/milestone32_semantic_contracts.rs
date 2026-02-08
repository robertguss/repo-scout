mod common;

use serde_json::Value;
use std::path::Path;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn write_phase7_semantic_fixture(repo: &Path) {
    common::write_file(repo, "src/util_a.ts", "export function helper(): number {\n  return 1;\n}\n");
    common::write_file(repo, "src/util_b.ts", "export function helper(): number {\n  return 2;\n}\n");
    common::write_file(
        repo,
        "src/app.ts",
        "import * as utilA from \"./util_a\";\nimport * as utilB from \"./util_b\";\n\nexport function run(): number {\n  return utilA.helper() + utilB.helper();\n}\n",
    );

    common::write_file(
        repo,
        "src/pkg_a/util.py",
        "def helper():\n    return 1\n",
    );
    common::write_file(
        repo,
        "src/pkg_b/util.py",
        "def helper():\n    return 2\n",
    );
    common::write_file(
        repo,
        "src/py_app.py",
        "import pkg_a.util as util_a\nimport pkg_b.util as util_b\n\ndef run_py():\n    return util_a.helper() + util_b.helper()\n",
    );
}

fn diff_impact_payload(repo: &Path, changed_file: &str) -> Value {
    let diff_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        changed_file,
        "--repo",
        repo.to_str().expect("repo path should be utf-8"),
        "--json",
    ]);
    serde_json::from_str(&diff_out).expect("diff-impact json should parse")
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
fn milestone32_typescript_namespace_alias_call_contract() {
    let repo = common::temp_repo();
    write_phase7_semantic_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "helper"
            && item["relationship"] == "changed_symbol"
            && item["file_path"] == "src/util_a.ts"
    }));
    assert!(
        has_called_by_row(results, "run", "src/app.ts"),
        "expected TypeScript namespace call to produce called_by row for src/app.ts::run"
    );
}

#[test]
fn milestone32_python_module_alias_call_contract() {
    let repo = common::temp_repo();
    write_phase7_semantic_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == "helper"
            && item["relationship"] == "changed_symbol"
            && item["file_path"] == "src/pkg_a/util.py"
    }));
    assert!(
        has_called_by_row(results, "run_py", "src/py_app.py"),
        "expected Python module-alias call to produce called_by row for src/py_app.py::run_py"
    );
}

#[test]
fn milestone32_schema_contracts_stay_stable() {
    let repo = common::temp_repo();
    write_phase7_semantic_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let find_out = run_stdout(&["find", "helper", "--repo", repo.path().to_str().unwrap(), "--json"]);
    let find_payload: Value = serde_json::from_str(&find_out).expect("find json should parse");
    assert_eq!(find_payload["schema_version"], 1);
    assert_eq!(find_payload["command"], "find");

    let impact_out = run_stdout(&["impact", "helper", "--repo", repo.path().to_str().unwrap(), "--json"]);
    let impact_payload: Value = serde_json::from_str(&impact_out).expect("impact json should parse");
    assert_eq!(impact_payload["schema_version"], 2);
    assert_eq!(impact_payload["command"], "impact");

    let ts_diff_payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    assert_eq!(ts_diff_payload["schema_version"], 3);
    assert_eq!(ts_diff_payload["command"], "diff-impact");
    assert!(
        has_called_by_row(
            ts_diff_payload["results"]
                .as_array()
                .expect("diff-impact results should be array"),
            "run",
            "src/app.ts"
        ),
        "schema 3 should include semantic caller row for TypeScript fixture"
    );

    let py_diff_payload = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    assert_eq!(py_diff_payload["schema_version"], 3);
    assert_eq!(py_diff_payload["command"], "diff-impact");
    assert!(
        has_called_by_row(
            py_diff_payload["results"]
                .as_array()
                .expect("diff-impact results should be array"),
            "run_py",
            "src/py_app.py"
        ),
        "schema 3 should include semantic caller row for Python fixture"
    );
}
