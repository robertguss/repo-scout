mod common;

#[test]
fn milestone68_phase16_release_checklist_doc_exists() {
    let checklist = common::read_repo_file("docs/release-checklist-phase16.md");
    assert!(
        checklist.contains("Phase 16 Release Checklist")
            && checklist.contains("quality_gate")
            && checklist.contains("evidence_gate")
            && checklist.contains("rollback_plan")
            && checklist.contains("docs_gate")
            && checklist.contains("ci_gate")
            && checklist.contains("| gate | status | evidence |"),
        "phase16 release checklist should define required gate statuses and evidence references"
    );
}

#[test]
fn milestone68_phase16_release_checklist_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_release_checklist.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--doc")
            && script.contains("--record")
            && script.contains("quality_gate")
            && script.contains("evidence_gate")
            && script.contains("rollback_plan")
            && script.contains("docs_gate")
            && script.contains("ci_gate")
            && script.contains("PASS"),
        "phase16 release checklist script should parse overrides and enforce gate statuses"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-release-checklist")
            && justfile.contains("scripts/check_phase16_release_checklist.sh"),
        "Justfile should expose phase16 release-checklist command"
    );
}

#[test]
fn milestone68_roadmap_references_phase16_release_checklist_gate() {
    let roadmap = common::read_repo_file("agents/plans/repo-scout-roadmap-to-production-and-ga.md");
    assert!(
        roadmap.contains("check_phase16_release_checklist.sh"),
        "roadmap should reference the phase16 release-checklist gate"
    );
}
