mod common;

use predicates::str::contains;

#[test]
fn milestone1_index_creates_db_and_prints_schema_version() {
    let repo = common::temp_repo();
    let db_path = repo.path().join(".repo-scout").join("index.db");

    let mut cmd = common::repo_scout_cmd();
    cmd.arg("index").arg("--repo").arg(repo.path());
    cmd.assert()
        .success()
        .stdout(contains("index_path: "))
        .stdout(contains("schema_version: 3"));

    assert!(
        db_path.exists(),
        "index command should create sqlite database"
    );
}

#[test]
fn milestone1_status_reports_schema_after_index_bootstrap() {
    let repo = common::temp_repo();

    let mut index_cmd = common::repo_scout_cmd();
    index_cmd.arg("index").arg("--repo").arg(repo.path());
    index_cmd.assert().success();

    let mut status_cmd = common::repo_scout_cmd();
    status_cmd.arg("status").arg("--repo").arg(repo.path());
    status_cmd
        .assert()
        .success()
        .stdout(contains("index_path: "))
        .stdout(contains("schema_version: 3"));
}

#[test]
fn milestone1_find_and_refs_accept_symbol_queries() {
    let repo = common::temp_repo();

    for subcommand in ["find", "refs"] {
        let mut cmd = common::repo_scout_cmd();
        cmd.arg(subcommand)
            .arg("main")
            .arg("--repo")
            .arg(repo.path());
        cmd.assert()
            .success()
            .stdout(contains(format!("command: {subcommand}")))
            .stdout(contains("query: main"))
            .stdout(contains("results: 0"));
    }
}
