mod common;

use common::run_stdout;
use serde_json::Value;
use std::path::Path;

const FIXTURE_RUST_LIB_RS: &str = include_str!("fixtures/phase15/convergence_pack/rust/src/lib.rs");
const FIXTURE_RUST_TEST_RS: &str =
    include_str!("fixtures/phase15/convergence_pack/rust/tests/phase63_flow.rs");
const FIXTURE_GO_SERVICE_GO: &str =
    include_str!("fixtures/phase15/convergence_pack/go/src/service.go");
const FIXTURE_GO_SERVICE_TEST_GO: &str =
    include_str!("fixtures/phase15/convergence_pack/go/src/service_test.go");
const FIXTURE_PYTEST_INI: &str =
    include_str!("fixtures/phase15/convergence_pack/python/pytest.ini");
const FIXTURE_PYTHON_SERVICE_PY: &str =
    include_str!("fixtures/phase15/convergence_pack/python/src/service.py");
const FIXTURE_PYTHON_TEST_PY: &str =
    include_str!("fixtures/phase15/convergence_pack/python/tests/test_service.py");
const FIXTURE_VITEST_PACKAGE_JSON: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_vitest/package.json");
const FIXTURE_VITEST_SERVICE_TS: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_vitest/src/service.ts");
const FIXTURE_VITEST_TEST_TS: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_vitest/tests/service.test.ts");
const FIXTURE_JEST_PACKAGE_JSON: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_jest/package.json");
const FIXTURE_JEST_SERVICE_TS: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_jest/src/service.ts");
const FIXTURE_JEST_SPEC_TS: &str =
    include_str!("fixtures/phase15/convergence_pack/typescript_jest/src/service.spec.ts");

fn parse_json(output: &str) -> Value {
    serde_json::from_str(output).expect("json output should parse")
}

fn results(payload: &Value) -> &[Value] {
    payload["results"]
        .as_array()
        .expect("results should be an array")
}

fn has_target(results: &[Value], target: &str, target_kind: &str) -> bool {
    results
        .iter()
        .any(|row| row["target"] == target && row["target_kind"] == target_kind)
}

fn has_step(results: &[Value], step: &str, scope: &str) -> bool {
    results
        .iter()
        .any(|row| row["step"] == step && row["scope"] == scope)
}

#[test]
fn milestone63_phase15_convergence_pack_fixture_layout_exists() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let required_files = [
        "tests/fixtures/phase15/convergence_pack/rust/src/lib.rs",
        "tests/fixtures/phase15/convergence_pack/rust/tests/phase63_flow.rs",
        "tests/fixtures/phase15/convergence_pack/go/src/service.go",
        "tests/fixtures/phase15/convergence_pack/go/src/service_test.go",
        "tests/fixtures/phase15/convergence_pack/python/pytest.ini",
        "tests/fixtures/phase15/convergence_pack/python/src/service.py",
        "tests/fixtures/phase15/convergence_pack/python/tests/test_service.py",
        "tests/fixtures/phase15/convergence_pack/typescript_vitest/package.json",
        "tests/fixtures/phase15/convergence_pack/typescript_vitest/src/service.ts",
        "tests/fixtures/phase15/convergence_pack/typescript_vitest/tests/service.test.ts",
        "tests/fixtures/phase15/convergence_pack/typescript_jest/package.json",
        "tests/fixtures/phase15/convergence_pack/typescript_jest/src/service.ts",
        "tests/fixtures/phase15/convergence_pack/typescript_jest/src/service.spec.ts",
    ];

    for file in required_files {
        assert!(
            repo_root.join(file).is_file(),
            "missing required convergence-pack fixture file: {file}"
        );
    }
}

#[test]
fn milestone63_phase15_convergence_pack_script_and_just_target_are_wired() {
    let script = common::read_repo_file("scripts/check_phase15_convergence_pack.sh");
    assert!(
        script.contains("--repo")
            && script.contains("--fixtures")
            && script.contains("tests-for")
            && script.contains("verify-plan")
            && script.contains("go test ./...")
            && script.contains("pytest")
            && script.contains("npx vitest run")
            && script.contains("npx jest"),
        "phase15 convergence-pack script should validate cross-language command contracts"
    );

    let justfile = common::read_repo_file("Justfile");
    assert!(
        justfile.contains("phase15-convergence-pack")
            && justfile.contains("scripts/check_phase15_convergence_pack.sh"),
        "Justfile should expose phase15 convergence-pack command"
    );
}

#[test]
fn milestone63_cross_language_command_contracts_match_fixture_pack() {
    let rust_repo = common::temp_repo();
    common::write_file(rust_repo.path(), "src/lib.rs", FIXTURE_RUST_LIB_RS);
    common::write_file(
        rust_repo.path(),
        "tests/phase63_flow.rs",
        FIXTURE_RUST_TEST_RS,
    );
    run_stdout(&["index", "--repo", rust_repo.path().to_str().unwrap()]);
    let rust_tests_for = parse_json(&run_stdout(&[
        "tests-for",
        "phase63_plan",
        "--repo",
        rust_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(rust_tests_for["schema_version"], 2);
    assert!(has_target(
        results(&rust_tests_for),
        "tests/phase63_flow.rs",
        "integration_test_file"
    ));
    let rust_verify = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/lib.rs",
        "--repo",
        rust_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(rust_verify["schema_version"], 2);
    assert!(has_step(
        results(&rust_verify),
        "cargo test --test phase63_flow",
        "targeted"
    ));
    assert!(has_step(results(&rust_verify), "cargo test", "full_suite"));

    let go_repo = common::temp_repo();
    common::write_file(go_repo.path(), "src/service.go", FIXTURE_GO_SERVICE_GO);
    common::write_file(
        go_repo.path(),
        "src/service_test.go",
        FIXTURE_GO_SERVICE_TEST_GO,
    );
    run_stdout(&["index", "--repo", go_repo.path().to_str().unwrap()]);
    let go_tests_for = parse_json(&run_stdout(&[
        "tests-for",
        "PlanPhase63",
        "--repo",
        go_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(go_tests_for["schema_version"], 2);
    assert!(has_target(
        results(&go_tests_for),
        "src/service_test.go",
        "integration_test_file"
    ));
    let go_verify = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.go",
        "--repo",
        go_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(go_verify["schema_version"], 2);
    assert!(has_step(results(&go_verify), "go test ./src", "targeted"));
    assert!(has_step(results(&go_verify), "go test ./...", "full_suite"));

    let python_repo = common::temp_repo();
    common::write_file(python_repo.path(), "pytest.ini", FIXTURE_PYTEST_INI);
    common::write_file(
        python_repo.path(),
        "src/service.py",
        FIXTURE_PYTHON_SERVICE_PY,
    );
    common::write_file(
        python_repo.path(),
        "tests/test_service.py",
        FIXTURE_PYTHON_TEST_PY,
    );
    run_stdout(&["index", "--repo", python_repo.path().to_str().unwrap()]);
    let python_tests_for = parse_json(&run_stdout(&[
        "tests-for",
        "plan_phase63",
        "--repo",
        python_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(python_tests_for["schema_version"], 2);
    assert!(has_target(
        results(&python_tests_for),
        "tests/test_service.py",
        "integration_test_file"
    ));
    let python_verify = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.py",
        "--repo",
        python_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(python_verify["schema_version"], 2);
    assert!(has_step(
        results(&python_verify),
        "pytest tests/test_service.py",
        "targeted"
    ));
    assert!(has_step(results(&python_verify), "pytest", "full_suite"));

    let vitest_repo = common::temp_repo();
    common::write_file(
        vitest_repo.path(),
        "package.json",
        FIXTURE_VITEST_PACKAGE_JSON,
    );
    common::write_file(
        vitest_repo.path(),
        "src/service.ts",
        FIXTURE_VITEST_SERVICE_TS,
    );
    common::write_file(
        vitest_repo.path(),
        "tests/service.test.ts",
        FIXTURE_VITEST_TEST_TS,
    );
    run_stdout(&["index", "--repo", vitest_repo.path().to_str().unwrap()]);
    let vitest_tests_for = parse_json(&run_stdout(&[
        "tests-for",
        "planPhase63",
        "--repo",
        vitest_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(vitest_tests_for["schema_version"], 2);
    assert!(has_target(
        results(&vitest_tests_for),
        "tests/service.test.ts",
        "integration_test_file"
    ));
    let vitest_verify = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.ts",
        "--repo",
        vitest_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(vitest_verify["schema_version"], 2);
    assert!(has_step(
        results(&vitest_verify),
        "npx vitest run tests/service.test.ts",
        "targeted"
    ));
    assert!(has_step(
        results(&vitest_verify),
        "npx vitest run",
        "full_suite"
    ));

    let jest_repo = common::temp_repo();
    common::write_file(jest_repo.path(), "package.json", FIXTURE_JEST_PACKAGE_JSON);
    common::write_file(jest_repo.path(), "src/service.ts", FIXTURE_JEST_SERVICE_TS);
    common::write_file(
        jest_repo.path(),
        "src/service.spec.ts",
        FIXTURE_JEST_SPEC_TS,
    );
    run_stdout(&["index", "--repo", jest_repo.path().to_str().unwrap()]);
    let jest_tests_for = parse_json(&run_stdout(&[
        "tests-for",
        "planPhase63",
        "--repo",
        jest_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(jest_tests_for["schema_version"], 2);
    assert!(has_target(
        results(&jest_tests_for),
        "src/service.spec.ts",
        "integration_test_file"
    ));
    let jest_verify = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.ts",
        "--repo",
        jest_repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert_eq!(jest_verify["schema_version"], 2);
    assert!(has_step(
        results(&jest_verify),
        "npx jest --runTestsByPath src/service.spec.ts",
        "targeted"
    ));
    assert!(has_step(results(&jest_verify), "npx jest", "full_suite"));
}
