mod common;

#[test]
fn deps_shows_file_dependencies() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/helper.rs", "pub fn help() {}\n");
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "mod helper;\npub fn main_fn() { helper::help(); }\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "deps",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("depends_on:") || output.contains("src/helper.rs"),
        "should show dependency on helper.rs:\n{output}"
    );
}
