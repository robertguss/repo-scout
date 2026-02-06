mod common;

use std::fs;

use serde_json::Value;

fn run_success_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be valid utf-8")
}

fn run_failure_stderr(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().failure().get_output().stderr.clone();
    String::from_utf8(output).expect("stderr should be valid utf-8")
}

#[test]
fn milestone5_end_to_end_flow_and_incremental_reindex_behavior() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "fn launch() {}\n\nfn run() {\n    launch();\n}\n",
    );
    common::write_file(repo.path(), "docs/search.txt", "orbit orbital\n");

    let first_index = run_success_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        first_index.contains("indexed_files: 2"),
        "first index should process both fixture files"
    );
    assert!(
        first_index.contains("skipped_files: 0"),
        "first index should not skip files"
    );

    let find_launch =
        run_success_stdout(&["find", "launch", "--repo", repo.path().to_str().unwrap()]);
    assert!(find_launch.contains("[ast_definition ast_exact]"));

    let refs_launch_json = run_success_stdout(&[
        "refs",
        "launch",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let refs_json: Value =
        serde_json::from_str(&refs_launch_json).expect("refs --json should output valid json");
    assert_eq!(refs_json["schema_version"], 1);
    assert_eq!(refs_json["command"], "refs");
    assert_eq!(refs_json["query"], "launch");
    assert_eq!(refs_json["results"][0]["why_matched"], "ast_reference");
    assert_eq!(refs_json["results"][0]["confidence"], "ast_likely");

    let second_index = run_success_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        second_index.contains("indexed_files: 0"),
        "second index should skip unchanged files"
    );
    assert!(
        second_index.contains("skipped_files: 2"),
        "second index should skip both files"
    );

    common::write_file(repo.path(), "docs/search.txt", "orbit orbital comet\n");
    let third_index = run_success_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        third_index.contains("indexed_files: 1"),
        "only modified file should be re-indexed"
    );
    assert!(
        third_index.contains("skipped_files: 1"),
        "unchanged file should still be skipped"
    );
}

#[test]
fn milestone5_corrupt_index_reports_recovery_hint_and_recovers_after_delete() {
    let repo = common::temp_repo();
    let index_dir = repo.path().join(".repo-scout");
    fs::create_dir_all(&index_dir).expect("index directory should be created");
    let db_path = index_dir.join("index.db");
    fs::write(&db_path, "not a sqlite database").expect("corrupt db fixture should be written");

    let stderr = run_failure_stderr(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        stderr.contains("index database appears corrupted"),
        "failure should include a clear corruption hint"
    );
    assert!(
        stderr.contains("delete"),
        "failure should tell users how to recover"
    );
    assert!(
        stderr.contains(db_path.to_string_lossy().as_ref()),
        "failure should mention the corrupt index path"
    );

    fs::remove_file(&db_path).expect("corrupt db file should be removable");
    let recovered = run_success_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        recovered.contains("schema_version: 1"),
        "reindex should succeed after deleting corrupt index file"
    );
}
