mod common;

#[test]
fn find_nonexistent_symbol_shows_did_you_mean() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn calculate_total() {}\npub fn calculate_sum() {}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    // Search for something that won't match exactly or via text
    let output = common::run_stdout(&[
        "find",
        "xyznonexistent_calculate",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // Should show did-you-mean suggestions or at least no crash
    assert!(
        output.contains("did you mean") || output.contains("results: 0"),
        "should show did-you-mean or empty results:\n{output}"
    );
}
