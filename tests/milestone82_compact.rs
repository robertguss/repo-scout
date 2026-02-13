mod common;

#[test]
fn find_compact_output_is_minimal() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "find",
        "hello",
        "--repo",
        repo.path().to_str().unwrap(),
        "--compact",
    ]);
    // Compact: just file:line symbol, no headers, no metadata
    assert!(
        !output.contains("command:"),
        "compact should not have command header:\n{output}"
    );
    assert!(
        !output.contains("results:"),
        "compact should not have results header:\n{output}"
    );
    assert!(
        output.contains("src/lib.rs:"),
        "should still show file:line:\n{output}"
    );
}
