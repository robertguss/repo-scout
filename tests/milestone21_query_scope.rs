mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone21_refs_code_only_omits_docs_text_fallback() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/code.rs", "// phase21codeonlytoken\n");
    common::write_file(repo.path(), "docs/guide.md", "phase21codeonlytoken\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline_out = run_stdout(&[
        "refs",
        "phase21codeonlytoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let baseline: Value = serde_json::from_str(&baseline_out).expect("refs json should parse");
    let baseline_results = baseline["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        baseline_results
            .iter()
            .any(|item| item["file_path"] == "docs/guide.md")
    );
    assert!(
        baseline_results
            .iter()
            .any(|item| item["file_path"] == "src/code.rs")
    );

    let scoped_out = run_stdout(&[
        "refs",
        "phase21codeonlytoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--json",
    ]);
    let scoped: Value = serde_json::from_str(&scoped_out).expect("refs json should parse");
    let scoped_results = scoped["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        scoped_results
            .iter()
            .any(|item| item["file_path"] == "src/code.rs")
    );
    assert!(
        !scoped_results
            .iter()
            .any(|item| item["file_path"] == "docs/guide.md")
    );
}

#[test]
fn milestone21_refs_exclude_tests_omits_test_paths() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/code.rs", "// phase21testfiltertoken\n");
    common::write_file(
        repo.path(),
        "tests/query_scope_test.rs",
        "// phase21testfiltertoken\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline_out = run_stdout(&[
        "refs",
        "phase21testfiltertoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let baseline: Value = serde_json::from_str(&baseline_out).expect("refs json should parse");
    let baseline_results = baseline["results"]
        .as_array()
        .expect("results should be array");
    assert!(
        baseline_results
            .iter()
            .any(|item| item["file_path"] == "tests/query_scope_test.rs")
    );

    let scoped_out = run_stdout(&[
        "refs",
        "phase21testfiltertoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--exclude-tests",
        "--json",
    ]);
    let scoped: Value = serde_json::from_str(&scoped_out).expect("refs json should parse");
    let scoped_results = scoped["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        scoped_results
            .iter()
            .any(|item| item["file_path"] == "src/code.rs")
    );
    assert!(
        !scoped_results
            .iter()
            .any(|item| item["file_path"] == "tests/query_scope_test.rs")
    );
}

#[test]
fn milestone21_find_scope_flags_keep_ast_priority_and_determinism() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn helper() {}\n\npub fn wrapper() {\n    helper();\n}\n",
    );
    common::write_file(repo.path(), "docs/guide.md", "helper helper helper\n");
    common::write_file(repo.path(), "tests/helper_test.rs", "helper helper\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline_out = run_stdout(&[
        "find",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let scoped_out = run_stdout(&[
        "find",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--json",
    ]);
    let scoped_again_out = run_stdout(&[
        "find",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--json",
    ]);

    let baseline: Value = serde_json::from_str(&baseline_out).expect("find json should parse");
    let baseline_results = baseline["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(baseline_results[0]["why_matched"], "ast_definition");

    let scoped: Value = serde_json::from_str(&scoped_out).expect("find json should parse");
    let scoped_results = scoped["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(scoped_results[0]["why_matched"], "ast_definition");

    assert_eq!(baseline_out, scoped_out);
    assert_eq!(scoped_out, scoped_again_out);
}
