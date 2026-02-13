mod common;

#[test]
fn path_finds_call_chain() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn a() { b(); }\npub fn b() { c(); }\npub fn c() {}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "call-path", "a", "c", "--repo", repo.path().to_str().unwrap(),
    ]);
    assert!(output.contains("a"), "path should include start:\n{output}");
    assert!(output.contains("b"), "path should include intermediate:\n{output}");
    assert!(output.contains("c"), "path should include end:\n{output}");
}
