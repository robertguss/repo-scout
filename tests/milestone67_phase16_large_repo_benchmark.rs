mod common;

#[test]
fn milestone67_phase16_large_repo_thresholds_doc_exists() {
    let thresholds = common::read_repo_file("docs/performance-thresholds-phase16-large-repo.md");
    assert!(
        thresholds.contains("Phase 16 Large-Repo Benchmark Thresholds")
            && thresholds.contains("max_index_seconds")
            && thresholds.contains("max_find_seconds")
            && thresholds.contains("max_refs_seconds")
            && thresholds.contains("max_context_seconds")
            && thresholds.contains("max_verify_plan_seconds")
            && thresholds.contains("max_diff_impact_seconds"),
        "phase16 large-repo thresholds doc should define explicit command budgets"
    );
}

#[test]
fn milestone67_phase16_large_repo_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_large_repo_benchmark.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--record")
            && script.contains("threshold")
            && script.contains("context")
            && script.contains("verify-plan")
            && script.contains("diff-impact"),
        "phase16 large-repo benchmark script should parse overrides and enforce thresholds"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-large-repo-benchmark")
            && justfile.contains("scripts/check_phase16_large_repo_benchmark.sh"),
        "Justfile should expose phase16 large-repo benchmark command"
    );
}

#[test]
fn milestone67_performance_baseline_references_large_repo_benchmark_gate() {
    let perf_doc = common::read_repo_file("docs/performance-baseline.md");
    assert!(
        perf_doc.contains("Phase 16 large-repo benchmark checks")
            && perf_doc.contains("check_phase16_large_repo_benchmark.sh"),
        "performance baseline should document the phase16 large-repo benchmark gate"
    );
}

#[test]
fn milestone67_roadmap_references_large_repo_benchmark_gate() {
    let roadmap = common::read_repo_file("agents/plans/repo-scout-roadmap-to-production-and-ga.md");
    assert!(
        roadmap.contains("check_phase16_large_repo_benchmark.sh"),
        "roadmap should reference the phase16 large-repo benchmark gate"
    );
}
