use std::fs;

fn read_repo_file(path: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{repo_root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|err| {
        panic!("failed to read {full_path}: {err}");
    })
}

fn find_function_header_offset(source: &str, name: &str) -> Option<usize> {
    let pattern = format!("fn {name}");
    source.find(&pattern)
}

fn brace_span(source: &str, function_name: &str) -> Option<(usize, usize)> {
    let header = find_function_header_offset(source, function_name)?;
    let bytes = source.as_bytes();
    let mut i = header;

    while i < bytes.len() && bytes[i] != b'{' {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }

    let start = i;
    let mut depth = 0_u32;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied().unwrap_or_default();

        if in_line_comment {
            if b == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }

        if in_block_comment {
            if b == b'*' && next == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        if in_string {
            if !escaped && b == b'"' {
                in_string = false;
            }
            escaped = !escaped && b == b'\\';
            i += 1;
            continue;
        }

        if in_char {
            if !escaped && b == b'\'' {
                in_char = false;
            }
            escaped = !escaped && b == b'\\';
            i += 1;
            continue;
        }

        if b == b'/' && next == b'/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if b == b'/' && next == b'*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if b == b'"' {
            in_string = true;
            escaped = false;
            i += 1;
            continue;
        }
        if b == b'\'' {
            in_char = true;
            escaped = false;
            i += 1;
            continue;
        }

        if b == b'{' {
            depth += 1;
        } else if b == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some((start, i));
            }
        }
        i += 1;
    }
    None
}

fn function_line_len(source: &str, function_name: &str) -> Option<usize> {
    let (start, end) = brace_span(source, function_name)?;
    let start_line = source[..start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let end_line = source[..=end].bytes().filter(|byte| *byte == b'\n').count() + 1;
    Some(end_line - start_line + 1)
}

fn assert_function_length(path: &str, function_name: &str, max_lines: usize) {
    let source = read_repo_file(path);
    let actual_len = function_line_len(&source, function_name).unwrap_or_else(|| {
        panic!("function '{function_name}' not found in {path}");
    });
    assert!(
        actual_len <= max_lines,
        "function '{function_name}' in {path} is {actual_len} lines; max is {max_lines}"
    );
}

#[test]
fn milestone45_hotspot_test_functions_fit_size_limit() {
    let max_lines = 70;
    let bounded_functions = [
        (
            "tests/milestone27_context_scope.rs",
            "milestone27_context_scope_flags_preserve_deterministic_json",
        ),
        (
            "tests/milestone16_python.rs",
            "milestone16_python_edges_and_queries",
        ),
        (
            "tests/milestone23_verify_plan_precision.rs",
            "milestone23_verify_plan_applies_targeted_cap_deterministically",
        ),
        (
            "tests/milestone12_diff_impact.rs",
            "milestone12_diff_impact_deterministic_ordering",
        ),
        (
            "tests/milestone30_query_focus.rs",
            "milestone30_find_and_refs_max_results_cap_deterministically",
        ),
        ("tests/milestone14_adapter.rs", "build_v2_index"),
        (
            "tests/milestone30_query_focus.rs",
            "milestone30_query_caps_compose_with_code_only_and_exclude_tests",
        ),
    ];
    for (path, function_name) in bounded_functions {
        assert_function_length(path, function_name, max_lines);
    }
}

#[test]
fn milestone45_test_hotspot_files_fit_line_length_limit() {
    let max_columns = 100;
    for path in [
        "tests/milestone27_context_scope.rs",
        "tests/milestone7_rust_symbols.rs",
        "tests/milestone6_schema_migration.rs",
        "tests/milestone6_lifecycle.rs",
    ] {
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
