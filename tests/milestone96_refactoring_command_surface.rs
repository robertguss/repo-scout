mod common;

#[test]
fn milestone96_top_level_help_lists_refactoring_subcommands() {
    let mut cmd = common::repo_scout_cmd();
    cmd.arg("--help");
    let output = cmd.assert().success().get_output().stdout.clone();
    let help_text = String::from_utf8(output).expect("help output should be utf-8");

    for subcommand in [
        "anatomy",
        "coupling",
        "dead",
        "test-gaps",
        "suggest",
        "boundary",
        "extract-check",
        "move-check",
        "rename-check",
        "split-check",
        "test-scaffold",
        "safe-steps",
        "verify-refactor",
    ] {
        assert!(
            help_text.contains(subcommand),
            "missing subcommand '{subcommand}' in --help output:\n{help_text}"
        );
    }
}

#[test]
fn milestone96_health_help_lists_baseline_flags() {
    let mut cmd = common::repo_scout_cmd();
    cmd.args(["health", "--help"]);
    let output = cmd.assert().success().get_output().stdout.clone();
    let help_text = String::from_utf8(output).expect("help output should be utf-8");

    assert!(
        help_text.contains("--save-baseline"),
        "health --help missing --save-baseline:\n{help_text}"
    );
    assert!(
        help_text.contains("--diff"),
        "health --help missing --diff:\n{help_text}"
    );
}
