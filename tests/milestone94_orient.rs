mod common;

use serde_json::Value;

/// Helper: create a multi-file repo for orient testing.
fn setup_orient_repo() -> tempfile::TempDir {
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

pub fn util() {
    println!("util");
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
fn milestone94_orient_shows_all_sections() {
    let repo = setup_orient_repo();
    let output = common::run_stdout(&["orient", "--repo", repo.path().to_str().unwrap()]);
    // Should contain all major sections
    assert!(
        output.contains("STRUCTURE"),
        "should have STRUCTURE section:\n{output}"
    );
    assert!(
        output.contains("HEALTH"),
        "should have HEALTH section:\n{output}"
    );
    assert!(
        output.contains("HOTSPOTS"),
        "should have HOTSPOTS section:\n{output}"
    );
    assert!(
        output.contains("CIRCULAR"),
        "should have CIRCULAR section:\n{output}"
    );
    assert!(
        output.contains("RECOMMENDATIONS"),
        "should have RECOMMENDATIONS section:\n{output}"
    );
}

#[test]
fn milestone94_orient_generates_recommendations() {
    let repo = setup_orient_repo();
    let output = common::run_stdout(&["orient", "--repo", repo.path().to_str().unwrap()]);
    // Should generate at least one recommendation
    assert!(
        output.contains("Start exploring") || output.contains("entry point"),
        "should suggest entry point:\n{output}"
    );
}

#[test]
fn milestone94_orient_depth_respected() {
    let repo = setup_orient_repo();
    let output = common::run_stdout(&[
        "orient",
        "--depth",
        "1",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);
    // Should still have all sections
    assert!(
        output.contains("STRUCTURE"),
        "should have STRUCTURE:\n{output}"
    );
    assert!(output.contains("HEALTH"), "should have HEALTH:\n{output}");
}

#[test]
fn milestone94_orient_json_contains_all_sections() {
    let repo = setup_orient_repo();
    let output = common::run_stdout(&["orient", "--json", "--repo", repo.path().to_str().unwrap()]);
    let json: Value = serde_json::from_str(&output).expect("should be valid JSON");
    assert_eq!(json["command"], "orient");
    assert_eq!(json["schema_version"], 2);
    assert!(json["tree"].is_object(), "should have tree:\n{output}");
    assert!(json["health"].is_object(), "should have health:\n{output}");
    assert!(
        json["hotspots"].is_array(),
        "should have hotspots:\n{output}"
    );
    assert!(
        json["circular"].is_object(),
        "should have circular:\n{output}"
    );
    assert!(
        json["recommendations"].is_array(),
        "should have recommendations:\n{output}"
    );
}

#[test]
fn milestone94_orient_json_deterministic() {
    let repo = setup_orient_repo();
    let output1 =
        common::run_stdout(&["orient", "--json", "--repo", repo.path().to_str().unwrap()]);
    let output2 =
        common::run_stdout(&["orient", "--json", "--repo", repo.path().to_str().unwrap()]);
    assert_eq!(output1, output2, "orient JSON should be deterministic");
}
