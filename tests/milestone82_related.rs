mod common;

#[test]
fn related_shows_siblings_and_shared_callers() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn sibling_a() {}\npub fn sibling_b() {}\npub fn user() { sibling_a(); sibling_b(); }\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "related",
        "sibling_a",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("sibling_b"),
        "sibling_b should be related to sibling_a:\n{output}"
    );
}
