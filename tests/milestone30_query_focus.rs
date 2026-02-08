mod common;

use common::run_stdout;
use serde_json::Value;

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

    let find_out = run_stdout(&[
        "find",
        "phase30captoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-results",
        "2",
        "--json",
    ]);
    let find_again_out = run_stdout(&[
        "find",
        "phase30captoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-results",
        "2",
        "--json",
    ]);
    assert_eq!(find_out, find_again_out);
    let find_payload: Value = serde_json::from_str(&find_out).expect("find json should parse");
    let find_results = find_payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(find_results.len(), 2);
    for item in find_results {
        let path = item["file_path"]
            .as_str()
            .expect("file_path should be string");
        assert!(
            path.starts_with("src/"),
            "expected capped find results to prioritize code paths, got {path}"
        );
    }

    let refs_out = run_stdout(&[
        "refs",
        "phase30captoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-results",
        "2",
        "--json",
    ]);
    let refs_again_out = run_stdout(&[
        "refs",
        "phase30captoken",
        "--repo",
        repo.path().to_str().unwrap(),
        "--max-results",
        "2",
        "--json",
    ]);
    assert_eq!(refs_out, refs_again_out);
    let refs_payload: Value = serde_json::from_str(&refs_out).expect("refs json should parse");
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(refs_results.len(), 2);
    for item in refs_results {
        let path = item["file_path"]
            .as_str()
            .expect("file_path should be string");
        assert!(
            path.starts_with("src/"),
            "expected capped refs results to prioritize code paths, got {path}"
        );
    }
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

    let find_out = run_stdout(&[
        "find",
        "phase30_ast_focus",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--max-results",
        "1",
        "--json",
    ]);
    let find_payload: Value = serde_json::from_str(&find_out).expect("find json should parse");
    let find_results = find_payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(find_results.len(), 1);
    assert_eq!(find_results[0]["why_matched"], "ast_definition");
    assert_eq!(find_results[0]["file_path"], "src/lib.rs");

    let refs_out = run_stdout(&[
        "refs",
        "phase30capscope",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--max-results",
        "1",
        "--json",
    ]);
    let refs_again_out = run_stdout(&[
        "refs",
        "phase30capscope",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--max-results",
        "1",
        "--json",
    ]);
    assert_eq!(refs_out, refs_again_out);
    let refs_payload: Value = serde_json::from_str(&refs_out).expect("refs json should parse");
    let refs_results = refs_payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(refs_results.len(), 1);
    assert_eq!(refs_results[0]["file_path"], "src/fallback_a.rs");
}
