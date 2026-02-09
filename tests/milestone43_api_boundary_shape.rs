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
    let required_enums = [
        "pub enum QueryPathMode",
        "pub enum QueryTestMode",
        "pub enum DiffImpactTestMode",
        "pub enum DiffImpactImportMode",
        "pub enum DiffImpactChangedMode",
    ];
    for marker in required_enums {
        assert!(
            query_source.contains(marker),
            "{marker} is required for explicit behavior modes"
        );
    }
    let disallowed_flags = [
        "pub code_only: bool",
        "pub exclude_tests: bool",
        "pub include_tests: bool",
        "pub include_imports: bool",
        "pub exclude_changed: bool",
    ];
    for marker in disallowed_flags {
        assert!(
            !query_source.contains(marker),
            "{marker} must not appear in query option APIs"
        );
    }
}

#[test]
fn milestone43_boundary_counts_use_fixed_width_integers() {
    let cli_source = read_repo_file("src/cli.rs");
    let query_source = read_repo_file("src/query/mod.rs");
    let output_source = read_repo_file("src/output.rs");

    let required_fixed_width = [
        (&cli_source, "pub max_results: Option<u32>"),
        (&cli_source, "pub budget: u32"),
        (&cli_source, "pub max_targeted: Option<u32>"),
        (&query_source, "pub max_results: Option<u32>"),
        (&query_source, "pub max_targeted: Option<u32>"),
        (
            &query_source,
            "pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: u32",
        ),
        (&output_source, "budget: u32"),
    ];
    for (source, marker) in required_fixed_width {
        assert!(source.contains(marker), "{marker} is required");
    }
    let disallowed_usize_markers = [
        (&cli_source, "pub max_results: Option<usize>"),
        (&cli_source, "pub budget: usize"),
        (&cli_source, "pub max_targeted: Option<usize>"),
        (&query_source, "pub max_results: Option<usize>"),
        (&query_source, "pub max_targeted: Option<usize>"),
        (
            &query_source,
            "pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: usize",
        ),
        (&output_source, "budget: usize"),
    ];
    for (source, marker) in disallowed_usize_markers {
        assert!(
            !source.contains(marker),
            "{marker} must not appear in boundary APIs"
        );
    }
}
