mod common;

#[test]
fn milestone71_readme_reports_phase16_closure() {
    let readme = common::read_repo_file("README.md");
    assert!(
        readme.contains("Phase 16 High-Bar/GA hardening is complete")
            && !readme.contains("Phase 16 is now in progress"),
        "README should report current closure posture instead of in-progress Phase 16 status"
    );
}

#[test]
fn milestone71_architecture_reports_post_phase16_state() {
    let architecture = common::read_repo_file("docs/architecture.md");
    assert!(
        architecture.contains("as of Phase 16 closure") && !architecture.contains("after Phase 14"),
        "architecture doc should be framed for post-Phase-16 state"
    );
}

#[test]
fn milestone71_phase9_execplan_is_marked_superseded() {
    let phase9 = common::read_repo_file("agents/plans/repo-scout-phase9-execplan.md");
    assert!(
        phase9.contains("Superseded Status")
            && phase9.contains("closed via later implemented phases")
            && !phase9.contains("- [ ] Milestone 42"),
        "phase9 execplan should be marked superseded and have no open milestone checkboxes"
    );
}

#[test]
fn milestone71_execplan_inventory_notes_phase9_superseded_policy() {
    let inventory = common::read_repo_file("agents/plans/README.md");
    assert!(
        inventory.contains("Phase 9 Superseded Status")
            && inventory.contains("historical planning artifact"),
        "plan inventory should document policy for phase9 superseded status handling"
    );
}
