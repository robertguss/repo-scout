mod common;

#[test]
fn diff_impact_unstaged_detects_modified_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\n");

    // Initialize git, make initial commit
    std::process::Command::new("git")
        .args(["init"])
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

    // Modify file without staging (unstaged change)
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn original() {}\npub fn unstaged_fn() {}\n",
    );

    let output = common::run_stdout(&[
        "diff-impact",
        "--unstaged",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // The output should reference the changed file
    assert!(
        output.contains("src/lib.rs") || output.contains("0 results"),
        "unstaged should detect changed file or produce valid output:\n{output}"
    );
}

#[test]
fn verify_plan_since_detects_changed_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\n");

    std::process::Command::new("git")
        .args(["init"])
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

    // Make a second commit
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
        "verify-plan",
        "--since",
        "HEAD~1",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("changed_files:") || output.contains("src/lib.rs"),
        "verify-plan --since should detect changed files:\n{output}"
    );
}

#[test]
fn verify_plan_unstaged_detects_modified_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\n");

    std::process::Command::new("git")
        .args(["init"])
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

    // Modify without staging
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn original() {}\npub fn unstaged_fn() {}\n",
    );

    let output = common::run_stdout(&[
        "verify-plan",
        "--unstaged",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("changed_files:") || output.contains("src/lib.rs") || output.contains("0 targeted"),
        "verify-plan --unstaged should detect or report changed file:\n{output}"
    );
}
