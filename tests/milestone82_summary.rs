mod common;

#[test]
fn summary_shows_repo_overview() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn hello() {}\npub struct Foo {}\n",
    );
    common::write_file(repo.path(), "src/main.rs", "fn main() { hello(); }\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["summary", "--repo", repo.path().to_str().unwrap()]);

    assert!(
        output.contains("source_files:"),
        "missing file count:\n{output}"
    );
    assert!(
        output.contains("definitions:"),
        "missing definition count:\n{output}"
    );
    assert!(output.contains("edges:"), "missing edge count:\n{output}");
    assert!(
        output.contains("rust"),
        "missing language breakdown:\n{output}"
    );
}
