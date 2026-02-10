mod common;

use common::run_stdout;
use serde_json::Value;

fn parse_json(output: &str) -> Value {
    serde_json::from_str(output).expect("json output should parse")
}

fn has_step(results: &[Value], step: &str, scope: &str) -> bool {
    results
        .iter()
        .any(|row| row["step"] == step && row["scope"] == scope)
}

fn has_called_by_row(results: &[Value], symbol: &str, file_path: &str) -> bool {
    results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == symbol
            && row["relationship"] == "called_by"
            && row["file_path"] == file_path
    })
}

#[test]
fn milestone60_tests_for_uses_pytest_targets_when_explicitly_configured() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "pytest.ini", "[pytest]\n");
    common::write_file(
        repo.path(),
        "src/service.py",
        "def compute_plan():\n    return 1\n",
    );
    common::write_file(
        repo.path(),
        "tests/test_service.py",
        "def test_compute_plan():\n    assert compute_plan() == 1\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results
            .iter()
            .any(|row| row["target"] == "tests/test_service.py"),
        "expected pytest-detected python test file to be runnable in tests-for"
    );
}

#[test]
fn milestone60_tests_for_requires_explicit_pytest_detection() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/service.py",
        "def compute_plan():\n    return 1\n",
    );
    common::write_file(
        repo.path(),
        "tests/test_service.py",
        "def test_compute_plan():\n    assert compute_plan() == 1\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results.is_empty(),
        "without explicit pytest signal, python tests should not be treated as runnable"
    );
}

#[test]
fn milestone60_verify_plan_emits_pytest_targeted_and_full_suite_steps() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "pyproject.toml",
        "[tool.pytest.ini_options]\naddopts = \"-q\"\n",
    );
    common::write_file(
        repo.path(),
        "src/service.py",
        "def compute_plan():\n    return 1\n",
    );
    common::write_file(
        repo.path(),
        "tests/unit/test_service.py",
        "def test_compute_plan():\n    assert compute_plan() == 1\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.py",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        has_step(results, "pytest tests/unit/test_service.py", "targeted"),
        "expected verify-plan to include runnable pytest targeted command"
    );
    assert!(
        has_step(results, "pytest", "full_suite"),
        "expected verify-plan full-suite gate to use pytest in explicit python runner contexts"
    );
}

#[test]
fn milestone60_tests_for_supports_python_tests_suffix_pattern() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "pytest.ini", "[pytest]\n");
    common::write_file(
        repo.path(),
        "src/service.py",
        "def compute_plan():\n    return 1\n",
    );
    common::write_file(
        repo.path(),
        "src/service_tests.py",
        "def test_compute_plan():\n    assert compute_plan() == 1\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results
            .iter()
            .any(|row| row["target"] == "src/service_tests.py"),
        "expected *_tests.py files to be classified as python test-like runnable targets"
    );
}

#[test]
fn milestone60_diff_impact_resolves_python_relative_import_calls() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/pkg/util.py",
        "def helper():\n    return 1\n",
    );
    common::write_file(
        repo.path(),
        "src/pkg/consumer.py",
        "from .util import helper\n\n\ndef run():\n    return helper()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/pkg/util.py",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        has_called_by_row(results, "run", "src/pkg/consumer.py"),
        "expected relative import call edge to attribute caller when callee file changes"
    );
}
