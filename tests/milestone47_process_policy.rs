use std::fs;

fn read_repo_file(path: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{repo_root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|err| {
        panic!("failed to read {full_path}: {err}");
    })
}

fn assert_contains(text: &str, expected: &str, context: &str) {
    assert!(
        text.contains(expected),
        "{context} should contain `{expected}`"
    );
}

#[test]
fn milestone47_execplan_legacy_policy_is_documented() {
    let plans_readme = read_repo_file("agents/plans/README.md");
    assert_contains(
        &plans_readme,
        "legacy pre-Tiger adoption artifacts",
        "agents/plans/README.md",
    );
    assert_contains(&plans_readme, "phase1", "agents/plans/README.md");
    assert_contains(&plans_readme, "phase8", "agents/plans/README.md");
    assert_contains(&plans_readme, "phase9", "agents/plans/README.md");
    assert_contains(&plans_readme, "Contract Inputs", "agents/plans/README.md");
    assert_contains(
        &plans_readme,
        "AGENTS.md Constraints",
        "agents/plans/README.md",
    );
    assert_contains(
        &plans_readme,
        "Risk Tier and Required Controls",
        "agents/plans/README.md",
    );
}

#[test]
fn milestone47_artifact_strategy_is_canonical_and_materialized() {
    let policy_doc = read_repo_file("docs/contract-artifact-policy.md");
    assert_contains(
        &policy_doc,
        "Canonical strategy: PR-body-first",
        "docs/contract-artifact-policy.md",
    );
    assert_contains(
        &policy_doc,
        ".github/pull_request_template.md",
        "docs/contract-artifact-policy.md",
    );
    assert_contains(
        &policy_doc,
        "scripts/validate_tdd_cycle.sh --base origin/main",
        "docs/contract-artifact-policy.md",
    );
    assert_contains(
        &policy_doc,
        "scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md",
        "docs/contract-artifact-policy.md",
    );

    let evidence_readme = read_repo_file(".evidence/README.md");
    assert_contains(&evidence_readme, "optional", ".evidence/README.md");
    assert_contains(
        &evidence_readme,
        "EVIDENCE_PACKET.md",
        ".evidence/README.md",
    );
}
