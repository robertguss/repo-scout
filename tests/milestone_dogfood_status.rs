mod common;

#[test]
fn status_shows_enriched_output() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\npub fn world() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);

    assert!(output.contains("source_files:"), "missing source_files count:\n{output}");
    assert!(output.contains("definitions:"), "missing definitions count:\n{output}");
    assert!(output.contains("references:"), "missing references count:\n{output}");
    assert!(output.contains("edges:"), "missing edges count:\n{output}");
}
