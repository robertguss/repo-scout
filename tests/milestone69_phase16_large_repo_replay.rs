mod common;

#[test]
fn milestone69_phase16_large_repo_replay_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_large_repo_replay.sh");
    assert!(
        script.contains("--repo")
            && script.contains("find")
            && script.contains("refs")
            && script.contains("tests-for")
            && script.contains("verify-plan")
            && script.contains("diff-impact")
            && script.contains("context")
            && script.contains("cmp -s"),
        "phase16 large-repo replay script should compare repeated command outputs"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-large-repo-replay")
            && justfile.contains("scripts/check_phase16_large_repo_replay.sh"),
        "Justfile should expose phase16 large-repo replay command"
    );
}

#[test]
fn milestone69_performance_baseline_references_phase16_large_repo_replay_gate() {
    let perf_doc = common::read_repo_file("docs/performance-baseline.md");
    assert!(
        perf_doc.contains("Phase 16 large-repo deterministic replay checks")
            && perf_doc.contains("check_phase16_large_repo_replay.sh"),
        "performance baseline should document the phase16 large-repo replay gate"
    );
}

#[test]
fn milestone69_roadmap_references_phase16_large_repo_replay_gate() {
    let roadmap = common::read_repo_file("agents/plans/repo-scout-roadmap-to-production-and-ga.md");
    assert!(
        roadmap.contains("check_phase16_large_repo_replay.sh")
            && roadmap.contains("phase16-large-repo-replay"),
        "roadmap should reference the phase16 large-repo replay gate"
    );
}
