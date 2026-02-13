mod common;

use serde_json::Value;

fn setup_repo() -> tempfile::TempDir {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        r#"
pub fn public_api() -> i32 {
    private_impl()
}

fn private_impl() -> i32 {
    42
}
"#,
    );
    common::run_stdout(&[
        "index",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
    ]);
    repo
}

#[test]
fn milestone106_boundary_public_only_text_and_json_are_strict() {
    let repo = setup_repo();

    let text = common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--public-only",
    ]);
    assert!(text.contains("public_api"), "output: {text}");
    assert!(!text.contains("private_impl"), "output: {text}");

    let out = common::run_stdout(&[
        "boundary",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().expect("repo path utf-8"),
        "--public-only",
        "--json",
    ]);
    let json: Value = serde_json::from_str(&out).expect("boundary json");
    assert!(
        json["report"]["internal_symbols"]
            .as_array()
            .expect("internal symbols")
            .is_empty(),
        "output: {out}"
    );
}
