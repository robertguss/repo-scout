mod common;

use std::fs;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

#[test]
fn milestone6_delete_prunes_rows() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/delete_me.txt",
        "deletable_symbol marker\n",
    );
    common::write_file(repo.path(), "src/keep.txt", "stable_symbol\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let before = run_stdout(&[
        "find",
        "deletable_symbol",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(before.contains("results: 1"));
    assert!(before.contains("src/delete_me.txt"));

    fs::remove_file(repo.path().join("src/delete_me.txt")).expect("delete fixture should succeed");

    let reindex = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        reindex.contains("indexed_files: 0"),
        "no remaining file content changed after deletion"
    );
    assert!(
        reindex.contains("skipped_files: 1"),
        "remaining file should be counted as unchanged"
    );

    let after = run_stdout(&[
        "find",
        "deletable_symbol",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        after.contains("results: 0"),
        "deleted file rows should be pruned from index tables"
    );
    assert!(!after.contains("src/delete_me.txt"));
}

#[test]
fn milestone6_rename_prunes_old_path() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/rename_from.txt",
        "rename_symbol before_path\n",
    );
    common::write_file(repo.path(), "src/stable.txt", "stable_symbol\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let before = run_stdout(&[
        "find",
        "rename_symbol",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(before.contains("src/rename_from.txt"));

    fs::rename(
        repo.path().join("src/rename_from.txt"),
        repo.path().join("src/rename_to.txt"),
    )
    .expect("fixture rename should succeed");

    let reindex = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(reindex.contains("indexed_files: 1"));
    assert!(reindex.contains("skipped_files: 1"));

    let after = run_stdout(&[
        "find",
        "rename_symbol",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(after.contains("results: 1"));
    assert!(after.contains("src/rename_to.txt"));
    assert!(!after.contains("src/rename_from.txt"));
}

#[test]
fn milestone6_lifecycle_counts_are_deterministic() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/a.txt", "alpha marker\n");
    common::write_file(repo.path(), "src/b.txt", "beta marker\n");

    let first = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(first.contains("indexed_files: 2"));
    assert!(first.contains("skipped_files: 0"));

    let second = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(second.contains("indexed_files: 0"));
    assert!(second.contains("skipped_files: 2"));

    fs::remove_file(repo.path().join("src/b.txt")).expect("fixture delete should succeed");

    let third = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(third.contains("indexed_files: 0"));
    assert!(third.contains("skipped_files: 1"));

    let fourth = run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(fourth.contains("indexed_files: 0"));
    assert!(fourth.contains("skipped_files: 1"));
}
