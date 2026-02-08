mod common;

use serde_json::Value;
use std::path::Path;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn write_semantic_precision_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/util_a.ts",
        include_str!("fixtures/phase8/semantic_precision/src/util_a.ts"),
    );
    common::write_file(
        repo,
        "src/util_b.ts",
        include_str!("fixtures/phase8/semantic_precision/src/util_b.ts"),
    );
    common::write_file(
        repo,
        "src/app.ts",
        include_str!("fixtures/phase8/semantic_precision/src/app.ts"),
    );
    common::write_file(
        repo,
        "src/pkg_a/util.py",
        include_str!("fixtures/phase8/semantic_precision/src/pkg_a/util.py"),
    );
    common::write_file(
        repo,
        "src/pkg_b/util.py",
        include_str!("fixtures/phase8/semantic_precision/src/pkg_b/util.py"),
    );
    common::write_file(
        repo,
        "src/py_app.py",
        include_str!("fixtures/phase8/semantic_precision/src/py_app.py"),
    );
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

fn has_called_by_row(results: &[Value], symbol: &str, file_path: &str, distance: u64) -> bool {
    results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["symbol"] == symbol
            && item["relationship"] == "called_by"
            && item["file_path"] == file_path
            && item["distance"] == distance
    })
}

#[test]
fn milestone37_typescript_namespace_alias_diff_impact_recalls_caller() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/util_a.ts");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(
        has_called_by_row(results, "run_namespace_a", "src/app.ts", 1),
        "expected utilA.helper() namespace alias call to resolve changed callee"
    );
    assert!(
        has_called_by_row(results, "run_alias_a", "src/app.ts", 1),
        "expected helperA() alias-import call to resolve directly to src/util_a.ts::helper"
    );
    assert!(
        !has_called_by_row(results, "run_namespace_b", "src/app.ts", 1),
        "namespace caller for util_b should not be impacted when util_a changes"
    );
    assert!(
        !has_called_by_row(results, "run_alias_b", "src/app.ts", 1),
        "alias caller for util_b should not be impacted when util_a changes"
    );
}

#[test]
fn milestone37_python_module_alias_diff_impact_recalls_caller() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let results = payload["results"]
        .as_array()
        .expect("diff-impact results should be an array");

    assert!(
        has_called_by_row(results, "run_module_a", "src/py_app.py", 1),
        "expected util_a.helper() module alias call to resolve changed callee"
    );
    assert!(
        has_called_by_row(results, "run_alias_a", "src/py_app.py", 1),
        "expected helper_a() from-import alias call to resolve changed callee"
    );
    assert!(
        !has_called_by_row(results, "run_module_b", "src/py_app.py", 1),
        "module alias caller for pkg_b should not be impacted when pkg_a changes"
    );
    assert!(
        !has_called_by_row(results, "run_alias_b", "src/py_app.py", 1),
        "from-import alias caller for pkg_b should not be impacted when pkg_a changes"
    );
}

#[test]
fn milestone37_semantic_precision_deterministic_ordering() {
    let repo = common::temp_repo();
    write_semantic_precision_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let ts_a = diff_impact_payload(repo.path(), "src/util_a.ts");
    let ts_b = diff_impact_payload(repo.path(), "src/util_a.ts");
    assert_eq!(ts_a, ts_b);

    let py_a = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    let py_b = diff_impact_payload(repo.path(), "src/pkg_a/util.py");
    assert_eq!(py_a, py_b);
}
