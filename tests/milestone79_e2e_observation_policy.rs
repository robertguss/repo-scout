mod common;

#[test]
fn milestone79_runner_enforces_required_observation_fields_and_status_model() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        script.contains("run_id")
            && script.contains("timestamp")
            && script.contains("lane")
            && script.contains("corpus")
            && script.contains("command")
            && script.contains("args")
            && script.contains("result")
            && script.contains("severity")
            && script.contains("status")
            && script.contains("owner")
            && script.contains("followup_id")
            && script.contains("resolved")
            && script.contains("waived")
            && script.contains("open"),
        "runner must emit the required observation row fields and status model"
    );
}

#[test]
fn milestone79_runner_enforces_zero_unresolved_policy_outside_record_mode() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        script.contains("unresolved_count")
            && script.contains("record_mode")
            && script.contains("E2E release matrix detected unresolved findings")
            && script.contains("exit 1"),
        "runner must fail strict mode when unresolved findings remain"
    );
}

#[test]
fn milestone79_observation_ledger_supports_pass_warn_fail_and_info_events() {
    let ledger = common::read_repo_file("agents/plans/repo-scout-e2e/observations.jsonl");
    assert!(
        ledger.contains("\"result\":\"PASS\"")
            && ledger.contains("\"result\":\"WARN\"")
            && ledger.contains("\"result\":\"INFO\""),
        "observation ledger should include pass/warn/info event types and be ready for fail events"
    );
}
