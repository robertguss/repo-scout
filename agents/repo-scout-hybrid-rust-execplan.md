# Build `repo-scout` v0 Hybrid CLI with Strict TDD (Rust First)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `/Users/robertguss/Projects/experiments/codex-5-3/agents/PLANS.md`, and this document must be maintained in accordance with that file.

## Purpose / Big Picture

This plan delivers a useful first release of `repo-scout` without waiting for multi-language semantic engines. After implementation, a user can index any repository, run `find` and `refs`, and get deterministic results with explicit confidence and provenance labels. Rust files receive stronger structural extraction through Tree-sitter AST parsing, while all files still work through text fallback. The result is useful for humans and coding agents because output can be read in the terminal or consumed in stable JSON.

This implementation is intentionally strict Test-Driven Development (TDD). Every feature is built in a red-green-refactor loop: first write a failing test (red), then write the minimum code to pass the test (green), then improve the code while keeping tests green (refactor). No production feature code is added before the corresponding failing test exists.

## Progress

- [x] (2026-02-06 00:00Z) Created initial ExecPlan document aligned to `/Users/robertguss/Projects/experiments/codex-5-3/agents/PLANS.md`.
- [x] (2026-02-06 00:00Z) Tightened scope to a v0-only deliverable and encoded strict red-green-refactor workflow.
- [x] (2026-02-06 01:27Z) Set up dependency baseline and testing harness for CLI and indexing flows (`Cargo.toml` dependencies, `tests/common/mod.rs`, `tests/harness_smoke.rs`).
- [x] (2026-02-06 01:30Z) Milestone 1 (red-green-refactor): command surface and SQLite schema bootstrap completed with module refactor (`src/cli.rs`, `src/store/mod.rs`, `src/store/schema.rs`, `src/output.rs`, `src/main.rs`, `tests/milestone1_cli.rs`).
- [ ] Milestone 2 (red-green-refactor): language-agnostic indexing and incremental updates.
- [ ] Milestone 3 (red-green-refactor): Rust Tree-sitter adapter for definitions and references.
- [ ] Milestone 4 (red-green-refactor): query ranking, deterministic ordering, and JSON contract.
- [ ] Milestone 5 (red-green-refactor): end-to-end validation, fixtures, and regression hardening.
- [x] (2026-02-06 01:30Z) Updated living sections with Milestone 1 evidence, decisions, and outcomes.

## Surprises & Discoveries

- Observation: `assert_cmd::Command::cargo_bin` emits a deprecation warning in integration tests.
  Evidence: Initial smoke test run produced a warning recommending `cargo::cargo_bin_cmd!`; switching to that macro removed the warning.

- Observation: Refactoring from Hello World to clap CLI changed zero-argument behavior and broke the existing smoke test expectation.
  Evidence: `cargo test` failed because test expected `Hello, world!`, while clap correctly returned usage output and exit code 2; test was updated to assert `--help` output instead.

## Decision Log

- Decision: Focus this document on a tight v0 scope (`index`, `status`, `find`, `refs`) and defer semantic compiler integration.
  Rationale: The smallest useful release should be shippable quickly and verifiable end-to-end.
  Date/Author: 2026-02-06 / Codex

- Decision: Require strict red-green-refactor cycles for every milestone and feature slice.
  Rationale: TDD reduces regressions, keeps code smaller, and forces observable behavior design before implementation.
  Date/Author: 2026-02-06 / Codex

- Decision: Ship hybrid behavior by combining text fallback for all files with Rust AST extraction as the first language adapter.
  Rationale: This maximizes immediate utility while still proving the adapter architecture.
  Date/Author: 2026-02-06 / Codex

- Decision: Use `assert_cmd::cargo::cargo_bin_cmd!` in test helpers instead of deprecated `Command::cargo_bin`.
  Rationale: Keeps test output warning-free and aligned with current assert_cmd guidance.
  Date/Author: 2026-02-06 / Codex

- Decision: Keep the smoke test as a harness verification test by asserting `--help` output rather than default no-arg output.
  Rationale: No-arg behavior is now clap usage/error, so `--help` is the stable command for smoke verification.
  Date/Author: 2026-02-06 / Codex

## Outcomes & Retrospective

Milestone 1 outcome: The CLI now supports `index`, `status`, `find`, and `refs`, and bootstraps a SQLite store at `.repo-scout/index.db` with schema metadata. Behavior is currently minimal by design (`find`/`refs` return zero results), but command wiring, schema bootstrap, and output surface are in place for subsequent milestones. The next milestone remains language-agnostic indexing and incremental behavior.

## Context and Orientation

The repository is a fresh Cargo binary project. The initial files are `/Users/robertguss/Projects/experiments/codex-5-3/Cargo.toml` and `/Users/robertguss/Projects/experiments/codex-5-3/src/main.rs`. This plan introduces a multi-module CLI in `/Users/robertguss/Projects/experiments/codex-5-3/src` and adds integration tests under `/Users/robertguss/Projects/experiments/codex-5-3/tests`.

An Abstract Syntax Tree (AST) is a tree representation of source code syntax (for example function declarations and import statements). Tree-sitter provides syntax structure, not full compiler-level semantic resolution. Hybrid indexing in this project means every file is searchable through text fallback while Rust files receive additional AST-derived entries. Confidence labels (`ast_exact`, `ast_likely`, `text_fallback`) and provenance labels (`ast_definition`, `ast_reference`, `text_identifier_match`) make certainty explicit to both humans and agents.

The core implementation targets these modules: `/Users/robertguss/Projects/experiments/codex-5-3/src/cli.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/indexer/mod.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/indexer/files.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/indexer/text.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/indexer/rust_ast.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/store/mod.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/store/schema.rs`, `/Users/robertguss/Projects/experiments/codex-5-3/src/query/mod.rs`, and `/Users/robertguss/Projects/experiments/codex-5-3/src/output.rs`.

## Plan of Work

Milestone 1 creates the CLI skeleton and database bootstrap through strict TDD. Begin by writing failing integration tests that invoke `repo-scout index`, `repo-scout status`, `repo-scout find`, and `repo-scout refs` and assert command parsing plus basic response shapes. Only after those tests fail should implementation begin. The green step should implement minimal command dispatch, SQLite connection, and schema creation. The refactor step should isolate parsing, storage, and output concerns into modules and keep all tests green.

Milestone 2 delivers language-agnostic indexing and incremental behavior through strict TDD. Start with failing tests against fixture repositories asserting that unchanged files are skipped on a second `index` run and that text fallback enables `find` and `refs` in non-Rust files. Implement only the minimum file walk, hashing, and token extraction needed for green tests. Refactor query and indexing boundaries for readability and deterministic behavior.

Milestone 3 adds Rust AST extraction through strict TDD. Start with failing fixture tests asserting that Rust definitions and references are discovered from syntax nodes and labeled with AST provenance and confidence. Implement Tree-sitter parsing for Rust files and persist extracted symbols/references while preserving text fallback. Refactor extraction code for clear node traversal and explicit confidence decisions.

Milestone 4 finalizes ranking and JSON output stability through strict TDD. Start with failing golden tests that lock JSON schema shape, field names, deterministic result ordering, and ranking rules. Implement exact-name-first ranking and explicit `why_matched` reporting, then pass tests with minimal logic. Refactor ranking and serialization into dedicated units without changing observable output.

Milestone 5 hardens and proves behavior through strict TDD. Start with failing end-to-end regression tests covering full index/query flows and one corrupt-index recovery path. Implement only missing functionality required by failing tests, then refactor for maintainability and test clarity. Record final evidence and retrospective notes in this document.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/codex-5-3`.

Install dependencies once:

    cargo add clap --features derive
    cargo add anyhow thiserror
    cargo add serde --features derive
    cargo add serde_json
    cargo add rusqlite --features bundled
    cargo add ignore blake3 tree-sitter tree-sitter-rust
    cargo add --dev assert_cmd predicates tempfile insta

For each feature slice, execute the same TDD loop and capture brief evidence in this file.

Red step (must fail first):

    cargo test <new_test_name>

Green step (minimum implementation):

    cargo test <new_test_name>

Refactor step (all tests still pass):

    cargo test

Run these scenario checks after Milestone 4 and again after Milestone 5:

    cargo run -- index --repo .
    cargo run -- status --repo .
    cargo run -- find main --repo .
    cargo run -- refs main --repo .
    cargo run -- find main --repo . --json

Expected behavior is file:line:column output for terminal mode and stable schema fields in JSON mode (`schema_version`, `command`, `query`, `results`, `why_matched`, `confidence`).

## Validation and Acceptance

Acceptance requires observed behavior, not only passing compilation. A novice must be able to run `index` on this repository, run `find` and `refs` for known symbols, and receive deterministic results with confidence and provenance labels. Re-running `index` without file changes must report skip behavior and must not duplicate rows. Running `--json` for queries must produce valid JSON matching the locked golden format. Running `cargo test` must pass all unit, integration, and golden tests.

TDD compliance is also part of acceptance. Each implemented feature must have a recorded red-green-refactor history in commit order and in this plan’s artifacts section, showing that a failing test preceded production code.

## Idempotence and Recovery

`index` must be idempotent. Repeated runs against unchanged files should preserve row counts and ordering. Schema creation and migrations must be safe to run repeatedly. If indexing stops midway, rerunning `index` must repair state through per-file upserts. If the local index database is corrupted, deleting the index file and rerunning `index` must recover a working state.

## Artifacts and Notes

As work proceeds, include short transcripts for one red test failure, the corresponding green pass, and the final refactor full-suite pass for each milestone. Also include one sample `find --json` output and one incremental-index skip transcript. Keep examples concise and tied to acceptance behavior.

Baseline harness verification transcript:

    $ cargo test harness_can_run_binary_and_create_fixture_files
    running 1 test
    test harness_can_run_binary_and_create_fixture_files ... ok
    test result: ok. 1 passed; 0 failed

Milestone 1 red transcript:

    $ cargo test milestone1_ -- --nocapture
    running 3 tests
    Unexpected stdout ... var: Hello, world!
    test result: FAILED. 0 passed; 3 failed

Milestone 1 green transcript:

    $ cargo test milestone1_ -- --nocapture
    running 3 tests
    test milestone1_index_creates_db_and_prints_schema_version ... ok
    test milestone1_status_reports_schema_after_index_bootstrap ... ok
    test milestone1_find_and_refs_accept_symbol_queries ... ok
    test result: ok. 3 passed; 0 failed

Milestone 1 refactor transcript:

    $ cargo test
    running 1 test
    test harness_can_run_binary_and_create_fixture_files ... ok
    running 3 tests
    test milestone1_index_creates_db_and_prints_schema_version ... ok
    test milestone1_status_reports_schema_after_index_bootstrap ... ok
    test milestone1_find_and_refs_accept_symbol_queries ... ok
    test result: ok. 4 passed; 0 failed

## Interfaces and Dependencies

Use `clap` for command parsing, `rusqlite` for storage, `ignore` for repository walking with ignore rules, `tree-sitter` plus `tree-sitter-rust` for Rust AST extraction, `serde` plus `serde_json` for JSON output, and `anyhow` plus `thiserror` for error handling.

The CLI command surface in `/Users/robertguss/Projects/experiments/codex-5-3/src/cli.rs` must include `Index`, `Status`, `Find`, and `Refs`. The query result structure in `/Users/robertguss/Projects/experiments/codex-5-3/src/query/mod.rs` must include file path, line, column, symbol text, symbol kind, `why_matched`, `confidence`, and score fields. The JSON response in `/Users/robertguss/Projects/experiments/codex-5-3/src/output.rs` must include `schema_version`, command metadata, query metadata, and results.

Keep confidence vocabulary fixed in v0 as `ast_exact`, `ast_likely`, and `text_fallback`. Keep provenance vocabulary fixed in v0 as `ast_definition`, `ast_reference`, `exact_symbol_name`, `text_identifier_match`, and `text_substring_match`. If vocabulary changes later, increment schema version and document migration behavior in this plan.

## Revision Note

2026-02-06: Initial creation of this ExecPlan based on collaborative ideation. Added a Rust-first hybrid roadmap, concrete milestones, interfaces, validation criteria, and living-document sections required by `/Users/robertguss/Projects/experiments/codex-5-3/agents/PLANS.md`.

2026-02-06: Tightened the plan to an explicitly scoped v0 and added strict red-green-refactor requirements for every milestone at the user’s request.

2026-02-06: Completed dependency and test harness setup, added smoke integration test, and updated test helper to avoid deprecated assert_cmd API.

2026-02-06: Completed Milestone 1 with strict red-green-refactor cycle; added command surface tests, implemented SQLite schema bootstrap, refactored CLI/storage/output into modules, and updated smoke test for clap behavior.
