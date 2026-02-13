mod common;

use std::fs;

fn read_agents_text_lowercase() -> String {
    let agents_path = common::repo_root().join("AGENTS.md");
    fs::read_to_string(&agents_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", agents_path.display()))
        .to_lowercase()
}

fn assert_agents_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "AGENTS.md should contain policy text matching: {needle}"
    );
}

#[test]
fn milestone46_agents_documents_unwrap_expect_policy_for_tests() {
    let agents = read_agents_text_lowercase();
    for expected in [
        "test error-handling policy",
        "tests/",
        "unwrap",
        "expect",
        "allowed",
    ] {
        assert_agents_contains(&agents, expected);
    }
}

#[test]
fn milestone46_agents_documents_panic_policy_and_src_boundary() {
    let agents = read_agents_text_lowercase();
    for expected in ["tests/common", "panic!", "src/", "must not"] {
        assert_agents_contains(&agents, expected);
    }
}
