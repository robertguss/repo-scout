mod common;

use common::run_stdout;
use serde_json::Value;
use std::path::Path;

fn run_query_json(repo_root: &Path, args: &[&str]) -> Value {
    let repo_arg = repo_root.to_str().expect("repo path should be utf-8");
    let mut full_args = Vec::with_capacity(args.len() + 3);
    full_args.extend_from_slice(args);
    full_args.extend_from_slice(&["--repo", repo_arg, "--json"]);
    let out = run_stdout(&full_args);
    serde_json::from_str(&out).expect("query json should parse")
}

fn run_query_json_deterministic(repo_root: &Path, args: &[&str]) -> Value {
    let first = run_query_json(repo_root, args);
    let second = run_query_json(repo_root, args);
    assert_eq!(first, second);
    first
}

fn query_results<'a>(payload: &'a Value) -> &'a [Value] {
    payload["results"]
        .as_array()
        .expect("results should be array")
}

fn assert_results_prioritize_src(results: &[Value], label: &str) {
    for item in results {
        let path = item["file_path"]
            .as_str()
            .expect("file_path should be string");
        assert!(
            path.starts_with("src/"),
            "expected {label} results to prioritize code paths, got {path}"
        );
    }
}

#[test]
fn milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/code.rs", "// phase30helpertoken\n");
    common::write_file(
        repo.path(),
        "tests/query_focus_test.rs",
        "// phase30helpertoken\n",
    );
    common::write_file(repo.path(), "docs/guide.md", "phase30helpertoken\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "refs",
        "phase30helpertoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("refs json should parse");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    let first = results.first().expect("expected at least one result");
    assert_eq!(first["file_path"], "src/code.rs");
}

#[test]
fn milestone30_find_and_refs_max_results_cap_deterministically() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/alpha.rs", "// phase30captoken\n");
    common::write_file(repo.path(), "src/beta.rs", "// phase30captoken\n");
    common::write_file(repo.path(), "tests/cap_test.rs", "// phase30captoken\n");
    common::write_file(repo.path(), "docs/guide.md", "phase30captoken\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let find_payload = run_query_json_deterministic(
        repo.path(),
        &["find", "phase30captoken", "--max-results", "2"],
    );
    let find_results = query_results(&find_payload);
    assert_eq!(find_results.len(), 2);
    assert_results_prioritize_src(find_results, "capped find");

    let refs_payload = run_query_json_deterministic(
        repo.path(),
        &["refs", "phase30captoken", "--max-results", "2"],
    );
    let refs_results = query_results(&refs_payload);
    assert_eq!(refs_results.len(), 2);
    assert_results_prioritize_src(refs_results, "capped refs");
}

#[test]
fn milestone30_query_caps_compose_with_code_only_and_exclude_tests() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn phase30_ast_focus() {}\n\npub fn wrapper() {\n    phase30_ast_focus();\n}\n",
    );
    common::write_file(
        repo.path(),
        "docs/guide.md",
        "phase30_ast_focus phase30capscope\n",
    );
    common::write_file(
        repo.path(),
        "tests/query_focus_scope_test.rs",
        "phase30_ast_focus phase30capscope\n",
    );
    common::write_file(repo.path(), "src/fallback_a.rs", "// phase30capscope\n");
    common::write_file(repo.path(), "src/fallback_b.rs", "// phase30capscope\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let find_payload = run_query_json(
        repo.path(),
        &[
            "find",
            "phase30_ast_focus",
            "--code-only",
            "--exclude-tests",
            "--max-results",
            "1",
        ],
    );
    let find_results = query_results(&find_payload);
    assert_eq!(find_results.len(), 1);
    assert_eq!(find_results[0]["why_matched"], "ast_definition");
    assert_eq!(find_results[0]["file_path"], "src/lib.rs");

    let refs_payload = run_query_json_deterministic(
        repo.path(),
        &[
            "refs",
            "phase30capscope",
            "--code-only",
            "--exclude-tests",
            "--max-results",
            "1",
        ],
    );
    let refs_results = query_results(&refs_payload);
    assert_eq!(refs_results.len(), 1);
    assert_eq!(refs_results[0]["file_path"], "src/fallback_a.rs");
}
