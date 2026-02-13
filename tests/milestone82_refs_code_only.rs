mod common;

#[test]
fn refs_code_only_excludes_non_source_files() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn helper() {}\npub fn caller() { helper(); }\n",
    );
    common::write_file(
        repo.path(),
        "docs/notes.md",
        "We use helper() in the codebase.\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let all = common::run_stdout(&["refs", "helper", "--repo", repo.path().to_str().unwrap()]);
    let code_only = common::run_stdout(&[
        "refs",
        "helper",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
    ]);

    // code_only should have fewer or equal results (no .md files)
    let all_count: usize = all.lines().filter(|l| l.contains("helper")).count();
    let code_count: usize = code_only.lines().filter(|l| l.contains("helper")).count();
    assert!(
        code_count <= all_count,
        "code-only should not have more results than all"
    );
    // Verify no .md files in code_only output
    for line in code_only.lines() {
        assert!(
            !line.ends_with(".md") && !line.contains(".md:"),
            "code-only should not contain .md files: {line}"
        );
    }
}
