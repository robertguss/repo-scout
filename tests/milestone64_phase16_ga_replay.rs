mod common;

#[test]
fn milestone64_phase16_execplan_exists_and_is_contract_scoped() {
    let execplan = common::read_repo_file("agents/plans/repo-scout-phase16-execplan.md");
    assert!(
        execplan.contains("Contract Inputs")
            && execplan.contains("AGENTS.md Constraints")
            && execplan.contains("Risk Tier and Required Controls"),
        "phase16 execplan should exist and include required contract/tiger sections"
    );
}

#[test]
fn milestone64_phase16_replay_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_deterministic_replay.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--fixtures")
            && script.contains("find")
            && script.contains("refs")
            && script.contains("tests-for")
            && script.contains("verify-plan")
            && script.contains("diff-impact")
            && script.contains("cmp -s"),
        "phase16 deterministic replay script should compare repeated command outputs"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-deterministic-replay")
            && justfile.contains("scripts/check_phase16_deterministic_replay.sh"),
        "Justfile should expose phase16 deterministic replay command"
    );
}

#[test]
fn milestone64_performance_baseline_references_phase16_replay_gate() {
    let perf_doc = common::read_repo_file("docs/performance-baseline.md");
    assert!(
        perf_doc.contains("Phase 16 deterministic replay checks")
            && perf_doc.contains("check_phase16_deterministic_replay.sh"),
        "performance baseline should document the phase16 deterministic replay gate"
    );
}
