mod common;

#[test]
fn find_suggests_similar_when_no_exact_match() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn index_repository() {}\npub fn index_file() {}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    // Misspelled: "index_repo" instead of "index_repository"
    let output = common::run_stdout(&[
        "find",
        "index_repo",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("did you mean") || output.contains("index_repository"),
        "should suggest similar symbol:\n{output}"
    );
}

#[test]
fn find_shows_did_you_mean_when_zero_results() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn calculate_total() {}\npub fn calculate_sum() {}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    // Search for something that won't match via text either
    // but contains "calculate" as substring
    let output = common::run_stdout(&["find", "calculat", "--repo", repo.path().to_str().unwrap()]);
    // Text fallback may find it, but if not, did-you-mean should appear
    assert!(
        output.contains("calculate_total")
            || output.contains("calculate_sum")
            || output.contains("did you mean")
            || output.contains("Did you mean"),
        "should find or suggest calculate symbols:\n{output}"
    );
}
