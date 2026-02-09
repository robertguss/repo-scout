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
fn milestone48_documents_adoption_boundary_for_commit_prefix_validation() {
    let agents = read_repo_file("AGENTS.md");
    let policy_doc = read_repo_file("docs/contract-artifact-policy.md");
    let tdd_validator = read_repo_file("scripts/validate_tdd_cycle.sh");

    assert_contains(
        &agents,
        "Historical commit subjects before Tiger adoption are not retroactively enforced",
        "AGENTS.md",
    );
    assert_contains(
        &policy_doc,
        "Validation scope for commit-prefix policy is `origin/main..HEAD`",
        "docs/contract-artifact-policy.md",
    );
    assert_contains(
        &tdd_validator,
        "pre-Tiger history is excluded from prefix enforcement",
        "scripts/validate_tdd_cycle.sh",
    );
}

#[test]
fn milestone48_documents_language_contract_installation_posture() {
    let agents = read_repo_file("AGENTS.md");
    let policy_doc = read_repo_file("docs/contract-artifact-policy.md");

    assert_contains(
        &agents,
        "Contract installation scope in this repository is intentionally Rust-only",
        "AGENTS.md",
    );
    assert_contains(
        &agents,
        "contracts/languages/PYTHON_CODING_CONTRACT.md is intentionally not installed",
        "AGENTS.md",
    );
    assert_contains(
        &agents,
        "contracts/languages/TYPESCRIPT_CODING_CONTRACT.md is intentionally not installed",
        "AGENTS.md",
    );
    assert_contains(
        &policy_doc,
        "Rust-only language contract scope is the canonical local posture",
        "docs/contract-artifact-policy.md",
    );
}

#[test]
fn milestone48_audit_index_reflects_current_artifacts() {
    let readme = read_repo_file("agents/tiger-style-audit/README.md");

    assert!(
        !readme.contains("09-upstream-tiger-style-v1.1-patch-list.md"),
        "audit README should not reference removed patch list artifact"
    );
    assert_contains(
        &readme,
        "Implementation status: Milestones 1 through 5 are complete",
        "agents/tiger-style-audit/README.md",
    );
    assert_contains(
        &readme,
        "Remaining high-priority findings: none",
        "agents/tiger-style-audit/README.md",
    );
}
