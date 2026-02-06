mod common;

use assert_cmd::prelude::*;
use serde_json::Value;

/// Run the repo-scout command with the provided arguments and return its standard output as a UTF-8 string.
///
/// Panics if the command's stdout is not valid UTF-8.
///
/// # Examples
///
/// ```
/// let out = run_stdout(&["index", "--help"]);
/// assert!(out.contains("Usage") || out.contains("help"));
/// ```
fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = OutputAssertExt::assert(&mut cmd)
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

/// Verifies the terminal output of the `impact validate` agent query against a freshly indexed repo.
///
/// Creates a temporary repository with the phase2 graph fixture, runs indexing, executes
/// `impact validate`, and asserts the command's stdout contains the expected markers:
/// `"command: impact"`, `"query: validate"`, `"called_by"`, and `"prepare"`.
///
/// # Examples
///
/// ```
/// // This is an integration test; running the test harness verifies the assertions below.
/// assert!(true);
/// ```
#[test]
fn milestone9_impact_terminal() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/graph/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "impact",
        "validate",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(out.contains("command: impact"));
    assert!(out.contains("query: validate"));
    assert!(out.contains("called_by"));
    assert!(out.contains("prepare"));
}

#[test]
fn milestone9_impact_json_schema() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/graph/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "impact",
        "persist",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("impact --json should be valid json");

    assert_eq!(payload["schema_version"], 2);
    assert_eq!(payload["command"], "impact");
    assert_eq!(payload["query"], "persist");
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert_eq!(results[0]["relationship"], "called_by");
    assert_eq!(results[0]["confidence"], "graph_likely");
}

/// Verifies that the CLI `context` command returns budgeted results on the terminal.
///
/// Sets up a temporary repository with the `rust_symbols` fixture, runs indexing, invokes
/// `context` with a specific task and a budget of 200, and asserts the terminal output
/// includes the command name, the budget value, a results count, and a `why:` explanation.
///
/// # Examples
///
/// ```
/// // Integration test that exercises the CLI against a temporary repository.
/// // See `milestone9_context_budgeted_terminal` for the full test.
—```
#[test]
fn milestone9_context_budgeted_terminal() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "context",
        "--task",
        "modify start_engine and update callers",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "200",
    ]);

    assert!(out.contains("command: context"));
    assert!(out.contains("budget: 200"));
    assert!(out.contains("results: 1"));
    assert!(out.contains("why:"));
}

/// Verifies the JSON schema and essential fields produced by `context --json` for a sample repository.
///
/// # Examples
///
/// ```
/// let out = r#"{
///   "schema_version": 2,
///   "command": "context",
///   "budget": 400,
///   "results": [ { "why_included": "reason", "confidence": "high" } ]
/// }"#;
/// let payload: serde_json::Value = serde_json::from_str(out).unwrap();
/// assert_eq!(payload["schema_version"], 2);
/// assert_eq!(payload["command"], "context");
/// assert_eq!(payload["budget"], 400);
/// let results = payload["results"].as_array().unwrap();
/// assert!(!results.is_empty());
/// assert!(results[0]["why_included"].is_string());
/// assert!(results[0]["confidence"].is_string());
/// ```
#[test]
fn milestone9_context_json_schema() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        include_str!("fixtures/phase2/rust_symbols/src/lib.rs"),
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out = run_stdout(&[
        "context",
        "--task",
        "modify start_engine and update callers",
        "--repo",
        repo.path().to_str().unwrap(),
        "--budget",
        "400",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&out).expect("context --json should be valid json");

    assert_eq!(payload["schema_version"], 2);
    assert_eq!(payload["command"], "context");
    assert_eq!(payload["budget"], 400);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(!results.is_empty());
    assert!(results[0]["why_included"].is_string());
    assert!(results[0]["confidence"].is_string());
}