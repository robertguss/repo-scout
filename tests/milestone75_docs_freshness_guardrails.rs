mod common;

#[test]
fn milestone75_phase18_docs_cadence_doc_exists_with_explicit_policy() {
    let cadence_doc = common::read_repo_file("docs/maintenance-cadence-phase18.md");
    assert!(
        cadence_doc.contains("# Phase 18 Documentation Maintenance Cadence")
            && cadence_doc.contains("refresh_interval_days")
            && cadence_doc.contains("required_markers")
            && cadence_doc.contains("last_reviewed")
            && cadence_doc.contains("next_review_due")
            && cadence_doc.contains("reviewer")
            && cadence_doc.contains("status"),
        "phase18 cadence doc should define explicit freshness cadence policy and required markers"
    );
}

#[test]
fn milestone75_phase18_docs_freshness_script_exists_and_enforces_markers() {
    let script = common::read_repo_file("scripts/check_phase18_docs_freshness.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--doc")
            && script.contains("last_reviewed")
            && script.contains("next_review_due")
            && script.contains("reviewer")
            && script.contains("status")
            && script.contains("PASS")
            && script.contains("FAIL"),
        "phase18 docs freshness script should parse required markers and emit deterministic pass/fail output"
    );
}

#[test]
fn milestone75_phase18_docs_freshness_wiring_exists_in_justfile_and_ci_or_pack_path() {
    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase18-docs-freshness")
            && justfile.contains("scripts/check_phase18_docs_freshness.sh"),
        "Justfile should expose a phase18 docs freshness command"
    );

    let workflow = common::read_repo_file(".github/workflows/contract-gates.yml");
    let pack_script = common::read_repo_file("scripts/check_phase18_maintenance_pack.sh");

    let wired_via_ci = workflow.contains("check_phase18_docs_freshness.sh");
    let wired_via_pack = workflow.contains("check_phase18_maintenance_pack.sh")
        && pack_script.contains("check_phase18_docs_freshness.sh");

    assert!(
        wired_via_ci || wired_via_pack,
        "docs freshness should be wired through CI directly or through the phase18 maintenance pack path"
    );
}
