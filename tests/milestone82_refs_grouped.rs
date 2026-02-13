mod common;

#[test]
fn refs_terminal_groups_by_category() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn helper() {}\npub fn caller() { helper(); }\n"
    );
    common::write_file(repo.path(), "tests/test_it.rs",
        "fn test_helper() { /* helper */ }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "refs", "helper", "--repo", repo.path().to_str().unwrap(),
    ]);

    // Output should have section headers
    let has_sections = output.contains("Source")
        || output.contains("Test")
        || output.contains("Definitions");
    assert!(
        has_sections,
        "refs output should group results by category:\n{output}"
    );
}
