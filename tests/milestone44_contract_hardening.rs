use std::fs;

fn read_repo_file(path: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{repo_root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|err| {
        panic!("failed to read {full_path}: {err}");
    })
}

fn assert_has_must_use_annotation(source: &str, function_name: &str) {
    let marker = format!("#[must_use]\npub fn {function_name}");
    assert!(
        source.contains(&marker),
        "expected #[must_use] on public function '{function_name}'"
    );
}

#[test]
fn milestone44_public_query_contract_apis_are_must_use() {
    let query_source = read_repo_file("src/query/mod.rs");
    for function_name in [
        "find_matches_scoped",
        "refs_matches_scoped",
        "context_matches_scoped",
        "diff_impact_for_changed_files",
        "verify_plan_for_changed_files",
    ] {
        assert_has_must_use_annotation(&query_source, function_name);
    }
}

#[test]
fn milestone44_query_boundaries_include_targeted_invariant_assertions() {
    let query_source = read_repo_file("src/query/mod.rs");
    for marker in [
        "usize::BITS >= 32",
        "debug_assert!(max_results >= 1",
        "results.len() <= bounded_usize(max_results)",
    ] {
        assert!(
            query_source.contains(marker),
            "missing invariant marker: {marker}"
        );
    }
}

#[test]
fn milestone44_touched_modules_respect_line_length_limit() {
    let max_columns = 100;
    for path in ["src/main.rs", "src/query/mod.rs", "src/output.rs"] {
        let source = read_repo_file(path);
        for (line_number, line) in source.lines().enumerate() {
            let width = line.chars().count();
            assert!(
                width <= max_columns,
                "{path}:{} exceeds {max_columns} columns (found {width})",
                line_number + 1
            );
        }
    }
}
