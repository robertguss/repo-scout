mod common;

use serde_json::Value;

#[test]
fn milestone118_query_batch_jsonl_returns_per_request_status() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn run_find() {}\n");
    let input = common::write_file(
        repo.path(),
        "query_batch.jsonl",
        "{\"id\":\"1\",\"command\":\"find\",\"symbol\":\"run_find\"}\n{\"id\":\"2\",\"command\":\"unknown\",\"symbol\":\"run_find\"}\n",
    );

    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);

    let out = common::run_stdout(&[
        "query",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--format",
        "jsonl",
        "--input",
        input.to_str().expect("input path utf-8"),
    ]);

    let lines: Vec<&str> = out.lines().filter(|line| !line.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "expected one response per request");

    let first: Value = serde_json::from_str(lines[0]).expect("first response json");
    assert_eq!(first["id"], "1");
    assert_eq!(first["ok"], true);

    let second: Value = serde_json::from_str(lines[1]).expect("second response json");
    assert_eq!(second["id"], "2");
    assert_eq!(second["ok"], false);
}
