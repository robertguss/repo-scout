mod common;

fn setup_hardening_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn target(a: i32) -> i32 {
    let b = a + 1;
    let c = b * 2;
    c
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
fn milestone100_extract_check_rejects_invalid_line_spec() {
    let repo = setup_hardening_repo();
    let mut cmd = common::repo_scout_cmd();
    cmd.args([
        "extract-check",
        "target",
        "--lines",
        "abc",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    cmd.assert().failure();
}

#[test]
fn milestone100_extract_check_rejects_out_of_bounds_range() {
    let repo = setup_hardening_repo();
    let mut cmd = common::repo_scout_cmd();
    cmd.args([
        "extract-check",
        "target",
        "--lines",
        "1-2",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    cmd.assert().failure();
}

#[test]
fn milestone100_extract_check_json_includes_symbol_and_bounds() {
    let repo = setup_hardening_repo();
    let out = common::run_stdout(&[
        "extract-check",
        "target",
        "--lines",
        "3-4",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--json",
    ]);
    assert!(
        out.contains("\"command\": \"extract-check\""),
        "output:\n{out}"
    );
    assert!(out.contains("\"symbol\": \"target\""), "output:\n{out}");
    assert!(out.contains("\"extract_start_line\": 3"), "output:\n{out}");
    assert!(out.contains("\"extract_end_line\": 4"), "output:\n{out}");
}

#[test]
fn milestone100_verify_refactor_strict_fails_when_before_after_differ() {
    let repo = setup_hardening_repo();
    let mut cmd = common::repo_scout_cmd();
    cmd.args([
        "verify-refactor",
        "--before",
        "a",
        "--after",
        "b",
        "--strict",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    cmd.assert().failure();
}
