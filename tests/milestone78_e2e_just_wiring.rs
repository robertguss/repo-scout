mod common;

#[test]
fn milestone78_justfile_exposes_e2e_release_matrix_targets() {
    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("e2e-release-matrix")
            && justfile.contains("scripts/run_e2e_release_matrix.sh")
            && justfile.contains("--mode full")
            && justfile.contains("e2e-release-matrix-record")
            && justfile.contains("--record"),
        "Justfile should expose strict and record e2e release-matrix targets"
    );
}
