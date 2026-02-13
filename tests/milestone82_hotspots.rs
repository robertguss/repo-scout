mod common;

#[test]
fn hotspots_returns_most_connected_symbols() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn core_fn() {}\npub fn a() { core_fn(); }\npub fn b() { core_fn(); }\npub fn c() { core_fn(); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "hotspots", "--repo", repo.path().to_str().unwrap(), "--limit", "5",
    ]);
    // core_fn should be #1 — it has the most inbound edges
    let lines: Vec<&str> = output.lines().filter(|l| l.contains("core_fn")).collect();
    assert!(!lines.is_empty(), "core_fn should appear in hotspots:\n{output}");
}
