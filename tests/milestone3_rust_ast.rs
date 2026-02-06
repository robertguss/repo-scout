mod common;

use predicates::str::contains;

#[test]
fn milestone3_find_reports_rust_ast_definition_match() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "fn launch() {}\n\nfn run() {\n    launch();\n}\n",
    );

    let mut index_cmd = common::repo_scout_cmd();
    index_cmd.arg("index").arg("--repo").arg(repo.path());
    index_cmd.assert().success();

    let mut find_cmd = common::repo_scout_cmd();
    find_cmd
        .arg("find")
        .arg("launch")
        .arg("--repo")
        .arg(repo.path());
    find_cmd
        .assert()
        .success()
        .stdout(contains("command: find"))
        .stdout(contains("query: launch"))
        .stdout(contains("results: 1"))
        .stdout(contains("src/lib.rs"))
        .stdout(contains("[ast_definition ast_exact]"));
}

#[test]
fn milestone3_refs_reports_rust_ast_reference_match() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "fn launch() {}\n\nfn run() {\n    launch();\n}\n",
    );

    let mut index_cmd = common::repo_scout_cmd();
    index_cmd.arg("index").arg("--repo").arg(repo.path());
    index_cmd.assert().success();

    let mut refs_cmd = common::repo_scout_cmd();
    refs_cmd
        .arg("refs")
        .arg("launch")
        .arg("--repo")
        .arg(repo.path());
    refs_cmd
        .assert()
        .success()
        .stdout(contains("command: refs"))
        .stdout(contains("query: launch"))
        .stdout(contains("results: 1"))
        .stdout(contains("src/lib.rs"))
        .stdout(contains("[ast_reference ast_likely]"));
}
