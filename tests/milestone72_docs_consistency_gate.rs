mod common;

#[test]
fn milestone72_docs_consistency_script_exists_and_enforces_alignment_checks() {
    let script = common::read_repo_file("scripts/check_docs_consistency.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--record")
            && script.contains("README.md")
            && script.contains("docs/architecture.md")
            && script.contains("CHANGELOG.md")
            && script.contains("repo-scout-phase9-execplan.md")
            && script.contains("Phase 16 High-Bar/GA hardening is complete")
            && script.contains("as of Phase 16 closure")
            && script.contains("## [Unreleased]")
            && script.contains("Superseded Status")
            && script.contains("closed via later implemented phases")
            && script.contains("PASS"),
        "docs consistency script should enforce required status-alignment checks"
    );
}

#[test]
fn milestone72_justfile_exposes_docs_consistency_command() {
    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("docs-consistency")
            && justfile.contains("scripts/check_docs_consistency.sh --repo"),
        "Justfile should expose docs-consistency command"
    );
}

#[test]
fn milestone72_ci_invokes_docs_consistency_gate() {
    let workflow = common::read_repo_file(".github/workflows/contract-gates.yml");
    assert!(
        workflow.contains("check_docs_consistency.sh")
            && workflow.contains("Run docs consistency gate")
            && workflow.contains("scripts/check_docs_consistency.sh --repo ."),
        "contract-gates workflow should invoke docs consistency gate"
    );
}
