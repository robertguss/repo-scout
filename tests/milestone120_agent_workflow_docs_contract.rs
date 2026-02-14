mod common;

#[test]
fn milestone120_docs_cover_agent_first_workflows() {
    let cli_ref = common::read_repo_file("docs/cli-reference.md");
    let json_doc = common::read_repo_file("docs/json-output.md");
    let playbook = common::read_repo_file("docs/agent-playbook-codex.md");

    for required in [
        "schema",
        "resolve",
        "query",
        "refactor-plan",
        "--require-index-fresh",
        "--auto-index",
    ] {
        assert!(
            cli_ref.contains(required),
            "docs/cli-reference.md must document `{required}`"
        );
    }

    for required in [
        "repo-scout/find@v1",
        "repo-scout/error@v1",
        "ok",
        "meta",
        "data",
    ] {
        assert!(
            json_doc.contains(required),
            "docs/json-output.md must include `{required}`"
        );
    }

    for required in [
        "repo-scout schema --repo . --json",
        "repo-scout resolve",
        "repo-scout query",
        "repo-scout refactor-plan",
    ] {
        assert!(
            playbook.contains(required),
            "docs/agent-playbook-codex.md must include `{required}`"
        );
    }
}
