mod common;

#[test]
fn milestone73_phase18_maintenance_backlog_doc_exists_with_required_fields() {
    let backlog = common::read_repo_file("docs/maintenance-backlog-phase18.md");
    assert!(
        backlog.contains("# Phase 18 Maintenance Backlog")
            && backlog.contains("| id | priority | owner | status | target window | notes |")
            && backlog.contains("| MB18-"),
        "phase18 maintenance backlog doc should define a machine-checkable table with required fields"
    );
}

#[test]
fn milestone73_phase18_maintenance_backlog_rows_are_complete() {
    let backlog = common::read_repo_file("docs/maintenance-backlog-phase18.md");
    let mut item_count = 0usize;

    for line in backlog.lines() {
        if !line.starts_with("| MB18-") {
            continue;
        }

        item_count += 1;
        let columns: Vec<&str> = line.split('|').map(str::trim).collect();
        assert_eq!(
            columns.len(),
            8,
            "backlog item rows must include id, priority, owner, status, target window, and notes"
        );

        for value in &columns[1..7] {
            assert!(
                !value.is_empty(),
                "backlog item row has an empty required field: {line}"
            );
        }
    }

    assert!(
        item_count > 0,
        "expected at least one maintenance backlog item row"
    );
}
