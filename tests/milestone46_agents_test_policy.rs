use std::fs;

fn read_agents_text_lowercase() -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let agents_path = format!("{repo_root}/AGENTS.md");
    fs::read_to_string(&agents_path)
        .unwrap_or_else(|err| panic!("failed to read {agents_path}: {err}"))
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
    assert_agents_contains(&agents, "test error-handling policy");
    assert_agents_contains(&agents, "tests/");
    assert_agents_contains(&agents, "unwrap");
    assert_agents_contains(&agents, "expect");
    assert_agents_contains(&agents, "allowed");
}

#[test]
fn milestone46_agents_documents_panic_policy_and_src_boundary() {
    let agents = read_agents_text_lowercase();
    assert_agents_contains(&agents, "tests/common");
    assert_agents_contains(&agents, "panic!");
    assert_agents_contains(&agents, "src/");
    assert_agents_contains(&agents, "must not");
}
