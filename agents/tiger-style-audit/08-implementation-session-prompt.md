# Implementation Prompt For New Session

You are implementing Tiger-style compliance remediation for `repo-scout` using the completed audit
artifacts.

## Objective

Implement the Tiger-style conformance fixes identified in:

- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/README.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/02-contract-installation-drift.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/03-src-compliance-report-and-plan.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/04-tests-compliance-report-and-plan.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/05-process-ci-docs-compliance-report-and-plan.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/06-tiger-style-framework-feedback.md`
  (use only for local notes and upstream issue extraction, not local contract downgrades)

## Constraints (must follow)

1. Read and follow:
   - `/Users/robertguss/Projects/experiments/repo-scout/AGENTS.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/contracts/core/*.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/contracts/languages/RUST_CODING_CONTRACT.md`
2. Strict Red -> Green -> Refactor per feature slice.
3. Declare risk tier before each implementation milestone.
4. Do not weaken Tiger requirements to make implementation easier.
5. Keep behavior stable unless a finding explicitly requires behavior change.
6. Never rewrite git history unless explicitly requested.

## Required Execution Order

### Milestone 1 (P1 process hardening)

Implement the P1 process/CI gaps first:

- F-PROC-01: enforce Rust contract required gates in CI (`fmt`, strict `clippy`, `test`).
- F-PROC-02: strengthen `scripts/validate_tdd_cycle.sh` so it does more than minimal prefix
  presence.
- F-PROC-03: strengthen `scripts/validate_evidence_packet.sh` with content-quality checks.
- F-PROC-07: merge PR template checklist + dogfooding expectations.

### Milestone 2 (P1/P2 source structure)

Start structural compliance on highest-impact source hotspots:

- F-SRC-01: reduce top oversized production functions toward <=70 lines.
- F-SRC-02: extract orchestration stages for index/query hotspots.
- F-SRC-03: formalize recursion approval or replace unapproved recursion.

### Milestone 3 (P2 boundary/API shape)

- F-SRC-04: replace behavior boolean clusters with explicit modes/enums where practical.
- F-SRC-05: migrate public boundary `usize` fields to fixed-width integer types.
- F-SRC-06/F-SRC-07: add targeted invariant checks and line-length cleanup.

### Milestone 4 (tests/docs/process normalization)

- F-TEST-01/F-TEST-02: split oversized tests and resolve 100-column drift.
- F-TEST-03/F-TEST-04: codify explicit test policy for `unwrap`/`expect`/`panic` in `AGENTS.md`.
- F-PROC-05/F-PROC-06: normalize plan/process artifact policy and legacy plan labeling.

## Execution Protocol

For each feature slice:

1. Red:
   - add/update failing tests first,
   - run only the targeted test command and confirm expected failure.
2. Green:
   - implement minimal production change,
   - rerun targeted tests to pass.
3. Refactor:
   - improve structure while preserving behavior,
   - run full validation gates.

## Commands You Must Run

Pre-work dogfooding:

- `cargo run -- index --repo .`
- `cargo run -- find <target_symbol> --repo . --json`
- `cargo run -- refs <target_symbol> --repo . --json`

Per-slice validation:

- targeted test command(s)
- `cargo test`

Milestone gates:

- `just check`
- `just contract-check`
- `bash scripts/validate_tdd_cycle.sh --base origin/main`
- `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

Post-change dogfooding:

- `cargo run -- index --repo .`
- `cargo run -- find <target_symbol> --repo .`
- `cargo run -- refs <target_symbol> --repo .`
- `cargo test`

## Deliverables

1. Implemented code and config changes.
2. Updated docs/process artifacts when behavior or policy changes.
3. A concise implementation report that maps each completed change back to finding IDs.
4. A remaining-work list for any unfinished findings, ordered by severity.

## Important

If a Tiger rule appears contradictory or impractical during implementation:

- do not silently bypass it,
- document the conflict in local notes,
- keep local repo compliant with current installed rules,
- add an upstream follow-up note referencing:
  `/Users/robertguss/Projects/experiments/repo-scout/agents/tiger-style-audit/06-tiger-style-framework-feedback.md`.
