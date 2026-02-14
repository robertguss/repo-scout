mod common;
use std::time::Duration;

#[test]
fn milestone111_exit_codes_follow_agent_contract() {
    let mut usage = common::repo_scout_cmd();
    usage.arg("--definitely-invalid-flag");
    usage.assert().code(2);

    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn stale_target() {}\n");
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    std::thread::sleep(Duration::from_secs(1));
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn stale_target() { let _x = 1; }\n",
    );

    let mut stale = common::repo_scout_cmd();
    stale.args([
        "find",
        "stale_target",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
        "--require-index-fresh",
    ]);
    stale.assert().code(3);

    let mut internal = common::repo_scout_cmd();
    internal.args([
        "query",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--input",
        "tests/fixtures/phase20/does-not-exist.jsonl",
    ]);
    internal.assert().code(4);

    let mut partial = common::repo_scout_cmd();
    partial.args([
        "verify-refactor",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--before",
        "snapshot-a",
        "--after",
        "snapshot-a",
        "--strict",
    ]);
    partial.assert().code(5);
}
