mod common;

use common::run_stdout;
use serde_json::Value;
use std::collections::HashSet;

#[test]
fn milestone49_refs_deduplicates_ast_rows() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn helper() -> Option<u8> {\n    Some(1)\n}\n\npub fn caller() -> bool {\n    helper().is_some()\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "refs",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("refs json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        !results.is_empty(),
        "expected at least one ast_reference row for helper"
    );

    let mut seen = HashSet::new();
    for row in results {
        let file_path = row["file_path"]
            .as_str()
            .expect("file_path should be string");
        let line = row["line"].as_u64().expect("line should be integer");
        let column = row["column"].as_u64().expect("column should be integer");
        let symbol = row["symbol"].as_str().expect("symbol should be string");
        let why_matched = row["why_matched"]
            .as_str()
            .expect("why_matched should be string");
        let key = (
            file_path.to_string(),
            line,
            column,
            symbol.to_string(),
            why_matched.to_string(),
        );
        assert!(
            seen.insert(key),
            "duplicate refs row returned for location and match kind"
        );
    }
}

#[test]
fn milestone49_verify_plan_targets_remain_deterministic() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn rust_target() -> usize {\n    1\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/rust_target.rs",
        "#[test]\nfn rust_target_test() {\n    let _ = rust_target();\n}\n",
    );
    common::write_file(
        repo.path(),
        "tests/python_target.py",
        "def test_python_target():\n    return rust_target()\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "tests/python_target.py",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("verify-plan json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        results
            .iter()
            .any(|row| row["step"] == "cargo test --test rust_target"),
        "expected runnable Rust target to be present"
    );
    assert!(
        !results
            .iter()
            .any(|row| row["step"] == "cargo test --test python_target"),
        "non-Rust tests/python_target.py should not map to cargo test --test"
    );
}

#[test]
fn milestone49_scope_filtering_preserves_contract() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/main.ts", "// phase49scopetoken\n");
    common::write_file(repo.path(), "src/widget.test.ts", "// phase49scopetoken\n");
    common::write_file(repo.path(), "src/test_widget.py", "# phase49scopetoken\n");
    common::write_file(repo.path(), "src/widget_test.py", "# phase49scopetoken\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline_out = run_stdout(&[
        "refs",
        "phase49scopetoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let baseline_payload: Value =
        serde_json::from_str(&baseline_out).expect("refs json should parse");
    let baseline_results = baseline_payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        baseline_results
            .iter()
            .any(|row| row["file_path"] == "src/widget.test.ts")
    );
    assert!(
        baseline_results
            .iter()
            .any(|row| row["file_path"] == "src/test_widget.py")
    );
    assert!(
        baseline_results
            .iter()
            .any(|row| row["file_path"] == "src/widget_test.py")
    );

    let scoped_out = run_stdout(&[
        "refs",
        "phase49scopetoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--exclude-tests",
        "--json",
    ]);
    let scoped_payload: Value = serde_json::from_str(&scoped_out).expect("refs json should parse");
    let scoped_results = scoped_payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/main.ts")
    );
    assert!(
        !scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/widget.test.ts"),
        "--exclude-tests should drop .test.ts paths"
    );
    assert!(
        !scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/test_widget.py"),
        "--exclude-tests should drop test_*.py paths"
    );
    assert!(
        !scoped_results
            .iter()
            .any(|row| row["file_path"] == "src/widget_test.py"),
        "--exclude-tests should drop *_test.py paths"
    );
}
