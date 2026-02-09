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

fn function_body<'a>(source: &'a str, function_name: &str) -> Option<&'a str> {
    let (start, end) = brace_span(source, function_name)?;
    source.get(start + 1..end)
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

fn assert_no_self_call(path: &str, function_name: &str) {
    let source = read_repo_file(path);
    let body = function_body(&source, function_name).unwrap_or_else(|| {
        panic!("function '{function_name}' not found in {path}");
    });
    let call_pattern = format!("{function_name}(");
    assert!(
        !body.contains(&call_pattern),
        "function '{function_name}' in {path} contains self-recursive call"
    );
}

#[test]
fn milestone42_hotspot_functions_fit_size_limit() {
    let max_lines = 70;
    assert_function_length("src/indexer/mod.rs", "index_repository", max_lines);
    assert_function_length("src/indexer/mod.rs", "resolve_symbol_id_in_tx", max_lines);
    assert_function_length("src/query/mod.rs", "context_matches_scoped", max_lines);
    assert_function_length(
        "src/query/mod.rs",
        "diff_impact_for_changed_files",
        max_lines,
    );
    assert_function_length(
        "src/query/mod.rs",
        "verify_plan_for_changed_files",
        max_lines,
    );
}

#[test]
fn milestone42_unapproved_recursive_helpers_are_removed() {
    assert_no_self_call("src/indexer/rust_ast.rs", "collect_call_identifiers");
    assert_no_self_call("src/indexer/rust_ast.rs", "last_identifier_text");
    assert_no_self_call(
        "src/indexer/languages/typescript.rs",
        "collect_type_identifiers",
    );
}
