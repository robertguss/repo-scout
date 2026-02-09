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
    let context = "agents/plans/README.md";
    for expected in [
        "legacy pre-Tiger adoption artifacts",
        "phase1",
        "phase8",
        "phase9",
        "Contract Inputs",
        "AGENTS.md Constraints",
        "Risk Tier and Required Controls",
    ] {
        assert_contains(&plans_readme, expected, context);
    }
}

#[test]
fn milestone47_artifact_strategy_is_canonical_and_materialized() {
    let policy_doc = read_repo_file("docs/contract-artifact-policy.md");
    let context = "docs/contract-artifact-policy.md";
    for expected in [
        "Canonical strategy: PR-body-first",
        ".github/pull_request_template.md",
        "scripts/validate_tdd_cycle.sh --base origin/main",
        "scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md",
    ] {
        assert_contains(&policy_doc, expected, context);
    }

    let evidence_readme = read_repo_file(".evidence/README.md");
    for expected in ["optional", "EVIDENCE_PACKET.md"] {
        assert_contains(&evidence_readme, expected, ".evidence/README.md");
    }
}
