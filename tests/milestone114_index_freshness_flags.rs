mod common;

use serde_json::Value;
use std::time::Duration;

#[test]
fn milestone114_require_index_fresh_and_auto_index_flags_work_for_read_commands() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn beta() {}\n");
    let repo_path = repo.path().to_str().expect("repo path utf-8");

    common::run_stdout(&["index", "--repo", repo_path]);
    std::thread::sleep(Duration::from_secs(1));
    common::write_file(repo.path(), "src/lib.rs", "pub fn beta() { let _x = 2; }\n");

    let mut stale = common::repo_scout_cmd();
    stale.args([
        "refs",
        "beta",
        "--repo",
        repo_path,
        "--json",
        "--require-index-fresh",
    ]);
    stale.assert().code(3);

    let out = common::run_stdout(&[
        "refs",
        "beta",
        "--repo",
        repo_path,
        "--json",
        "--auto-index",
        "--require-index-fresh",
    ]);
    let json: Value = serde_json::from_str(&out).expect("refs json");
    assert_eq!(json["ok"], true);
    assert_eq!(json["meta"]["index"]["stale"], false);
}
