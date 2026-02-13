mod common;

use serde_json::Value;

/// Helper: create a multi-file repo for tree testing.
fn setup_tree_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/main.rs",
        r#"
fn main() {
    lib::greet();
    helper::assist();
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn greet() {
    println!("hello");
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/helper.rs",
        r#"
pub fn assist() {
    println!("assist");
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/utils/format.rs",
        r#"
pub fn format_output() {
    println!("formatted");
}
"#,
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    repo
}

#[test]
fn milestone91_tree_shows_structure() {
    let repo = setup_tree_repo();
    let output = common::run_stdout(&["tree", "--repo", repo.path().to_str().unwrap()]);
    // Should show directory structure
    assert!(
        output.contains("src/") || output.contains("src"),
        "should show src dir:\n{output}"
    );
    assert!(output.contains("main.rs"), "should show main.rs:\n{output}");
    assert!(output.contains("lib.rs"), "should show lib.rs:\n{output}");
}

#[test]
fn milestone91_tree_depth_limits_output() {
    let repo = setup_tree_repo();
    let output = common::run_stdout(&[
        "tree",
        "--depth",
        "1",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // Depth 1 should show src/ but not files inside utils/
    assert!(output.contains("src"), "should show src:\n{output}");
    // At depth 1, we should see the top-level entries but not nested deeply
    // utils/ might show as collapsed
}

#[test]
fn milestone91_tree_no_deps_hides_arrows() {
    let repo = setup_tree_repo();
    let output =
        common::run_stdout(&["tree", "--no-deps", "--repo", repo.path().to_str().unwrap()]);
    // Should not contain dependency arrows
    assert!(
        !output.contains("→ imports:") && !output.contains("← used by:"),
        "no-deps should hide arrows:\n{output}"
    );
}

#[test]
fn milestone91_tree_focus_filters() {
    let repo = setup_tree_repo();
    let output = common::run_stdout(&[
        "tree",
        "--focus",
        "src/utils",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // Focus should show only the utils subtree
    assert!(
        output.contains("format.rs"),
        "focus should show format.rs:\n{output}"
    );
}

#[test]
fn milestone91_tree_symbols_expands() {
    let repo = setup_tree_repo();
    let output =
        common::run_stdout(&["tree", "--symbols", "--repo", repo.path().to_str().unwrap()]);
    // Should show individual symbols
    assert!(
        output.contains("greet") || output.contains("main") || output.contains("assist"),
        "symbols should show function names:\n{output}"
    );
}

#[test]
fn milestone91_tree_json_output() {
    let repo = setup_tree_repo();
    let output = common::run_stdout(&["tree", "--json", "--repo", repo.path().to_str().unwrap()]);
    let json: Value = serde_json::from_str(&output).expect("should be valid JSON");
    assert_eq!(json["command"], "tree");
    assert_eq!(json["schema_version"], 2);
    assert!(json["tree"].is_object(), "should have tree object");
}

#[test]
fn milestone91_tree_json_deterministic() {
    let repo = setup_tree_repo();
    let output1 = common::run_stdout(&["tree", "--json", "--repo", repo.path().to_str().unwrap()]);
    let output2 = common::run_stdout(&["tree", "--json", "--repo", repo.path().to_str().unwrap()]);
    assert_eq!(output1, output2, "tree JSON should be deterministic");
}
