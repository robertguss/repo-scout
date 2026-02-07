mod common;

use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone13_explain_definition_summary() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/validation/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let terminal_out = run_stdout(&[
        "explain",
        "compute_plan",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(terminal_out.contains("command: explain"));
    assert!(terminal_out.contains("query: compute_plan"));
    assert!(terminal_out.contains("src/lib.rs"));
    assert!(terminal_out.contains("signature: pub fn compute_plan()"));
}

#[test]
fn milestone13_explain_relationship_summary() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn root() {\n    leaf();\n}\n\npub fn leaf() {}\n\npub fn caller() {\n    root();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let terminal_out = run_stdout(&["explain", "root", "--repo", repo.path().to_str().unwrap()]);
    assert!(terminal_out.contains("inbound: called_by=1"));
    assert!(terminal_out.contains("outbound: calls=1"));

    let json_out = run_stdout(&[
        "explain",
        "root",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value =
        serde_json::from_str(&json_out).expect("explain --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert_eq!(results[0]["inbound"]["called_by"], 1);
    assert_eq!(results[0]["outbound"]["calls"], 1);
}

#[test]
fn milestone13_explain_json_determinism() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn root() {\n    leaf();\n}\n\npub fn leaf() {}\n\npub fn caller() {\n    root();\n}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out_a = run_stdout(&[
        "explain",
        "root",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
        "--include-snippets",
    ]);
    let out_b = run_stdout(&[
        "explain",
        "root",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
        "--include-snippets",
    ]);
    assert_eq!(out_a, out_b);

    let payload: Value =
        serde_json::from_str(&out_a).expect("explain --json should produce valid json");
    assert_eq!(payload["schema_version"], 3);
    assert_eq!(payload["command"], "explain");
    assert_eq!(payload["query"], "root");
    assert_eq!(payload["include_snippets"], true);

    let results = payload["results"]
        .as_array()
        .expect("results should be array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["language"], "rust");
    assert_eq!(results[0]["confidence"], "graph_exact");
    assert_eq!(results[0]["provenance"], "ast_definition");
    let snippet = results[0]["snippet"]
        .as_str()
        .expect("snippet should be present with include-snippets=true");
    assert!(snippet.contains("leaf();"));
}
