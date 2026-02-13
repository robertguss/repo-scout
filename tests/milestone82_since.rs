mod common;

#[test]
fn diff_impact_since_head_detects_changed_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\n");

    // Initialize git, make initial commit
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "CI"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "ci@example.com"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    // Make a change and commit
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn original() {}\npub fn added() {}\n",
    );
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "add function"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = common::run_stdout(&[
        "diff-impact",
        "--since",
        "HEAD~1",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("src/lib.rs"),
        "should detect changed file:\n{output}"
    );
}
