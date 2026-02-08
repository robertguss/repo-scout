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
fn milestone34_python_module_alias_resolves_changed_callee() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/pkg_a/util.py", "def helper():\n    return 1\n");
    common::write_file(repo.path(), "src/pkg_b/util.py", "def helper():\n    return 2\n");
    common::write_file(
        repo.path(),
        "src/py_app.py",
        "import pkg_a.util as util_a\nimport pkg_b.util as util_b\n\n\ndef run_py():\n    return util_a.helper() + util_b.helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(
        has_called_by_row(results, "run_py", "src/py_app.py"),
        "expected Python module alias call to resolve changed callee"
    );
}

#[test]
fn milestone34_python_attribute_call_prefers_import_context() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/pkg_a/util.py", "def helper():\n    return 1\n");
    common::write_file(repo.path(), "src/pkg_b/util.py", "def helper():\n    return 2\n");
    common::write_file(
        repo.path(),
        "src/py_app.py",
        "import pkg_a.util as util_a\n\n\ndef run_py():\n    return util_a.helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let util_a_payload = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let util_a_results = util_a_payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");
    assert!(
        has_called_by_row(util_a_results, "run_py", "src/py_app.py"),
        "expected util_a.helper() to resolve to src/pkg_a/util.py::helper"
    );

    let util_b_payload = diff_impact_payload(repo.path(), "src/pkg_b/util.py");
    let util_b_results = util_b_payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");
    assert!(
        !has_called_by_row(util_b_results, "run_py", "src/py_app.py"),
        "run_py should not be impacted when src/pkg_b/util.py changes in this fixture"
    );
}

#[test]
fn milestone34_python_semantics_preserve_existing_m16_behavior() {
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
    let refs_payload: Value = serde_json::from_str(&refs_out).expect("refs json should parse");
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("refs results should be array");
    assert!(
        refs_results
            .iter()
            .any(|item| item["file_path"] == "src/app.py")
    );
}
