mod common;

use common::run_stdout;
use serde_json::Value;

fn parse_json(output: &str) -> Value {
    serde_json::from_str(output).expect("json output should parse")
}

fn write_go_scope_fixture(repo: &std::path::Path) {
    common::write_file(
        repo,
        "src/service.go",
        include_str!("fixtures/phase15/go_recommendations/src/service.go"),
    );
    common::write_file(
        repo,
        "src/service_test.go",
        include_str!("fixtures/phase15/go_recommendations/src/service_test.go"),
    );
}

fn results(payload: &Value) -> &[Value] {
    payload["results"]
        .as_array()
        .expect("results should be an array")
}

fn has_file(results: &[Value], file_path: &str) -> bool {
    results.iter().any(|row| row["file_path"] == file_path)
}

fn has_step(results: &[Value], step: &str, scope: &str) -> bool {
    results
        .iter()
        .any(|row| row["step"] == step && row["scope"] == scope)
}

#[test]
fn milestone62_scope_filters_normalize_test_patterns_across_languages() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "// phase62crossscope\npub fn phase62_cross_scope() -> usize { 62 }\n",
    );
    common::write_file(
        repo.path(),
        "tests/lib_test.rs",
        "// phase62crossscope\n#[test]\nfn test_phase62_cross_scope() { assert_eq!(62, 62); }\n",
    );
    common::write_file(
        repo.path(),
        "src/service.go",
        "package service\n\n// phase62crossscope\nfunc phase62_cross_scope_go() int { return 62 }\n",
    );
    common::write_file(
        repo.path(),
        "src/service_test.go",
        "package service\n\nimport \"testing\"\n\n// phase62crossscope\nfunc TestPhase62CrossScope(t *testing.T) {}\n",
    );
    common::write_file(
        repo.path(),
        "src/service.py",
        "# phase62crossscope\ndef phase62_cross_scope_py():\n    return 62\n",
    );
    common::write_file(
        repo.path(),
        "tests/test_service.py",
        "# phase62crossscope\ndef test_phase62_cross_scope_py():\n    assert 62 == 62\n",
    );
    common::write_file(
        repo.path(),
        "src/service.ts",
        "// phase62crossscope\nexport function phase62CrossScopeTs(): number { return 62; }\n",
    );
    common::write_file(
        repo.path(),
        "tests/service.test.ts",
        "// phase62crossscope\nimport { phase62CrossScopeTs } from \"../src/service\";\n\ntest(\"phase62\", () => expect(phase62CrossScopeTs()).toBe(62));\n",
    );
    common::write_file(repo.path(), "docs/guide.md", "phase62crossscope\n");

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload = parse_json(&run_stdout(&[
        "refs",
        "phase62crossscope",
        "--repo",
        repo.path().to_str().unwrap(),
        "--code-only",
        "--exclude-tests",
        "--json",
    ]));
    let results = results(&payload);

    assert!(has_file(results, "src/lib.rs"));
    assert!(has_file(results, "src/service.go"));
    assert!(has_file(results, "src/service.py"));
    assert!(has_file(results, "src/service.ts"));
    assert!(!has_file(results, "src/service_test.go"));
    assert!(!has_file(results, "tests/lib_test.rs"));
    assert!(!has_file(results, "tests/test_service.py"));
    assert!(!has_file(results, "tests/service.test.ts"));
    assert!(!has_file(results, "docs/guide.md"));
}

#[test]
fn milestone62_exclude_tests_filters_go_test_suffix_paths() {
    let repo = common::temp_repo();
    write_go_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let baseline_find = parse_json(&run_stdout(&[
        "find",
        "phase62goexclude",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert!(
        has_file(results(&baseline_find), "src/service_test.go"),
        "baseline find should include go _test.go path before exclude-tests filtering"
    );

    let scoped_find = parse_json(&run_stdout(&[
        "find",
        "phase62goexclude",
        "--repo",
        repo.path().to_str().unwrap(),
        "--exclude-tests",
        "--json",
    ]));
    assert!(
        has_file(results(&scoped_find), "src/service.go"),
        "scoped find should keep non-test go source"
    );
    assert!(
        !has_file(results(&scoped_find), "src/service_test.go"),
        "scoped find should omit go _test.go files"
    );

    let baseline_refs = parse_json(&run_stdout(&[
        "refs",
        "phase62goexclude",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]));
    assert!(
        has_file(results(&baseline_refs), "src/service_test.go"),
        "baseline refs should include go _test.go path before exclude-tests filtering"
    );

    let scoped_refs = parse_json(&run_stdout(&[
        "refs",
        "phase62goexclude",
        "--repo",
        repo.path().to_str().unwrap(),
        "--exclude-tests",
        "--json",
    ]));
    assert!(
        has_file(results(&scoped_refs), "src/service.go"),
        "scoped refs should keep non-test go source"
    );
    assert!(
        !has_file(results(&scoped_refs), "src/service_test.go"),
        "scoped refs should omit go _test.go files"
    );
}

#[test]
fn milestone62_tests_for_include_support_recognizes_go_test_suffix() {
    let repo = common::temp_repo();
    write_go_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload = parse_json(&run_stdout(&[
        "tests-for",
        "PlanPhase62",
        "--repo",
        repo.path().to_str().unwrap(),
        "--include-support",
        "--json",
    ]));
    let results = results(&payload);

    assert!(
        results
            .iter()
            .any(|row| row["target"] == "src/service_test.go"),
        "expected tests-for include-support to include go _test.go targets"
    );
}

#[test]
fn milestone62_tests_for_uses_go_targets_by_default_when_runnable() {
    let repo = common::temp_repo();
    write_go_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload = parse_json(&run_stdout(&[
        "tests-for",
        "PlanPhase62",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]));
    let results = results(&payload);

    assert!(
        results
            .iter()
            .any(|row| row["target"] == "src/service_test.go"),
        "expected go _test.go target to be runnable and visible without include-support"
    );
    assert!(
        results.iter().any(|row| {
            row["target"] == "src/service_test.go" && row["target_kind"] == "integration_test_file"
        }),
        "expected runnable go target to use integration_test_file kind"
    );
}

#[test]
fn milestone62_verify_plan_emits_go_targeted_and_full_suite_steps() {
    let repo = common::temp_repo();
    write_go_scope_fixture(repo.path());

    run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let payload = parse_json(&run_stdout(&[
        "verify-plan",
        "--changed-file",
        "src/service.go",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]));
    let results = results(&payload);

    assert!(
        has_step(results, "go test ./src", "targeted"),
        "expected verify-plan to include runnable go targeted command"
    );
    assert!(
        has_step(results, "go test ./...", "full_suite"),
        "expected verify-plan full-suite gate to use go test for go-only changed scope"
    );
}
