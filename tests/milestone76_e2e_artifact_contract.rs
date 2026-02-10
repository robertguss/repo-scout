mod common;

#[test]
fn milestone76_e2e_plan_readme_exists_with_required_scope() {
    let readme = common::read_repo_file("agents/plans/repo-scout-e2e/README.md");
    assert!(
        readme.contains("# repo-scout E2E Release Matrix")
            && readme.contains("Tier 1")
            && readme.contains("zero unresolved")
            && readme.contains("command-matrix.md")
            && readme.contains("language-corpus-matrix.md")
            && readme.contains("gates-and-tooling-matrix.md")
            && readme.contains("runbook.md")
            && readme.contains("issues-log.md")
            && readme.contains("observations.jsonl"),
        "e2e README should define scope, risk tier, pass criteria, and split artifact index"
    );
}

#[test]
fn milestone76_e2e_command_and_language_matrix_docs_exist() {
    let command_matrix = common::read_repo_file("agents/plans/repo-scout-e2e/command-matrix.md");
    assert!(
        command_matrix.contains("index")
            && command_matrix.contains("status")
            && command_matrix.contains("find")
            && command_matrix.contains("refs")
            && command_matrix.contains("impact")
            && command_matrix.contains("context")
            && command_matrix.contains("tests-for")
            && command_matrix.contains("verify-plan")
            && command_matrix.contains("diff-impact")
            && command_matrix.contains("explain")
            && command_matrix.contains("negative")
            && command_matrix.contains("invalid"),
        "command matrix must enumerate all public commands plus negative/error scenarios"
    );

    let language_matrix =
        common::read_repo_file("agents/plans/repo-scout-e2e/language-corpus-matrix.md");
    assert!(
        language_matrix.contains("rust")
            && language_matrix.contains("go")
            && language_matrix.contains("python")
            && language_matrix.contains("typescript")
            && language_matrix.contains(".tsx")
            && language_matrix.contains("generated"),
        "language matrix must cover all supported languages and generated TSX coverage"
    );

    let gates_matrix =
        common::read_repo_file("agents/plans/repo-scout-e2e/gates-and-tooling-matrix.md");
    assert!(
        gates_matrix.contains("just check")
            && gates_matrix.contains("just docs-consistency .")
            && gates_matrix.contains("just phase18-docs-freshness .")
            && gates_matrix.contains("just phase18-maintenance-pack .")
            && gates_matrix.contains("just phase15-convergence-pack .")
            && gates_matrix.contains("just phase16-deterministic-replay .")
            && gates_matrix.contains("just phase16-benchmark-pack .")
            && gates_matrix.contains("just phase16-known-issues-budget .")
            && gates_matrix.contains("just phase16-release-checklist .")
            && gates_matrix.contains("just phase16-large-repo-benchmark .")
            && gates_matrix.contains("just phase16-large-repo-replay ."),
        "gates matrix must include all required local gate commands"
    );
}

#[test]
fn milestone76_e2e_runbook_and_observation_artifacts_exist() {
    let runbook = common::read_repo_file("agents/plans/repo-scout-e2e/runbook.md");
    assert!(
        runbook.contains("baseline")
            && runbook.contains("ongoing")
            && runbook.contains("smoke")
            && runbook.contains("full")
            && runbook.contains("sign-off"),
        "runbook should define baseline + ongoing cadence and full/smoke execution modes"
    );

    let issues_log = common::read_repo_file("agents/plans/repo-scout-e2e/issues-log.md");
    assert!(
        issues_log.contains("Open Findings")
            && issues_log.contains("Resolved Findings")
            && issues_log.contains("Waived Findings")
            && issues_log.contains("reproduction"),
        "issues log should provide rolling finding sections and reproduction details"
    );

    let observation_ledger =
        common::read_repo_file("agents/plans/repo-scout-e2e/observations.jsonl");
    assert!(
        observation_ledger.contains("\"run_id\"")
            && observation_ledger.contains("\"timestamp\"")
            && observation_ledger.contains("\"lane\"")
            && observation_ledger.contains("\"corpus\"")
            && observation_ledger.contains("\"command\"")
            && observation_ledger.contains("\"args\"")
            && observation_ledger.contains("\"result\"")
            && observation_ledger.contains("\"severity\"")
            && observation_ledger.contains("\"status\"")
            && observation_ledger.contains("\"owner\"")
            && observation_ledger.contains("\"followup_id\""),
        "observations ledger rows must carry the required machine-readable fields"
    );
}
