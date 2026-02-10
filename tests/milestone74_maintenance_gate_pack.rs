mod common;

#[test]
fn milestone74_phase18_maintenance_pack_script_exists_with_deterministic_outputs() {
    let script = common::read_repo_file("scripts/check_phase18_maintenance_pack.sh");
    assert!(
        script.contains("--repo")
            && script.contains("PASS")
            && script.contains("FAIL")
            && script.contains("maintenance-backlog-phase18.md")
            && script.contains("check_docs_consistency.sh"),
        "phase18 maintenance-pack script should support --repo and deterministic pass/fail checks"
    );
}

#[test]
fn milestone74_phase18_maintenance_pack_just_target_is_wired() {
    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase18-maintenance-pack")
            && justfile.contains("scripts/check_phase18_maintenance_pack.sh --repo"),
        "Justfile should expose phase18 maintenance-pack command"
    );
}

#[test]
fn milestone74_phase18_maintenance_pack_is_invoked_in_contract_gates_workflow() {
    let workflow = common::read_repo_file(".github/workflows/contract-gates.yml");
    assert!(
        workflow.contains("Run phase18 maintenance pack gate")
            && workflow.contains("scripts/check_phase18_maintenance_pack.sh --repo ."),
        "contract-gates workflow should invoke phase18 maintenance pack script"
    );
}
