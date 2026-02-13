mod common;

use predicates::str::contains;

#[test]
fn milestone2_second_index_skips_unchanged_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "notes/todo.txt", "orbit\n");

    let mut first_index = common::repo_scout_cmd();
    first_index.arg("index").arg("--repo").arg(repo.path());
    first_index
        .assert()
        .success()
        .stdout(contains("indexed_files: 1"))
        .stdout(contains("non_source_files: 0"));

    let mut second_index = common::repo_scout_cmd();
    second_index.arg("index").arg("--repo").arg(repo.path());
    second_index
        .assert()
        .success()
        .stdout(contains("indexed_files: 0"))
        .stdout(contains("non_source_files: 1"));
}

#[test]
fn milestone2_find_and_refs_use_text_fallback_for_plain_text_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "docs/guide.txt", "orbital checklist\n");

    let mut index_cmd = common::repo_scout_cmd();
    index_cmd.arg("index").arg("--repo").arg(repo.path());
    index_cmd
        .assert()
        .success()
        .stdout(contains("indexed_files: 1"));

    for subcommand in ["find", "refs"] {
        let mut cmd = common::repo_scout_cmd();
        cmd.arg(subcommand)
            .arg("orbital")
            .arg("--repo")
            .arg(repo.path());
        cmd.assert()
            .success()
            .stdout(contains(format!("command: {subcommand}")))
            .stdout(contains("query: orbital"))
            .stdout(contains("results: 1"))
            .stdout(contains("docs/guide.txt"));
    }
}
