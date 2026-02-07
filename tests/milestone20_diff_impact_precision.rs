mod common;

use predicates::str::contains;
use serde_json::Value;

fn run_stdout(args: &[&str]) -> String {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(args);
    let output = cmd.assert().success().get_output().stdout.clone();
    String::from_utf8(output).expect("stdout should be utf-8")
}

fn write_import_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        "mod dep {\n    pub struct Thing;\n    pub struct Other;\n}\n\nuse dep::Thing;\nuse dep::Other as Alias;\n\npub fn changed() {\n    helper();\n    let _ = std::mem::size_of::<Thing>();\n    let _ = std::mem::size_of::<Alias>();\n}\n\npub fn helper() {}\n",
    );
}

#[test]
fn milestone20_diff_impact_excludes_import_seeds_by_default() {
    let repo = common::temp_repo();
    write_import_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(
        !results.iter().any(|item| {
            item["result_kind"] == "impacted_symbol"
                && item["distance"] == 0
                && item["kind"] == "import"
        }),
        "distance=0 changed-symbol seeds should omit imports by default"
    );
}

#[test]
fn milestone20_diff_impact_include_imports_restores_import_rows() {
    let repo = common::temp_repo();
    write_import_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--include-imports",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    assert!(results.iter().any(|item| {
        item["result_kind"] == "impacted_symbol"
            && item["distance"] == 0
            && item["kind"] == "import"
    }));
}

#[test]
fn milestone20_diff_impact_changed_line_limits_seed_symbols() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn alpha() {\n    helper();\n}\n\npub fn beta() {\n    helper();\n}\n\npub fn helper() {}\n",
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let json_out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/lib.rs",
        "--changed-line",
        "src/lib.rs:1:3",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);

    let payload: Value =
        serde_json::from_str(&json_out).expect("diff-impact --json should produce valid json");
    let results = payload["results"]
        .as_array()
        .expect("results should be array");

    let mut distance_zero_symbols = results
        .iter()
        .filter(|item| item["result_kind"] == "impacted_symbol" && item["distance"] == 0)
        .map(|item| item["symbol"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    distance_zero_symbols.sort();
    distance_zero_symbols.dedup();

    assert_eq!(distance_zero_symbols, vec!["alpha".to_string()]);

    common::repo_scout_cmd()
        .args([
            "diff-impact",
            "--changed-file",
            "src/lib.rs",
            "--changed-line",
            "src/lib.rs:oops",
            "--repo",
            repo.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("path:start[:end]"))
        .stderr(contains("src/lib.rs:oops"));
}
