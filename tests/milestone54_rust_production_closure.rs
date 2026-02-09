mod common;

use common::run_stdout;
use serde_json::Value;
use std::path::Path;

fn diff_impact_json(repo_root: &Path, changed_file: &str) -> Value {
    let output = run_stdout(&[
        "diff-impact",
        "--changed-file",
        changed_file,
        "--repo",
        repo_root.to_str().expect("repo path should be utf-8"),
        "--json",
    ]);
    serde_json::from_str(&output).expect("diff-impact json should parse")
}

fn has_called_by_row(payload: &Value, symbol: &str, file_path: &str) -> bool {
    payload["results"]
        .as_array()
        .expect("results should be an array")
        .iter()
        .any(|row| {
            row["result_kind"] == "impacted_symbol"
                && row["relationship"] == "called_by"
                && row["symbol"] == symbol
                && row["file_path"] == file_path
        })
}

fn write_mod_rs_disambiguation_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase11/rust_production/mod_rs_disambiguation/src/lib.rs"),
    );
    common::write_file(
        repo,
        "src/support.rs",
        include_str!("fixtures/phase11/rust_production/mod_rs_disambiguation/src/support.rs"),
    );
    common::write_file(
        repo,
        "src/util/mod.rs",
        include_str!("fixtures/phase11/rust_production/mod_rs_disambiguation/src/util/mod.rs"),
    );
}

fn write_crate_qualified_disambiguation_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase11/rust_production/crate_qualified_disambiguation/src/lib.rs"),
    );
    common::write_file(
        repo,
        "src/support.rs",
        include_str!(
            "fixtures/phase11/rust_production/crate_qualified_disambiguation/src/support.rs"
        ),
    );
    common::write_file(
        repo,
        "src/util/mod.rs",
        include_str!(
            "fixtures/phase11/rust_production/crate_qualified_disambiguation/src/util/mod.rs"
        ),
    );
}

fn write_super_qualified_preference_fixture(repo: &Path) {
    common::write_file(
        repo,
        "src/lib.rs",
        include_str!("fixtures/phase11/rust_production/super_qualified_preference/src/lib.rs"),
    );
    common::write_file(
        repo,
        "src/parent.rs",
        include_str!("fixtures/phase11/rust_production/super_qualified_preference/src/parent.rs"),
    );
    common::write_file(
        repo,
        "src/parent/child.rs",
        include_str!(
            "fixtures/phase11/rust_production/super_qualified_preference/src/parent/child.rs"
        ),
    );
}

#[test]
fn milestone54_diff_impact_mod_rs_disambiguates_duplicate_symbols() {
    let repo = common::temp_repo();
    write_mod_rs_disambiguation_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_json(repo.path(), "src/util/mod.rs");

    assert!(
        has_called_by_row(&payload, "run", "src/lib.rs"),
        "expected diff-impact to include run as called_by for src/util/mod.rs helper"
    );
}

#[test]
fn milestone54_diff_impact_crate_qualified_call_disambiguates() {
    let repo = common::temp_repo();
    write_crate_qualified_disambiguation_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_json(repo.path(), "src/util/mod.rs");

    assert!(
        has_called_by_row(&payload, "run", "src/lib.rs"),
        "expected diff-impact to include run as called_by for crate-qualified util::helper"
    );
}

#[test]
fn milestone54_diff_impact_super_qualified_call_prefers_parent_symbol() {
    let repo = common::temp_repo();
    write_super_qualified_preference_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let payload = diff_impact_json(repo.path(), "src/parent.rs");

    assert!(
        has_called_by_row(&payload, "run", "src/parent/child.rs"),
        "expected diff-impact to include child::run as called_by for parent top_helper"
    );
}
