mod common;

#[test]
fn callers_shows_who_calls_a_function() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn leaf() {}\npub fn mid() { leaf(); }\npub fn top() { mid(); }\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["callers", "leaf", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("mid"), "mid should call leaf:\n{output}");
}

#[test]
fn callees_shows_what_a_function_calls() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn a() {}\npub fn b() {}\npub fn hub() { a(); b(); }\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["callees", "hub", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("a"), "hub should call a:\n{output}");
    assert!(output.contains("b"), "hub should call b:\n{output}");
}
