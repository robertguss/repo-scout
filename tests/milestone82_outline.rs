mod common;

use serde_json::Value;

#[test]
fn outline_shows_signatures_without_bodies() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub struct Foo {\n    pub x: i32,\n}\n\npub fn bar(a: i32) -> i32 {\n    a * 2\n}\n\nfn baz() {\n    println!(\"hello\");\n}\n",
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "outline",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
    ]);

    assert!(output.contains("Foo"), "should list struct Foo:\n{output}");
    assert!(
        output.contains("bar"),
        "should list function bar:\n{output}"
    );
    assert!(
        output.contains("baz"),
        "should list function baz:\n{output}"
    );
    // Should NOT contain implementation details
    assert!(
        !output.contains("a * 2"),
        "should not contain function bodies:\n{output}"
    );
    assert!(
        !output.contains("println"),
        "should not contain function bodies:\n{output}"
    );
}

#[test]
fn outline_json_output() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "outline",
        "src/lib.rs",
        "--repo",
        repo.path().to_str().unwrap(),
        "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    assert_eq!(payload["command"], "outline");
    assert!(payload["results"].as_array().unwrap().len() > 0);
}
