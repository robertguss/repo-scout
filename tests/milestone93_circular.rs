mod common;

use serde_json::Value;

/// Helper: create a repo with circular deps between two files.
/// a.rs calls b::helper(), b.rs calls a::init() — forming a cycle.
fn setup_circular_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/a.rs",
        r#"
pub fn init() {
    b::helper();
}
"#,
    );
    common::write_file(
        repo.path(),
        "src/b.rs",
        r#"
pub fn helper() {
    a::init();
}
"#,
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    repo
}

/// Helper: create a repo with no circular deps.
fn setup_clean_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
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
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    repo
}

#[test]
fn milestone93_circular_detects_cycle() {
    let repo = setup_circular_repo();
    let output = common::run_stdout(&["circular", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        output.contains("Cycle") || output.contains("cycle"),
        "should detect cycle:\n{output}"
    );
    assert!(
        output.contains("src/a.rs") || output.contains("src/b.rs"),
        "should mention files in cycle:\n{output}"
    );
}

#[test]
fn milestone93_circular_clean_repo() {
    let repo = setup_clean_repo();
    let output = common::run_stdout(&["circular", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        output.contains("No circular") || output.contains("0 cycle"),
        "should report no cycles:\n{output}"
    );
}

#[test]
fn milestone93_circular_max_length_filters() {
    let repo = setup_circular_repo();
    // With max-length=1, a 2-file cycle should be filtered out
    let output = common::run_stdout(&[
        "circular",
        "--max-length",
        "1",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("No circular") || output.contains("0 cycle"),
        "max-length=1 should filter 2-file cycle:\n{output}"
    );
}

#[test]
fn milestone93_circular_json_output() {
    let repo = setup_circular_repo();
    let output = common::run_stdout(&[
        "circular",
        "--json",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    let json: Value = serde_json::from_str(&output).expect("should be valid JSON");
    assert_eq!(json["command"], "circular");
    assert!(json["cycles"].is_array(), "should have cycles array");
    assert_eq!(json["schema_version"], 2);
}

#[test]
fn milestone93_circular_json_clean_repo() {
    let repo = setup_clean_repo();
    let output = common::run_stdout(&[
        "circular",
        "--json",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    let json: Value = serde_json::from_str(&output).expect("should be valid JSON");
    assert_eq!(json["cycles"].as_array().unwrap().len(), 0);
    assert_eq!(json["total_cycles"], 0);
}
