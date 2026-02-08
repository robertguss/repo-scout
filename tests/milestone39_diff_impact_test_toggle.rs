mod common;

use predicates::str::contains;
use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn build_diff_impact_toggle_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        "pub fn changed_entry() {\n    helper();\n}\n\npub fn helper() {}\n",
    );
    common::write_file(
        repo,
        "src/other.rs",
        "pub fn watcher() {\n    changed_entry();\n}\n",
    );
    common::write_file(
        repo,
        "tests/impact_test.rs",
        "#[test]\nfn covers_changed_entry() {\n    changed_entry();\n    changed_entry();\n}\n",
    );
}

#[test]
fn milestone39_diff_impact_exclude_tests_omits_test_targets() {
    let repo = common::temp_repo();
    build_diff_impact_toggle_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--exclude-tests",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&output).expect("diff-impact json should parse");

    assert_eq!(payload["include_tests"], false);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");
    assert!(
        !results
            .iter()
            .any(|row| row["result_kind"] == "test_target"),
        "--exclude-tests should remove test_target rows"
    );
}

#[test]
fn milestone39_diff_impact_default_and_include_tests_keep_test_targets() {
    let repo = common::temp_repo();
    build_diff_impact_toggle_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let default_output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let explicit_output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--include-tests",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let default_payload: Value =
        serde_json::from_str(&default_output).expect("default diff-impact json should parse");
    let explicit_payload: Value = serde_json::from_str(&explicit_output)
        .expect("explicit include diff-impact json should parse");

    assert_eq!(default_payload["include_tests"], true);
    assert_eq!(explicit_payload["include_tests"], true);

    let default_results = default_payload["results"]
        .as_array()
        .expect("default results should be an array");
    let explicit_results = explicit_payload["results"]
        .as_array()
        .expect("explicit results should be an array");

    assert!(
        default_results
            .iter()
            .any(|row| row["result_kind"] == "test_target"),
        "default diff-impact output should keep test_target rows"
    );
    assert!(
        explicit_results
            .iter()
            .any(|row| row["result_kind"] == "test_target"),
        "--include-tests should keep test_target rows"
    );
    assert_eq!(default_payload, explicit_payload);
}

#[test]
fn milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit() {
    let repo = common::temp_repo();
    build_diff_impact_toggle_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    common::repo_scout_cmd()
        .args([
            "diff-impact",
            "--changed-file",
            "src/lib.rs",
            "--include-tests",
            "--exclude-tests",
            "--repo",
            repo.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}
