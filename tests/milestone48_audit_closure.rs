mod common;

fn assert_contains(text: &str, expected: &str, context: &str) {
    assert!(
        text.contains(expected),
        "{context} should contain `{expected}`"
    );
}

#[test]
fn milestone48_documents_adoption_boundary_for_commit_prefix_validation() {
    let agents = common::read_repo_file("AGENTS.md");
    let policy_doc = common::read_repo_file("docs/contract-artifact-policy.md");
    let tdd_validator = common::read_repo_file("scripts/validate_tdd_cycle.sh");

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
    let agents = common::read_repo_file("AGENTS.md");
    let policy_doc = common::read_repo_file("docs/contract-artifact-policy.md");

    for expected in [
        "Contract installation scope in this repository is intentionally Rust-only",
        "contracts/languages/PYTHON_CODING_CONTRACT.md is intentionally not installed",
        "contracts/languages/TYPESCRIPT_CODING_CONTRACT.md is intentionally not installed",
    ] {
        assert_contains(&agents, expected, "AGENTS.md");
    }
    assert_contains(
        &policy_doc,
        "Rust-only language contract scope is the canonical local posture",
        "docs/contract-artifact-policy.md",
    );
}

#[test]
fn milestone48_audit_index_reflects_current_artifacts() {
    let readme = common::read_repo_file("agents/tiger-style-audit/README.md");

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
    assert!(
        !readme.contains("/Users/robertguss/Projects/programming_tiger_style"),
        "audit README should not contain developer-local filesystem paths"
    );
}

#[test]
fn milestone48_implementation_prompt_uses_repository_relative_paths() {
    let prompt =
        common::read_repo_file("agents/tiger-style-audit/08-implementation-session-prompt.md");
    assert!(
        !prompt.contains("/Users/robertguss/Projects/experiments/repo-scout/"),
        "implementation prompt should not hardcode developer-local repository roots"
    );
    for expected in [
        "- `agents/tiger-style-audit/README.md`",
        "- `agents/tiger-style-audit/02-contract-installation-drift.md`",
        "- `AGENTS.md`",
        "- `contracts/core/*.md`",
        "- `contracts/languages/RUST_CODING_CONTRACT.md`",
    ] {
        assert_contains(
            &prompt,
            expected,
            "agents/tiger-style-audit/08-implementation-session-prompt.md",
        );
    }
}
