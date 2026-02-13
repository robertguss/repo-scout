mod common;

#[test]
fn help_text_shows_subcommand_descriptions() {
    let output = common::run_stdout(&["--help"]);
    assert!(
        output.contains("Index a repository"),
        "missing Index description:\n{output}"
    );
    assert!(
        output.contains("Show index status"),
        "missing Status description:\n{output}"
    );
    assert!(
        output.contains("Find symbol definitions"),
        "missing Find description:\n{output}"
    );
    assert!(
        output.contains("Find all references"),
        "missing Refs description:\n{output}"
    );
    assert!(
        output.contains("Show what depends on"),
        "missing Impact description:\n{output}"
    );
    assert!(
        output.contains("Find code relevant to"),
        "missing Context description:\n{output}"
    );
    assert!(
        output.contains("Find test files"),
        "missing TestsFor description:\n{output}"
    );
    assert!(
        output.contains("Suggest test commands"),
        "missing VerifyPlan description:\n{output}"
    );
    assert!(
        output.contains("Analyze blast radius"),
        "missing DiffImpact description:\n{output}"
    );
    assert!(
        output.contains("Show symbol details"),
        "missing Explain description:\n{output}"
    );
}
