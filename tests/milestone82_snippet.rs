mod common;

use serde_json::Value;

#[test]
fn snippet_returns_function_source() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n\npub fn farewell() -> &'static str {\n    \"bye\"\n}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["snippet", "greet", "--repo", repo.path().to_str().unwrap()]);

    assert!(
        output.contains("pub fn greet"),
        "should contain function signature:\n{output}"
    );
    assert!(
        output.contains("Hello, {name}!"),
        "should contain function body:\n{output}"
    );
    assert!(
        !output.contains("farewell"),
        "should NOT contain other functions:\n{output}"
    );
}

#[test]
fn snippet_json_output() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "snippet",
        "add",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    assert_eq!(payload["command"], "snippet");
    assert!(
        payload["results"][0]["snippet"]
            .as_str()
            .unwrap()
            .contains("a + b")
    );
}

#[test]
fn snippet_with_context_lines() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "// This is a greeting function\npub fn greet() -> String {\n    \"hi\".to_string()\n}\n// End of greet\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "snippet",
        "greet",
        "--repo",
        repo.path().to_str().unwrap(),
        "--context",
        "1",
    ]);
    assert!(
        output.contains("greeting function"),
        "should contain 1 context line above:\n{output}"
    );
}
