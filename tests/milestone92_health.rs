mod common;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn run_json(args: &[&str]) -> serde_json::Value {
    let stdout = run_stdout(args);
    serde_json::from_str(&stdout).expect("stdout should be valid JSON")
}

fn setup_health_repo(repo_path: &std::path::Path) {
    // Large file (many lines)
    let large_content = (0..100)
        .map(|i| format!("fn func_{i}() {{}}\n"))
        .collect::<String>();
    common::write_file(repo_path, "src/large.rs", &large_content);

    // Small file
    common::write_file(repo_path, "src/small.rs", "fn tiny() {}\n");

    // Medium file with a big function
    let medium_content = format!(
        "fn short() {{}}\n\nfn big_function() {{\n{}\n}}\n",
        (0..50).map(|i| format!("    let x{i} = {i};\n")).collect::<String>()
    );
    common::write_file(repo_path, "src/medium.rs", &medium_content);
}

#[test]
fn milestone92_health_shows_largest_files() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&["health", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("LARGEST FILES"), "health output should contain LARGEST FILES section");
    assert!(output.contains("src/large.rs"), "health output should list the largest file");
}

#[test]
fn milestone92_health_shows_largest_functions() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&["health", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        output.contains("LARGEST FUNCTIONS"),
        "health output should contain LARGEST FUNCTIONS section"
    );
    assert!(
        output.contains("big_function"),
        "health output should list big_function"
    );
}

#[test]
fn milestone92_health_top_limits_output() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().unwrap(),
        "--top",
        "1",
    ]);
    // With --top 1, should only show the #1 entry
    assert!(output.contains("#1"), "should show rank #1");
    // Count occurrences of "#" followed by a digit — should have at most 2 (one per section)
    let rank_count = output.matches("#1").count();
    assert!(
        rank_count <= 2,
        "with --top 1, should have at most 2 rank entries (one per section), got {rank_count}"
    );
}

#[test]
fn milestone92_health_threshold_filters() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().unwrap(),
        "--threshold",
        "9999",
    ]);
    // With a very high threshold, no files or functions should appear
    assert!(
        !output.contains("#1"),
        "threshold 9999 should filter out all entries"
    );
}

#[test]
fn milestone92_health_large_files_filter() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().unwrap(),
        "--large-files",
    ]);
    assert!(output.contains("LARGEST FILES"), "should show files section");
    assert!(
        !output.contains("LARGEST FUNCTIONS"),
        "--large-files should hide functions section"
    );
}

#[test]
fn milestone92_health_large_functions_filter() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = run_stdout(&[
        "health",
        "--repo",
        repo.path().to_str().unwrap(),
        "--large-functions",
    ]);
    assert!(
        !output.contains("LARGEST FILES"),
        "--large-functions should hide files section"
    );
    assert!(
        output.contains("LARGEST FUNCTIONS"),
        "should show functions section"
    );
}

#[test]
fn milestone92_health_json_output() {
    let repo = common::temp_repo();
    setup_health_repo(repo.path());
    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let json = run_json(&[
        "health",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(json["command"], "health");
    assert!(json["schema_version"].is_number());
    assert!(json["largest_files"].is_array());
    assert!(json["largest_functions"].is_array());

    let files = json["largest_files"].as_array().unwrap();
    assert!(!files.is_empty(), "JSON should contain largest files");

    let first_file = &files[0];
    assert!(first_file["file_path"].is_string());
    assert!(first_file["line_count"].is_number());
}
