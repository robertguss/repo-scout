mod common;

use predicates::str::contains;

#[test]
fn harness_can_run_binary_and_create_fixture_files() {
    let repo = common::temp_repo();
    let created = common::write_file(repo.path(), "src/sample.txt", "hello");
    assert!(created.exists());

    let mut cmd = common::repo_scout_cmd();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage:"))
        .stdout(contains("index"));
}
