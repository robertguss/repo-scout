mod common;

use predicates::str::contains;

#[test]
fn milestone81_python_refs_cover_identifier_and_attribute_fallback_paths() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/app.py",
        "def helper():\n    return 1\n\ndef run():\n    helper()\n    obj.render()\n",
    );

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    common::repo_scout_cmd()
        .args(["refs", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_reference ast_likely]"));
    common::repo_scout_cmd()
        .args(["refs", "render", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_reference ast_likely]"));
}

#[test]
fn milestone81_typescript_refs_cover_identifier_and_member_expression_fallback_paths() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/app.ts",
        "export function helper(): void {}\n\nexport function run(): void {\n  helper();\n  obj.render();\n}\n",
    );

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    common::repo_scout_cmd()
        .args(["refs", "helper", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_reference ast_likely]"));
    common::repo_scout_cmd()
        .args(["refs", "render", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("[ast_reference ast_likely]"));
}

#[test]
fn milestone81_go_index_handles_malformed_selector_without_crashing() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.go",
        "package main\n\nfunc run() {\n    alias.\n}\n",
    );

    common::repo_scout_cmd()
        .args(["index", "--repo", repo.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(contains("indexed_files:"));
}
