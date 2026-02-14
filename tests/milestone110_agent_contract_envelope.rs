mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn run_find() {}
pub fn run_refs() { run_find(); }
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
fn milestone110_find_refs_and_status_use_agent_envelope() {
    let repo = setup_repo();
    let repo_path = repo.path().to_str().expect("repo path utf-8");

    let find_json: Value = serde_json::from_str(&common::run_stdout(&[
        "find", "run_find", "--repo", repo_path, "--json",
    ]))
    .expect("find output json");
    assert_eq!(find_json["ok"], true, "find should be wrapped in envelope");
    assert_eq!(find_json["command"], "find");
    assert!(
        find_json["schema"]
            .as_str()
            .unwrap_or_default()
            .contains("repo-scout/find@"),
        "find schema id missing"
    );
    assert!(find_json["data"]["results"].is_array());

    let refs_json: Value = serde_json::from_str(&common::run_stdout(&[
        "refs", "run_find", "--repo", repo_path, "--json",
    ]))
    .expect("refs output json");
    assert_eq!(refs_json["ok"], true, "refs should be wrapped in envelope");
    assert_eq!(refs_json["command"], "refs");
    assert!(refs_json["data"]["results"].is_array());

    let status_json: Value = serde_json::from_str(&common::run_stdout(&[
        "status", "--repo", repo_path, "--json",
    ]))
    .expect("status output json");
    assert_eq!(
        status_json["ok"], true,
        "status should be wrapped in envelope"
    );
    assert_eq!(status_json["command"], "status");
    assert!(status_json["data"]["summary"].is_object());
}
