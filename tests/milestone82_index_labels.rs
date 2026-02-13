mod common;

#[test]
fn index_output_uses_non_source_files_label() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::write_file(repo.path(), "README.md", "# Hello\n");
    let output = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        output.contains("non_source_files:"),
        "should use 'non_source_files' label, got:\n{output}"
    );
    assert!(
        !output.contains("skipped_files:"),
        "should NOT use 'skipped_files' label, got:\n{output}"
    );
}
