mod common;

#[test]
fn milestone70_phase16_known_issues_budget_requires_zero_deferred() {
    let budget_doc = common::read_repo_file("docs/known-issues-budget-phase16.md");
    assert!(
        budget_doc.contains("- max_deferred: 0"),
        "phase16 known-issues budget should require zero deferred issues at closure"
    );
}

#[test]
fn milestone70_phase16_deferred_issue_is_closed() {
    let budget_doc = common::read_repo_file("docs/known-issues-budget-phase16.md");
    assert!(
        budget_doc.contains("| PH16-003 |")
            && budget_doc.contains("| PH16-003 | Larger-repo benchmark budget remains pending beyond fixture-pack coverage | P3 | closed |")
            && budget_doc.contains("Milestone 67")
            && budget_doc.contains("Milestone 69"),
        "phase16 deferred issue PH16-003 should be closed with closure evidence"
    );
}

#[test]
fn milestone70_release_checklist_captures_zero_deferred_known_issues_evidence() {
    let checklist = common::read_repo_file("docs/release-checklist-phase16.md");
    assert!(
        checklist.contains("known-issues budget") && checklist.contains("deferred=0"),
        "phase16 release checklist should capture known-issues deferred=0 closure evidence"
    );
}
