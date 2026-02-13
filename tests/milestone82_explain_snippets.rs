mod common;

use serde_json::Value;

#[test]
fn explain_include_snippets_terminal_shows_snippet() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n",
    );
    let _index_out = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let terminal_out = common::run_stdout(&[
        "explain",
        "greet",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-snippets",
    ]);
    assert!(
        terminal_out.contains("snippet:"),
        "terminal output should contain 'snippet:' label, got:\n{terminal_out}"
    );
    assert!(
        terminal_out.contains("pub fn greet"),
        "terminal output should contain the function source, got:\n{terminal_out}"
    );
}

#[test]
fn explain_include_snippets_terminal_matches_json_snippet() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    let _index_out = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let terminal_out = common::run_stdout(&[
        "explain",
        "add",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-snippets",
    ]);
    let json_out = common::run_stdout(&[
        "explain",
        "add",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-snippets",
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    let json_snippet = payload["results"][0]["snippet"]
        .as_str()
        .expect("snippet in json");

    // Terminal should contain the same snippet content
    assert!(
        terminal_out.contains(json_snippet.lines().next().unwrap()),
        "terminal should contain first line of JSON snippet"
    );
}
