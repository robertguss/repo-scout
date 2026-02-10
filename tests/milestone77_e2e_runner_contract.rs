mod common;

#[test]
fn milestone77_e2e_runner_script_exists_with_required_cli() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        script.contains("#!/usr/bin/env bash")
            && script.contains("--repo")
            && script.contains("--mode")
            && script.contains("--record")
            && script.contains("full")
            && script.contains("smoke"),
        "runner script must expose repo/mode/record interfaces with full and smoke modes"
    );
}

#[test]
fn milestone77_e2e_runner_covers_required_corpora_and_commands() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        script.contains("tests/fixtures/phase15/convergence_pack/rust")
            && script.contains("tests/fixtures/phase15/convergence_pack/go")
            && script.contains("tests/fixtures/phase15/convergence_pack/python")
            && script.contains("tests/fixtures/phase15/convergence_pack/typescript_vitest")
            && script.contains("tests/fixtures/phase15/convergence_pack/typescript_jest")
            && script.contains(".tsx")
            && script.contains("index")
            && script.contains("status")
            && script.contains("find")
            && script.contains("refs")
            && script.contains("impact")
            && script.contains("context")
            && script.contains("tests-for")
            && script.contains("verify-plan")
            && script.contains("diff-impact")
            && script.contains("explain"),
        "runner script must cover all required corpora, TSX coverage, and all CLI commands"
    );
}

#[test]
fn milestone77_e2e_runner_enforces_determinism_negative_checks_and_gate_suite() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        script.contains("cmp -s")
            && script.contains("--include-tests")
            && script.contains("--exclude-tests")
            && script.contains("changed-line")
            && script.contains("just check")
            && script.contains("just docs-consistency .")
            && script.contains("just phase18-docs-freshness .")
            && script.contains("just phase18-maintenance-pack .")
            && script.contains("just phase15-convergence-pack .")
            && script.contains("just phase16-deterministic-replay .")
            && script.contains("just phase16-benchmark-pack .")
            && script.contains("just phase16-known-issues-budget .")
            && script.contains("just phase16-release-checklist .")
            && script.contains("just phase16-large-repo-benchmark .")
            && script.contains("just phase16-large-repo-replay ."),
        "runner script must include deterministic replay checks, negative checks, and required local gate suite"
    );
}
