mod common;

use serde_json::Value;

fn setup_full_refactor_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn public_api() -> i32 {
    helper() + 1
}

fn helper() -> i32 {
    41
}
"#,
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        r#"
#[test]
fn public_api_works() {
    assert_eq!(42, crate::lib::public_api());
}
"#,
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone99_move_check_and_rename_check_render_reports() {
    let repo = setup_full_refactor_repo();
    let move_out = common::run_stdout(&[
        "move-check",
        "public_api",
        "--to",
        "src/new_lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        move_out.contains("Move check for public_api"),
        "output:\n{move_out}"
    );

    let rename_out = common::run_stdout(&[
        "rename-check",
        "public_api",
        "--to",
        "public_api_v2",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        rename_out.contains("Rename check for public_api -> public_api_v2"),
        "output:\n{rename_out}"
    );
}

#[test]
fn milestone99_split_check_test_scaffold_and_safe_steps_work() {
    let repo = setup_full_refactor_repo();
    let split_out = common::run_stdout(&[
        "split-check",
        "src/lib.rs",
        "--auto",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        split_out.contains("Split check for src/lib.rs"),
        "output:\n{split_out}"
    );

    let scaffold_json = common::run_stdout(&[
        "test-scaffold",
        "public_api",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    let scaffold: Value = serde_json::from_str(&scaffold_json).expect("json");
    assert_eq!(scaffold["command"], "test-scaffold");

    let safe_steps = common::run_stdout(&[
        "safe-steps",
        "public_api",
        "--action",
        "rename",
        "--to",
        "public_api_v2",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(safe_steps.contains("Step 1"), "output:\n{safe_steps}");
}

#[test]
fn milestone99_verify_refactor_and_health_diff_commands_work() {
    let repo = setup_full_refactor_repo();

    let verify_out = common::run_stdout(&[
        "verify-refactor",
        "--before",
        "snapshot-a",
        "--after",
        "snapshot-a",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    assert!(
        verify_out.contains("Refactoring verification"),
        "output:\n{verify_out}"
    );

    let baseline_out = common::run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--save-baseline",
    ]);
    assert!(
        baseline_out.contains("Code health report"),
        "output:\n{baseline_out}"
    );

    let diff_out = common::run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--diff",
    ]);
    assert!(
        diff_out.contains("Health comparison"),
        "output:\n{diff_out}"
    );
}
