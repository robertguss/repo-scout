mod common;

#[test]
fn milestone80_runner_uses_portable_json_escaping_for_newlines() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        !script.contains(":a;N;$!ba;s/\\n/\\\\n/g")
            && script.contains("tr '\\n' '\\r'")
            && script.contains("s/\\r/\\\\n/g"),
        "runner must avoid non-portable sed label loops and use portable newline escaping"
    );
}

#[test]
fn milestone80_runner_issue_log_formatting_does_not_execute_command_substitution() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        !script.contains("echo \"  - command: \\\\`$command\\\\`\"")
            && !script.contains("echo \"  - args: \\\\`$args\\\\`\"")
            && script.contains("printf '  - command: `%s`\\n' \"$command\"")
            && script.contains("printf '  - args: `%s`\\n' \"$args\""),
        "runner must format command and args using printf placeholders to avoid backtick substitution"
    );
}

#[test]
fn milestone80_runner_diff_impact_matrix_uses_empty_default_flag_not_literal_word() {
    let script = common::read_repo_file("scripts/run_e2e_release_matrix.sh");
    assert!(
        !script.contains("local test_modes=(\"default\"")
            && script.contains("local test_modes=(\"\" \"--include-tests\" \"--exclude-tests\")"),
        "runner diff-impact matrix must represent the default mode with an empty flag, not a literal token"
    );
}
