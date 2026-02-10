mod common;

#[test]
fn milestone66_phase16_known_issues_budget_doc_exists() {
    let budget_doc = common::read_repo_file("docs/known-issues-budget-phase16.md");
    assert!(
        budget_doc.contains("Phase 16 Known-Issues Budget")
            && budget_doc.contains("max_open")
            && budget_doc.contains("max_deferred")
            && budget_doc.contains("max_unowned")
            && budget_doc.contains("| id |")
            && budget_doc.contains("| owner |"),
        "phase16 known-issues budget doc should define thresholds and issue ownership fields"
    );
}

#[test]
fn milestone66_phase16_budget_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_known_issues_budget.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--doc")
            && script.contains("--record")
            && script.contains("max_open")
            && script.contains("max_deferred")
            && script.contains("max_unowned")
            && script.contains("deferred")
            && script.contains("owner"),
        "phase16 known-issues budget script should parse overrides and enforce issue-budget thresholds"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-known-issues-budget")
            && justfile.contains("scripts/check_phase16_known_issues_budget.sh"),
        "Justfile should expose phase16 known-issues budget command"
    );
}

#[test]
fn milestone66_roadmap_references_phase16_known_issues_budget_gate() {
    let roadmap = common::read_repo_file("agents/plans/repo-scout-roadmap-to-production-and-ga.md");
    assert!(
        roadmap.contains("check_phase16_known_issues_budget.sh"),
        "roadmap should reference the phase16 known-issues budget gate"
    );
}
