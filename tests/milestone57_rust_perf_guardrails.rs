mod common;

#[test]
fn milestone57_rust_perf_thresholds_file_exists_and_defines_budgets() {
    let thresholds = common::read_repo_file("docs/performance-thresholds-rust.md");
    assert!(
        thresholds.contains("Rust Performance Guardrails")
            && thresholds.contains("index")
            && thresholds.contains("find")
            && thresholds.contains("refs")
            && thresholds.contains("impact")
            && thresholds.contains("diff-impact"),
        "rust thresholds doc should define budgets for core rust production commands"
    );
}

#[test]
fn milestone57_perf_guardrail_script_and_just_targets_are_wired() {
    let script = common::read_repo_file("scripts/check_rust_perf_guardrails.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--fixture")
            && (script.contains("cargo run --release") || script.contains("cargo build --release"))
            && script.contains("threshold"),
        "guardrail script should parse repo/fixture overrides and enforce thresholds"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("perf-rust-guardrails")
            && justfile.contains("perf-rust-record")
            && justfile.contains("scripts/check_rust_perf_guardrails.sh"),
        "Justfile should expose rust performance guardrail commands"
    );
}
