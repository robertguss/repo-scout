mod common;

use common::run_stdout;
use serde_json::Value;

const FIXTURE_VITEST_PACKAGE_JSON: &str =
    include_str!("fixtures/phase14/typescript_production/vitest/package.json");
const FIXTURE_VITEST_SERVICE_TS: &str =
    include_str!("fixtures/phase14/typescript_production/vitest/src/service.ts");
const FIXTURE_VITEST_SERVICE_TEST_TS: &str =
    include_str!("fixtures/phase14/typescript_production/vitest/tests/service.test.ts");
const FIXTURE_JEST_PACKAGE_JSON: &str =
    include_str!("fixtures/phase14/typescript_production/jest/package.json");
const FIXTURE_JEST_SERVICE_TS: &str =
    include_str!("fixtures/phase14/typescript_production/jest/src/service.ts");
const FIXTURE_JEST_SERVICE_SPEC_TS: &str =
    include_str!("fixtures/phase14/typescript_production/jest/src/service.spec.ts");
const FIXTURE_AMBIGUOUS_PACKAGE_JSON: &str =
    include_str!("fixtures/phase14/typescript_production/ambiguous/package.json");
const FIXTURE_AMBIGUOUS_SERVICE_TS: &str =
    include_str!("fixtures/phase14/typescript_production/ambiguous/src/service.ts");
const FIXTURE_AMBIGUOUS_SERVICE_TEST_TS: &str =
    include_str!("fixtures/phase14/typescript_production/ambiguous/tests/service.test.ts");
const FIXTURE_INDEX_IMPORT_UTIL_INDEX_TS: &str =
    include_str!("fixtures/phase14/typescript_production/index_import/src/util/index.ts");
const FIXTURE_INDEX_IMPORT_APP_TS: &str =
    include_str!("fixtures/phase14/typescript_production/index_import/src/app.ts");

fn parse_json(output: &str) -> Value {
    serde_json::from_str(output).expect("json output should parse")
}

fn has_step(results: &[Value], step: &str, scope: &str) -> bool {
    results
        .iter()
        .any(|row| row["step"] == step && row["scope"] == scope)
}

fn has_called_by_row(results: &[Value], symbol: &str, file_path: &str) -> bool {
    results.iter().any(|row| {
        row["result_kind"] == "impacted_symbol"
            && row["symbol"] == symbol
            && row["relationship"] == "called_by"
            && row["file_path"] == file_path
    })
}

#[test]
fn milestone61_tests_for_uses_vitest_targets_when_explicitly_configured() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "package.json", FIXTURE_VITEST_PACKAGE_JSON);
    common::write_file(repo.path(), "src/service.ts", FIXTURE_VITEST_SERVICE_TS);
    common::write_file(
        repo.path(),
        "tests/service.test.ts",
        FIXTURE_VITEST_SERVICE_TEST_TS,
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "computePlan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results.iter().any(|row| {
            row["target"] == "tests/service.test.ts"
                && row["target_kind"] == "integration_test_file"
        }),
        "expected vitest-detected TypeScript test target to be runnable in tests-for"
    );
}

#[test]
fn milestone61_tests_for_requires_unambiguous_node_runner_detection() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "package.json", FIXTURE_AMBIGUOUS_PACKAGE_JSON);
    common::write_file(repo.path(), "src/service.ts", FIXTURE_AMBIGUOUS_SERVICE_TS);
    common::write_file(
        repo.path(),
        "tests/service.test.ts",
        FIXTURE_AMBIGUOUS_SERVICE_TEST_TS,
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "tests-for",
        "computePlan",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        results.is_empty(),
        "without unambiguous Jest/Vitest detection, TypeScript targets should remain non-runnable"
    );
}

#[test]
fn milestone61_verify_plan_emits_vitest_targeted_and_full_suite_steps() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "package.json", FIXTURE_VITEST_PACKAGE_JSON);
    common::write_file(repo.path(), "src/service.ts", FIXTURE_VITEST_SERVICE_TS);
    common::write_file(
        repo.path(),
        "tests/service.test.ts",
        FIXTURE_VITEST_SERVICE_TEST_TS,
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.ts",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        has_step(results, "npx vitest run tests/service.test.ts", "targeted"),
        "expected verify-plan to include runnable Vitest targeted command"
    );
    assert!(
        has_step(results, "npx vitest run", "full_suite"),
        "expected verify-plan full-suite gate to use Vitest for TypeScript-only changed scope"
    );
}

#[test]
fn milestone61_verify_plan_emits_jest_targeted_and_full_suite_steps() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "package.json", FIXTURE_JEST_PACKAGE_JSON);
    common::write_file(repo.path(), "src/service.ts", FIXTURE_JEST_SERVICE_TS);
    common::write_file(
        repo.path(),
        "src/service.spec.ts",
        FIXTURE_JEST_SERVICE_SPEC_TS,
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.ts",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        has_step(
            results,
            "npx jest --runTestsByPath src/service.spec.ts",
            "targeted"
        ),
        "expected verify-plan to include runnable Jest targeted command"
    );
    assert!(
        has_step(results, "npx jest", "full_suite"),
        "expected verify-plan full-suite gate to use Jest for TypeScript-only changed scope"
    );
}

#[test]
fn milestone61_verify_plan_ambiguous_node_runner_keeps_default_full_suite() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "package.json", FIXTURE_AMBIGUOUS_PACKAGE_JSON);
    common::write_file(repo.path(), "src/service.ts", FIXTURE_AMBIGUOUS_SERVICE_TS);
    common::write_file(
        repo.path(),
        "tests/service.test.ts",
        FIXTURE_AMBIGUOUS_SERVICE_TEST_TS,
    );

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.ts",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        !results.iter().any(|row| row["scope"] == "targeted"
            && row["step"]
                .as_str()
                .is_some_and(|step| step.starts_with("npx "))),
        "ambiguous node runner contexts should not emit runnable targeted node commands"
    );
    assert!(
        has_step(results, "cargo test", "full_suite"),
        "ambiguous node runner contexts should retain conservative cargo full-suite fallback"
    );
}

#[test]
fn milestone61_diff_impact_resolves_typescript_directory_index_import_calls() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/util/index.ts",
        FIXTURE_INDEX_IMPORT_UTIL_INDEX_TS,
    );
    common::write_file(repo.path(), "src/app.ts", FIXTURE_INDEX_IMPORT_APP_TS);

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let out = run_stdout(&[
        "diff-impact",
        "--changed-file",
        "src/util/index.ts",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload = parse_json(&out);
    let results = payload["results"]
        .as_array()
        .expect("results should be an array");

    assert!(
        has_called_by_row(results, "run", "src/app.ts"),
        "expected directory import (./util -> ./util/index.ts) to preserve called_by attribution"
    );
}
