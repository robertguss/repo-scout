use std::fs;

fn read_repo_file(path: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{repo_root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|err| {
        panic!("failed to read {full_path}: {err}");
    })
}

#[test]
fn milestone43_query_options_use_explicit_modes() {
    let query_source = read_repo_file("src/query/mod.rs");
    assert!(
        query_source.contains("pub enum QueryPathMode"),
        "QueryPathMode enum is required for query scope behavior"
    );
    assert!(
        query_source.contains("pub enum QueryTestMode"),
        "QueryTestMode enum is required for query scope behavior"
    );
    assert!(
        query_source.contains("pub enum DiffImpactTestMode"),
        "DiffImpactTestMode enum is required for diff-impact behavior"
    );
    assert!(
        query_source.contains("pub enum DiffImpactImportMode"),
        "DiffImpactImportMode enum is required for diff-impact behavior"
    );
    assert!(
        query_source.contains("pub enum DiffImpactChangedMode"),
        "DiffImpactChangedMode enum is required for diff-impact behavior"
    );
    assert!(
        !query_source.contains("pub code_only: bool"),
        "QueryScope must not expose behavior booleans"
    );
    assert!(
        !query_source.contains("pub exclude_tests: bool"),
        "QueryScope must not expose behavior booleans"
    );
    assert!(
        !query_source.contains("pub include_tests: bool"),
        "DiffImpactOptions must not expose behavior booleans"
    );
    assert!(
        !query_source.contains("pub include_imports: bool"),
        "DiffImpactOptions must not expose behavior booleans"
    );
    assert!(
        !query_source.contains("pub exclude_changed: bool"),
        "DiffImpactOptions must not expose behavior booleans"
    );
}

#[test]
fn milestone43_boundary_counts_use_fixed_width_integers() {
    let cli_source = read_repo_file("src/cli.rs");
    let query_source = read_repo_file("src/query/mod.rs");
    let output_source = read_repo_file("src/output.rs");

    assert!(
        cli_source.contains("pub max_results: Option<u32>"),
        "cli max-results fields must use Option<u32>"
    );
    assert!(
        cli_source.contains("pub budget: u32"),
        "cli budget field must use u32"
    );
    assert!(
        cli_source.contains("pub max_targeted: Option<u32>"),
        "cli max-targeted field must use Option<u32>"
    );
    assert!(
        !cli_source.contains("pub max_results: Option<usize>"),
        "cli must not expose usize max-results boundary fields"
    );
    assert!(
        !cli_source.contains("pub budget: usize"),
        "cli must not expose usize budget boundary fields"
    );
    assert!(
        !cli_source.contains("pub max_targeted: Option<usize>"),
        "cli must not expose usize max-targeted boundary fields"
    );

    assert!(
        query_source.contains("pub max_results: Option<u32>"),
        "query options max-results must use Option<u32>"
    );
    assert!(
        query_source.contains("pub max_targeted: Option<u32>"),
        "verify-plan options max-targeted must use Option<u32>"
    );
    assert!(
        query_source.contains("pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: u32"),
        "verify-plan max-targeted default constant must use u32"
    );
    assert!(
        !query_source.contains("pub max_results: Option<usize>"),
        "query options must not expose usize max-results boundary fields"
    );
    assert!(
        !query_source.contains("pub max_targeted: Option<usize>"),
        "verify-plan options must not expose usize max-targeted boundary fields"
    );
    assert!(
        !query_source.contains("pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: usize"),
        "verify-plan default constant must not use usize"
    );

    assert!(
        output_source.contains("budget: u32"),
        "output JSON context payload budget must use u32"
    );
    assert!(
        !output_source.contains("budget: usize"),
        "output module must not expose usize budget fields"
    );
}
