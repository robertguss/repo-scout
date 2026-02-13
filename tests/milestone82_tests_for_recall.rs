mod common;

#[test]
fn tests_for_finds_cli_integration_tests() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.rs",
        "fn main() {}\nfn run_index() { index_repository(); }\nfn index_repository() {}\n",
    );
    common::write_file(
        repo.path(),
        "tests/test_index.rs",
        "use assert_cmd::Command;\n#[test]\nfn test_indexing() {\n    Command::cargo_bin(\"myapp\").unwrap().arg(\"index\").assert().success();\n}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "tests-for",
        "index_repository",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // The test file invokes "index" CLI command, which maps to run_index -> index_repository
    // This is a stretch goal — if the basic heuristic picks it up via text matching, that's fine
    assert!(
        output.contains("test_index") || output.contains("tests/"),
        "should find test that exercises index_repository via CLI:\n{output}"
    );
}
