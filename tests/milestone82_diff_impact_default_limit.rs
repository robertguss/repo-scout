mod common;

#[test]
fn diff_impact_defaults_to_30_max_results() {
    let repo = common::temp_repo();
    // Create a file with many symbols to generate many impact results
    let mut source = String::new();
    for i in 0..50 {
        source.push_str(&format!("pub fn func_{i}() {{}}\n"));
    }
    source.push_str("pub fn hub() {\n");
    for i in 0..50 {
        source.push_str(&format!("    func_{i}();\n"));
    }
    source.push_str("}\n");
    common::write_file(repo.path(), "src/lib.rs", &source);
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = common::run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    // Count result lines (lines with file paths, not header lines)
    let result_count: usize = output.lines().filter(|l| l.starts_with("src/")).count();
    assert!(
        result_count <= 30,
        "default diff-impact should cap at 30 results, got {result_count}"
    );
}
