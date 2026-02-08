mod common;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn build_terminal_fixture(repo: &std::path::Path) {
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
        "#[test]\nfn covers_changed_entry() {\n    changed_entry();\n}\n",
    );
}

#[test]
fn milestone40_diff_impact_terminal_lists_impacted_symbol_rows() {
    let repo = common::temp_repo();
    build_terminal_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    let changed_line = output
        .lines()
        .find(|line| {
            line.starts_with("impacted_symbol ")
                && line.contains(" changed_entry ")
                && line.contains("relationship=changed_symbol")
        })
        .expect("terminal output should include a changed_entry impacted_symbol row");

    assert!(changed_line.contains("src/lib.rs:"));
    assert!(changed_line.contains("confidence="));
    assert!(changed_line.contains("score="));
}

#[test]
fn milestone40_diff_impact_terminal_lists_test_target_rows_conditionally() {
    let repo = common::temp_repo();
    build_terminal_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let with_tests = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    let without_tests = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--exclude-tests",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    let test_target_line = with_tests
        .lines()
        .find(|line| line.starts_with("test_target "))
        .expect("terminal output should include test_target rows by default");

    assert!(test_target_line.contains("tests/impact_test.rs"));
    assert!(test_target_line.contains("integration_test_file"));
    assert!(test_target_line.contains("confidence="));
    assert!(test_target_line.contains("score="));

    assert!(
        !without_tests
            .lines()
            .any(|line| line.starts_with("test_target ")),
        "--exclude-tests should omit terminal test_target rows"
    );
}

#[test]
fn milestone40_diff_impact_terminal_output_is_deterministic() {
    let repo = common::temp_repo();
    build_terminal_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let out_a = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    let out_b = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert_eq!(out_a, out_b);
    assert!(
        out_a
            .lines()
            .any(|line| line.starts_with("impacted_symbol ")),
        "determinism check requires row-oriented impacted_symbol output"
    );
}
