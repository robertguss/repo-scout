mod common;

use serde_json::Value;
use std::time::Duration;

#[test]
fn milestone113_status_reports_freshness_and_queries_distinguish_stale_from_no_results() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn alpha() {}\n");
    let repo_path = repo.path().to_str().expect("repo path utf-8");

    common::run_stdout(&["index", "--repo", repo_path]);

    let status_fresh: Value = serde_json::from_str(&common::run_stdout(&[
        "status", "--repo", repo_path, "--json",
    ]))
    .expect("status json");

    assert!(status_fresh["meta"]["index"]["indexed_at"].is_string());
    assert_eq!(status_fresh["meta"]["index"]["stale"], false);

    std::thread::sleep(Duration::from_secs(1));
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn alpha() { let _x = 1; }\n",
    );

    let status_stale: Value = serde_json::from_str(&common::run_stdout(&[
        "status", "--repo", repo_path, "--json",
    ]))
    .expect("status json");
    assert_eq!(status_stale["meta"]["index"]["stale"], true);

    let mut stale_query = common::repo_scout_cmd();
    stale_query.args([
        "find",
        "does_not_exist",
        "--repo",
        repo_path,
        "--json",
        "--require-index-fresh",
    ]);
    stale_query.assert().code(3);

    let no_result: Value = serde_json::from_str(&common::run_stdout(&[
        "find",
        "does_not_exist",
        "--repo",
        repo_path,
        "--json",
    ]))
    .expect("find json");
    assert_eq!(no_result["ok"], true);
    assert!(
        no_result["data"]["results"]
            .as_array()
            .expect("results array")
            .is_empty(),
        "no-result query should succeed with empty results, not fail"
    );
}
