mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be valid utf-8")
}

#[test]
fn milestone4_find_json_schema_and_exact_name_first_ranking() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "docs/rank.txt", "orbit orbital\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let stdout = run_stdout(&[
        "find",
        "orbit",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let json: Value = serde_json::from_str(&stdout).expect("find --json should output valid json");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "find");
    assert_eq!(json["query"], "orbit");

    let results = json["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["symbol"], "orbit");
    assert_eq!(results[0]["why_matched"], "exact_symbol_name");
    assert_eq!(results[0]["confidence"], "text_fallback");
    assert_eq!(results[1]["symbol"], "orbital");
    assert_eq!(results[1]["why_matched"], "text_substring_match");

    let first_score = results[0]["score"]
        .as_f64()
        .expect("score should be numeric");
    let second_score = results[1]["score"]
        .as_f64()
        .expect("score should be numeric");
    assert!(
        first_score > second_score,
        "exact match should outrank substring"
    );
}

#[test]
fn milestone4_find_json_output_is_deterministic_across_runs() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "docs/a.txt", "orbit orbital\n");
    common::write_file(repo.path(), "docs/b.txt", "orbital orbit\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let first = run_stdout(&[
        "find",
        "orbit",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let second = run_stdout(&[
        "find",
        "orbit",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    assert_eq!(first, second, "json output should be stable across runs");
}

#[test]
fn milestone4_refs_json_preserves_ast_labels() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "fn launch() {}\n\nfn run() {\n    launch();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let stdout = run_stdout(&[
        "refs",
        "launch",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let json: Value = serde_json::from_str(&stdout).expect("refs --json should output valid json");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "refs");
    assert_eq!(json["query"], "launch");

    let results = json["results"]
        .as_array()
        .expect("results should be an array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["symbol"], "launch");
    assert_eq!(results[0]["why_matched"], "ast_reference");
    assert_eq!(results[0]["confidence"], "ast_likely");
}
