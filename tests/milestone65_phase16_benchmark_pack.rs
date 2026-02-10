mod common;

#[test]
fn milestone65_phase16_benchmark_thresholds_doc_exists() {
    let thresholds = common::read_repo_file("docs/performance-thresholds-phase16.md");
    assert!(
        thresholds.contains("Phase 16 Benchmark Pack Thresholds")
            && thresholds.contains("index")
            && thresholds.contains("find")
            && thresholds.contains("refs")
            && thresholds.contains("tests-for")
            && thresholds.contains("verify-plan")
            && thresholds.contains("diff-impact"),
        "phase16 thresholds doc should define benchmark budgets for core cross-language commands"
    );
}

#[test]
fn milestone65_phase16_benchmark_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase16_benchmark_pack.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--fixtures")
            && script.contains("--record")
            && script.contains("threshold")
            && script.contains("find")
            && script.contains("refs")
            && script.contains("tests-for")
            && script.contains("verify-plan")
            && script.contains("diff-impact"),
        "phase16 benchmark script should parse repo/fixture overrides and enforce thresholds"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase16-benchmark-pack")
            && justfile.contains("scripts/check_phase16_benchmark_pack.sh"),
        "Justfile should expose phase16 benchmark-pack command"
    );
}

#[test]
fn milestone65_performance_baseline_references_phase16_benchmark_gate() {
    let perf_doc = common::read_repo_file("docs/performance-baseline.md");
    assert!(
        perf_doc.contains("Phase 16 benchmark-pack checks")
            && perf_doc.contains("check_phase16_benchmark_pack.sh"),
        "performance baseline should document the phase16 benchmark-pack gate"
    );
}
