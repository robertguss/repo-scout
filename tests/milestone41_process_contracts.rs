use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn read_repo_file(path: &str) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{repo_root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|err| {
        panic!("failed to read {full_path}: {err}");
    })
}

fn run_command(cwd: &Path, program: &str, args: &[&str]) -> Output {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {program} {:?}: {err}", args))
}

fn git(repo: &Path, args: &[&str]) {
    let output = run_command(repo, "git", args);
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(repo: &Path, args: &[&str]) -> String {
    let output = run_command(repo, "git", args);
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output should be utf-8")
        .trim()
        .to_string()
}

fn init_git_repo() -> TempDir {
    let tempdir = tempfile::tempdir().expect("temp repo should be created");
    let repo = tempdir.path();

    git(repo, &["init", "-q"]);
    git(repo, &["config", "user.name", "Repo Scout Test"]);
    git(
        repo,
        &["config", "user.email", "repo-scout-test@example.com"],
    );

    fs::write(repo.join("README.md"), "fixture\n").expect("readme should be written");
    git(repo, &["add", "README.md"]);
    git(repo, &["commit", "-q", "-m", "DOCS: seed repository"]);

    tempdir
}

fn write_commit(repo: &Path, path: &str, contents: &str, subject: &str) {
    let full_path = repo.join(path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("parent dirs should exist");
    }
    fs::write(&full_path, contents).expect("fixture file should be written");
    git(repo, &["add", path]);
    git(repo, &["commit", "-q", "-m", subject]);
}

fn run_tdd_validator(repo: &Path, base_ref: &str) -> Output {
    let script = format!(
        "{}/scripts/validate_tdd_cycle.sh",
        env!("CARGO_MANIFEST_DIR")
    );

    run_command(repo, "bash", &[&script, "--base", base_ref])
}

fn run_evidence_validator(repo: &Path, args: &[&str]) -> Output {
    let script = format!(
        "{}/scripts/validate_evidence_packet.sh",
        env!("CARGO_MANIFEST_DIR")
    );

    let mut full_args = vec![script];
    full_args.extend(args.iter().map(|arg| (*arg).to_string()));
    let refs: Vec<&str> = full_args.iter().map(String::as_str).collect();
    run_command(repo, "bash", &refs)
}

fn run_evidence_validator_on_file(repo: &Path, evidence_path: &Path) -> Output {
    run_evidence_validator(
        repo,
        &[
            "--file",
            evidence_path.to_str().expect("path should be utf-8"),
        ],
    )
}

#[test]
fn milestone41_ci_workflow_enforces_rust_contract_gates() {
    let workflow = read_repo_file(".github/workflows/contract-gates.yml");

    let required_snippets = [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features -- \\",
        "-D warnings",
        "-D clippy::unwrap_used",
        "-D clippy::expect_used",
        "-D clippy::undocumented_unsafe_blocks",
        "cargo test --workspace --all-features",
    ];

    for snippet in required_snippets {
        assert!(
            workflow.contains(snippet),
            "missing required CI gate snippet: {snippet}"
        );
    }
}

#[test]
fn milestone41_tdd_validator_rejects_empty_commit_range() {
    let repo = init_git_repo();
    let repo_path = repo.path();
    let head = git_stdout(repo_path, &["rev-parse", "HEAD"]);

    let output = run_tdd_validator(repo_path, &head);
    assert!(
        !output.status.success(),
        "expected failure for empty commit range\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn milestone41_tdd_validator_rejects_green_without_new_red_cycle() {
    let repo = init_git_repo();
    let repo_path = repo.path();
    let base = git_stdout(repo_path, &["rev-parse", "HEAD"]);

    write_commit(
        repo_path,
        "tests/m41_red.rs",
        "#[test]\nfn red() { assert!(false); }\n",
        "RED: add failing regression test",
    );
    write_commit(
        repo_path,
        "src/lib.rs",
        "pub fn green_one() -> u32 { 1 }\n",
        "GREEN: implement first behavior slice",
    );
    write_commit(
        repo_path,
        "src/lib.rs",
        "pub fn green_one() -> u32 {\n    1\n}\n",
        "REFACTOR: improve formatting",
    );
    write_commit(
        repo_path,
        "src/lib.rs",
        "pub fn green_one() -> u32 {\n    2\n}\n",
        "GREEN: mutate behavior without a new red stage",
    );

    let output = run_tdd_validator(repo_path, &base);
    assert!(
        !output.status.success(),
        "expected failure for GREEN without a new RED cycle\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn milestone41_evidence_validator_rejects_headings_without_quality_fields() {
    let repo = tempfile::tempdir().expect("temp dir should be created");
    let evidence_path = repo.path().join("evidence.md");

    let evidence = r#"## Objective
Objective text.

## Risk Tier
Tier one.

## Scope
Scope text.

## Red
Tests failed.

## Green
Implementation passed tests.

## Refactor
Code was cleaned up.

## Invariants
Invariants listed.

## Security Impact
Security impact listed.

## Performance Impact
Performance impact listed.

## Assumptions
One assumption.

## Open Questions
No questions.

## Rollback Plan
Rollback plan text.

## Validation Commands
```bash
cargo test
```
"#;

    fs::write(&evidence_path, evidence).expect("evidence should be written");
    let output = run_evidence_validator_on_file(repo.path(), &evidence_path);

    assert!(
        !output.status.success(),
        "expected quality validation failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn milestone41_evidence_validator_accepts_detailed_evidence_sections() {
    let repo = tempfile::tempdir().expect("temp dir should be created");
    let evidence_path = repo.path().join("evidence.md");

    let evidence = r#"## Objective
- Problem solved: tighten validator quality checks.
- Intended outcome: reject low-detail evidence packets.

## Risk Tier
- Tier: `1`
- Rationale: process-only validator behavior updates.

## Scope
- Files/components changed: scripts/validate_evidence_packet.sh
- Explicit exclusions: runtime query/index paths

## Red
- Failing test(s): milestone41_evidence_validator_rejects_headings_without_quality_fields
- Command(s): cargo test milestone41_evidence_validator -- --nocapture
- Expected failure summary: evidence headings exist but quality fields are missing.
- Why this failure is expected: heading-only evidence omits required semantic details.

## Green
- Minimal implementation summary: added strict section content checks.
- Command(s): cargo test milestone41_evidence_validator -- --nocapture
- Passing summary: evidence validation now fails low-detail packets.

## Refactor
- Structural improvements: extracted reusable section-check helper functions.
- Why behavior is unchanged: heading requirements and placeholder checks remain intact.
- Confirmation commands: cargo test

## Invariants
- Invariants added/updated: Red/Green/Refactor sections must contain concrete details.
- Boundary checks added/updated: template scaffold mode is recognized by filename.

## Security Impact
- Threats considered: malformed evidence could bypass process controls.
- Mitigations: semantic checks require concrete failure/pass details.
- Residual risk: reviewers still must validate truthfulness of statements.

## Performance Impact
- Baseline: negligible runtime for small markdown files.
- Post-change: negligible runtime remains.
- Delta explanation: grep/awk checks are linear in file size.

## Assumptions
1. PR bodies use the required section headings.

## Open Questions
1. None.

## Rollback Plan
- Trigger conditions: validator false positives block legitimate PRs.
- Rollback steps: revert strict checks and keep heading-only validation.

## Validation Commands
```bash
cargo test milestone41_evidence_validator -- --nocapture
```
"#;

    fs::write(&evidence_path, evidence).expect("evidence should be written");
    let output = run_evidence_validator_on_file(repo.path(), &evidence_path);

    assert!(
        output.status.success(),
        "expected detailed evidence to pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn milestone41_pr_template_includes_checklist_and_exception_attestations() {
    let template = read_repo_file(".github/pull_request_template.md");

    let required_snippets = [
        "checklists/PR_CONTRACT_CHECKLIST.md",
        "checklists/ADVERSARIAL_REVIEW_CHECKLIST.md",
        "exception",
        "## Dogfooding Evidence (repo-scout required)",
        "## Docs and Plans",
    ];

    for snippet in required_snippets {
        assert!(
            template.contains(snippet),
            "missing required PR template snippet: {snippet}"
        );
    }
}
