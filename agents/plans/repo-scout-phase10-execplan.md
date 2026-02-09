# Build `repo-scout` Phase 10 Rust Production-Readiness Hardening + Go `find` MVP

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase9-execplan.md` as historical context, but it is
self-contained and executable on its own.

## Purpose / Big Picture

Phase 10 moves `repo-scout` toward production readiness without widening blast radius. A user will
gain two concrete outcomes: stronger Rust reliability/performance guardrails in existing commands,
and first-class Go symbol-definition discovery via `find` (AST-backed, deterministic, schema-stable)
so Go repositories become practically usable right away. This phase intentionally keeps Go scope
minimal (`find` definitions only) and defers Go `refs` plus deeper polyglot hardening to later
phases for low-risk delivery.

## Progress

- [x] (2026-02-09 23:40Z) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-phase9-execplan.md` to anchor constraints and current boundary.
- [x] (2026-02-09 23:40Z) Ran dogfood baseline: `cargo run -- index --repo .`,
      `cargo run -- find test_command_for_target --repo . --json`, and
      `cargo run -- refs test_command_for_target --repo . --json`.
- [x] (2026-02-09 23:40Z) Ran quality baseline: `cargo test`, `cargo clippy --all-targets
      --all-features -- -D warnings`, and both contract validators.
- [x] (2026-02-09 23:40Z) Confirmed current repo-state risks to address in this phase:
      Rust-only test-command synthesis, narrow test-like classifier, and no Go adapter.
- [x] (2026-02-09 23:40Z) Authored this Phase 10 ExecPlan as planning-only work.
- [x] (2026-02-09 21:47Z) Declared implementation risk tier `1`, created branch
      `codex/phase10-kickoff`, and reran milestone dogfood prechecks
      (`index`/`find`/`refs test_command_for_target`).
- [x] (2026-02-09 21:47Z) Completed strict TDD slice 49A/50A for duplicate AST refs:
      added `milestone49_refs_deduplicates_ast_rows` (Red observed), implemented minimal dedupe in
      `src/query/mod.rs`, reran Green, ran `cargo test`, and reran post-slice dogfood.
- [x] (2026-02-09 21:56Z) Completed strict TDD slices 49B/50B and 49C/50C:
      added failing tests for non-Rust `verify-plan` target suppression and cross-language
      `--exclude-tests` filtering, then hardened `src/query/mod.rs` classifiers and runnable-target
      synthesis with all slice tests Green.
- [x] (2026-02-09 21:56Z) Completed strict TDD Milestone 51/52 Go `find` MVP:
      added failing Go integration tests, added `tree-sitter-go` dependency, implemented
      `src/indexer/languages/go.rs`, wired adapter dispatch, and confirmed deterministic
      AST-backed Go `find` behavior with `symbols_v2.language = "go"` persistence.
- [x] (2026-02-09 21:56Z) Completed Milestone 53 docs/evidence/verification closure:
      refreshed `README.md`, CLI/JSON/architecture docs, dogfood/perf artifacts, reran dogfood
      and full quality/contract validators.
- [x] (2026-02-09 21:57Z) Recorded final verification outputs:
      `cargo run` milestone checks, `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test`, `cargo fmt -- --check`, `validate_tdd_cycle --allow-empty-range`, and
      `validate_evidence_packet` all passed.
- [x] Milestone 49 strict TDD Rust hardening contract tests and baseline fixtures.
- [x] Milestone 50 strict TDD Rust recommendation/scope hardening implementation.
- [x] Milestone 51 strict TDD Go language adapter skeleton with definition extraction only.
- [x] Milestone 52 strict TDD `find` integration for Go and deterministic ranking guarantees.
- [x] Milestone 53 strict TDD docs/performance baseline refresh and contract closure.

## Surprises & Discoveries

- Observation: baseline targeted test command generation was Rust-only and path-shape dependent.
  Evidence: `src/query/mod.rs::test_command_for_target` emits only `cargo test --test <stem>` for
  direct `tests/<file>` paths.

- Observation: baseline test-like filtering was narrow and missed common Python/TS patterns.
  Evidence: `src/query/mod.rs::is_test_like_path` currently checks `tests/`, `/tests/`, and
  `*_test.rs` only.

- Observation: `refs test_command_for_target --json` currently includes a duplicated AST reference
  row for one location in `src/query/mod.rs`.
  Evidence: local dogfood output shows duplicate row at `src/query/mod.rs:2110:5`.

- Observation: chained call expressions (for example `helper().is_some()`) can produce duplicated
  AST reference rows for the same symbol location.
  Evidence: `tests/milestone49_rust_hardening.rs::milestone49_refs_deduplicates_ast_rows` Red
  failure before query-layer dedupe.

- Observation: baseline language adapter set covered Rust/TypeScript/Python only.
  Evidence: `src/indexer/mod.rs::extract_with_adapter` dispatches only `RustLanguageAdapter`,
  `TypeScriptLanguageAdapter`, and `PythonLanguageAdapter`.

- Observation: after Milestone 51/52, Go definitions are AST-backed for `find` while Go `refs`
  remains text-fallback in this phase by design.
  Evidence: `tests/milestone50_go_find.rs` passes for `find` AST-definition expectations and
  language persistence, while plan scope explicitly defers Go `refs`.

- Observation: local `validate_tdd_cycle.sh --base origin/main` fails on empty commit ranges.
  Evidence: current output says `No commits in range origin/main..HEAD. Empty ranges require
  --allow-empty-range.`

## Decision Log

- Decision: phase scope is Rust hardening + Go `find` only, with Go `refs` explicitly deferred.
  Rationale: this delivers immediate Go utility while preserving low-risk, sequential depth.
  Date/Author: 2026-02-09 / Codex

- Decision: preserve all JSON schema versions (`1`, `2`, `3`) unchanged in Phase 10.
  Rationale: contract stability is a production-readiness requirement and avoids automation breakage.
  Date/Author: 2026-02-09 / Codex

- Decision: keep Rust as the active coding-contract language and do not change
  `contracts/ACTIVE_LANGUAGE_CONTRACTS.md` in this phase.
  Rationale: repository policy intentionally keeps coding contract scope Rust-only even while
  indexing/querying other languages.
  Date/Author: 2026-02-09 / Codex

- Decision: treat duplicated `refs` rows as a hardening defect and fix it under Rust quality scope.
  Rationale: deterministic, de-duplicated output is a core reliability expectation.
  Date/Author: 2026-02-09 / Codex

- Decision: apply duplicate suppression in query result assembly (`ast_reference_matches`) instead
  of changing Rust AST extraction in this slice.
  Rationale: minimal blast radius fix that satisfies deterministic output contract without changing
  index schema or extraction semantics.
  Date/Author: 2026-02-09 / Codex

- Decision: add `tree-sitter-go` and a Go definition-only adapter in Phase 10.
  Rationale: unlock immediate Go `find` utility with minimal risk; defer Go `refs`/graph expansion
  to later phases.
  Date/Author: 2026-02-09 / Codex

## Outcomes & Retrospective

Outcome: Phase 10 delivered low-risk hardening and Go `find` MVP without schema churn.

Completion outcome: Rust recommendation/scope behavior is more robust and deterministic; duplicate
AST `refs` rows are suppressed; non-Rust files no longer produce invalid `cargo test --test`
targets; and cross-language test-like filtering now includes common TS/Python patterns. Go
projects now receive AST-backed definition lookup via `find`, with persisted `language = "go"`.
All quality/contract gates are green and docs/evidence were refreshed.

Residual work after completion: Go `refs`/graph depth, Python production-ready runner hardening,
TypeScript production-ready runner hardening, and final cross-language High-Bar/GA gates.

## Context and Orientation

`repo-scout` is a local CLI indexed into `<repo>/.repo-scout/index.db`. Command dispatch lives in
`src/main.rs` and CLI argument definitions in `src/cli.rs`. Indexing is orchestrated in
`src/indexer/mod.rs` through language adapters in `src/indexer/languages/`. Query behavior for
`find`, `refs`, `impact`, `context`, `tests-for`, `verify-plan`, `diff-impact`, and `explain` is
in `src/query/mod.rs`. Terminal/JSON rendering is in `src/output.rs`.

Current behavior relevant to this phase:

- Rust/TypeScript/Python adapter extraction existed at phase start; Go definition extraction is now
  added in this phase.
- `find` and `refs` are AST-first with text fallback and deterministic tie-breaks.
- JSON contracts are stable and versioned (`1`, `2`, `3`).
- Query hardening process constraints are enforced by milestone policy tests
  (`tests/milestone42_src_structure.rs`, `tests/milestone43_api_boundary_shape.rs`,
  `tests/milestone44_contract_hardening.rs`, `tests/milestone45_test_suite_structure.rs`).

Terms used in this plan:

- "Go `find` MVP" means AST-backed Go definitions are indexed and returned by `find`; no Go
  `refs` AST path is required in this phase.
- "Feature slice" means one user-visible behavior unit with its own Red -> Green -> Refactor loop.
- "Production-readiness hardening" means reliability/performance/determinism improvements without
  schema or command-surface expansion unless explicitly called out.

## Contract Inputs

Phase 10 implementation must consume and reference:

- Core risk policy: `contracts/core/RISK_TIER_POLICY.md`
- Core evidence policy: `contracts/core/EVIDENCE_REQUIREMENTS.md`
- Core review policy: `contracts/core/REVIEW_CONTRACT.md`
- Active language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Active language manifest: `contracts/ACTIVE_LANGUAGE_CONTRACTS.md`
- Task framing template: `templates/TASK_PACKET_TEMPLATE.md`
- Test plan template: `templates/TEST_PLAN_TEMPLATE.md`
- Evidence template: `templates/EVIDENCE_PACKET_TEMPLATE.md`
- PR checklist: `checklists/PR_CONTRACT_CHECKLIST.md`
- Adversarial checklist guidance (recommended for Tier 1): `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`
- TDD validator: `scripts/validate_tdd_cycle.sh`
- Evidence validator: `scripts/validate_evidence_packet.sh`
- CI contract gates: `.github/workflows/contract-gates.yml`

Required validator commands before PR merge:

    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## AGENTS.md Constraints

Consulted file:

- `AGENTS.md`

Effective constraints enforced by this plan:

- Strict Red -> Green -> Refactor ordering per feature slice; no production edits before Red.
- Risk tier must be declared before implementation.
- Dogfooding with `repo-scout` before and after each milestone:
  `index`, `find <target_symbol>`, and `refs <target_symbol>`.
- Integration-style tests in `tests/` with milestone-focused naming.
- Production code in `src/` must not introduce `unwrap()`/`expect()`/`panic!` without explicit
  contract exception.
- Required validators run before PR update.

If `AGENTS.md` and contracts conflict, the stricter requirement wins.

## Risk Tier and Required Controls

Phase 10 risk tier: `1` (moderate).

Rationale: this phase changes query/indexing behavior and introduces a new language adapter but
avoids security-sensitive code, auth paths, irreversible operations, and planned schema migrations.

Tier 1 controls required and mapped:

- Red -> Green -> Refactor evidence for every feature slice.
- Task packet required (`templates/TASK_PACKET_TEMPLATE.md` mapping).
- Test plan required (`templates/TEST_PLAN_TEMPLATE.md` mapping).
- Evidence packet required (PR-body-first evidence and validator).
- Rollback plan required (`Idempotence and Recovery` section below).
- Reviewer count: minimum 1.
- Adversarial review checklist: recommended and included in review gate.

Escalation rule for this plan: if schema changes become necessary, pause and escalate to Tier 2
until controls and checklist coverage are updated.

## Strict TDD Contract

No production code change is allowed for a feature slice until the slice-specific failing test is
observed.

Feature slices for Phase 10 are:

- Rust hardening slices (`refs` dedupe, test-target command precision, and test-like scope
  consistency behavior chosen for this phase).
- Go `find` definition extraction slices (parser load, symbol extraction, and CLI-visible result
  behavior).

For each slice record and keep in this ExecPlan:

- Red evidence: failing test command and concise failure reason.
- Green evidence: same test passing after minimal implementation.
- Refactor evidence: full-suite `cargo test` pass after cleanup.

## Plan of Work

### Milestone 49: Rust Hardening Contract Baseline

Milestone goal: lock failing behavior contracts before production edits.

Feature slice 49A adds a failing test for duplicate AST reference row suppression in
`tests/milestone49_rust_hardening.rs` using a minimal fixture where one logical reference should
appear exactly once in `refs --json`.

Feature slice 49B adds failing tests for deterministic test-target command behavior around
Rust-oriented targets and non-runnable support paths to ensure no regression while hardening.

Feature slice 49C adds failing tests for scope filtering consistency over test-like paths touched in
this phase.

Evidence log (updated as slices complete):

- Slice 49A/50A (`milestone49_refs_deduplicates_ast_rows`)
  Red: `cargo test milestone49_refs_deduplicates_ast_rows -- --nocapture` failed with
  "duplicate refs row returned for location and match kind".
  Green: same command passed after dedupe change in `src/query/mod.rs::ast_reference_matches`.
  Refactor: `cargo test` passed; post-slice dogfood `refs test_command_for_target --repo .`
  now returns 3 rows (duplicate removed).

- Slice 49B/50B (`milestone49_verify_plan_targets_remain_deterministic`)
  Red: `cargo test milestone49_verify_plan_targets_remain_deterministic -- --nocapture` failed
  because `tests/python_target.py` incorrectly generated `cargo test --test python_target`.
  Green: same command passed after restricting runnable target synthesis to direct
  `tests/<file>.rs` paths.
  Refactor: `cargo test` passed with existing `verify-plan` contracts preserved.

- Slice 49C/50C (`milestone49_scope_filtering_preserves_contract`)
  Red: `cargo test milestone49_scope_filtering_preserves_contract -- --nocapture` failed because
  `--exclude-tests` did not drop `.test.ts` and Python test-naming patterns.
  Green: same command passed after broadening test-like classifiers in `src/query/mod.rs`.
  Refactor: `cargo test` passed and scope behavior remained deterministic.

### Milestone 50: Rust Hardening Implementation

Milestone goal: ship minimal Rust hardening fixes to satisfy Milestone 49 contracts.

Feature slice 50A updates query result assembly in `src/query/mod.rs` to eliminate duplicate AST
reference rows while preserving deterministic ordering.

Feature slice 50B hardens runnable-target command synthesis and supporting classifiers in
`src/query/mod.rs` with no command-surface changes.

Feature slice 50C preserves existing schema envelopes and command defaults while improving
deterministic behavior and reliability.

### Milestone 51: Go Adapter Skeleton and Definition Extraction

Milestone goal: add Go indexing extraction limited to symbols needed for `find`.

Feature slice 51A adds Go parser dependency and adapter module:
`src/indexer/languages/go.rs`, wired from `src/indexer/languages/mod.rs` and adapter dispatch in
`src/indexer/mod.rs`.

Feature slice 51B extracts Go definitions into `ExtractedSymbol` only (no Go reference/edge
requirements in this phase). Target kinds include practical top-level and receiver-bound symbols
needed for definition lookup.

Feature slice 51C adds integration fixtures and tests in `tests/milestone50_go_find.rs` validating
persisted `symbols_v2.language = "go"` and AST definition path behavior for `find`.

Evidence log (updated as slices complete):

- Slice 51A/51B (`milestone50_go_find_definitions_are_ast_backed`,
  `milestone50_go_find_persists_language_metadata`)
  Red: both tests failed before Go adapter wiring (`why_matched=exact_symbol_name` fallback and no
  Go `symbols_v2` row).
  Green: both tests passed after adding `tree-sitter-go`, implementing
  `src/indexer/languages/go.rs`, and wiring adapter dispatch.
  Refactor: `cargo test milestone50_go_find -- --nocapture` passed.

### Milestone 52: Go `find` End-to-End Integration and Determinism

Milestone goal: ensure Go definition results are CLI-visible, deterministic, and schema-stable.

Feature slice 52A verifies `find <go_symbol> --json` returns `ast_definition` entries with
`schema_version = 1`.

Feature slice 52B verifies fallback controls (`--code-only`, `--exclude-tests`, `--max-results`)
compose without regressing existing language behavior.

Feature slice 52C verifies deterministic repeatability over fixture repos across repeated runs.

Evidence log (updated as slices complete):

- Slice 52A/52C (`milestone50_go_find_json_is_deterministic`)
  Red: test failed before Go adapter because first result used text fallback.
  Green: test passed with AST-backed Go definition rows and repeatable JSON output.
  Refactor: `cargo test milestone50_go_find -- --nocapture` passed.

- Slice 52B (`milestone50_go_find_scope_flags_do_not_regress_existing_languages`)
  Red baseline preserved existing behavior (test already Green pre-implementation).
  Green: test remained Green after Go adapter integration.
  Refactor: `cargo test milestone50_go_find -- --nocapture` and full `cargo test` both passed.

### Milestone 53: Docs, Dogfood, and Contract Closure

Milestone goal: align docs/evidence with shipped Phase 10 behavior and close all gates.

Feature slice 53A updates user docs:
`README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`.

Feature slice 53B updates proof artifacts and baselines:
`docs/dogfood-log.md`, `docs/performance-baseline.md`, and this ExecPlan progress transcripts.

Feature slice 53C runs all required validators and records acceptance evidence.

Evidence log (updated as slices complete):

- Slice 53A/53B
  Docs updated: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
  `docs/architecture.md`, `docs/dogfood-log.md`, `docs/performance-baseline.md`.

- Slice 53C verification
  Executed: milestone dogfood commands (`index`, `find`, `refs`), `cargo test`,
  `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt -- --check`,
  `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`, and
  `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/repo-scout`.

Before each milestone:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json

Per-slice strict TDD loop:

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure before production edits
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimum implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 49 expected slice commands:

    cargo test milestone49_refs_deduplicates_ast_rows -- --nocapture
    cargo test milestone49_verify_plan_targets_remain_deterministic -- --nocapture
    cargo test milestone49_scope_filtering_preserves_contract -- --nocapture

Milestone 51/52 expected Go slice commands:

    cargo test milestone50_go_find_definitions_are_ast_backed -- --nocapture
    cargo test milestone50_go_find_persists_language_metadata -- --nocapture
    cargo test milestone50_go_find_json_is_deterministic -- --nocapture
    cargo test milestone50_go_find_scope_flags_do_not_regress_existing_languages -- --nocapture

Milestone 53 verification commands:

    cargo run -- index --repo .
    cargo run -- find main --repo . --json
    cargo run -- refs main --repo . --json
    cargo run -- find GoLanguageAdapter --repo .
    cargo run -- refs GoLanguageAdapter --repo .
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo fmt -- --check
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

If validating on a no-commit local working tree, use this non-PR-only helper for local checks:

    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range

## Validation and Acceptance

Acceptance is behavior-first:

- `find` and `refs` remain deterministic with no duplicate AST rows for the same location.
- Go fixtures produce AST-backed `find` definition results with `schema_version = 1`.
- Existing language behavior (Rust/TypeScript/Python) remains unchanged for non-Go paths.
- All quality and contract gates are green.

Strict TDD acceptance evidence is mandatory: each feature slice must include one recorded Red
failure, one Green pass, and one Refactor full-suite pass in this document.

## Idempotence and Recovery

This phase is additive and idempotent. Re-running index/query/test commands should not mutate
tracked files except planned docs/tests/source edits.

No schema migration is planned. If schema change pressure emerges:

- pause implementation,
- record decision and rationale in `Decision Log`,
- escalate risk tier controls before continuing.

Rollback plan:

- Keep Go extraction isolated to new adapter/module wiring for easy revert.
- Keep Rust hardening changes isolated to specific helper paths in `src/query/mod.rs`.
- If regression occurs, revert the latest milestone while preserving prior passing milestones and
  rerun full gates.

## Review and CI Gates

Before merge:

- Complete `checklists/PR_CONTRACT_CHECKLIST.md`.
- Complete `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` (recommended for Tier 1, required if risk
  escalates).
- Ensure `.github/workflows/contract-gates.yml` expected outcomes remain green for core and Rust
  gates.
- Ensure PR body sections in `.github/pull_request_template.md` contain Red/Green/Refactor and risk
  evidence.

## Interfaces and Dependencies

Expected touch points:

- `src/indexer/languages/mod.rs`
- `src/indexer/languages/go.rs` (new)
- `src/indexer/mod.rs`
- `src/query/mod.rs`
- `tests/milestone49_rust_hardening.rs` (new)
- `tests/milestone50_go_find.rs` (new)
- `tests/fixtures/phase10/go_find/...` (new)
- `README.md`
- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`
- `docs/dogfood-log.md`
- `docs/performance-baseline.md`

Dependencies:

- Add `tree-sitter-go` only if required for adapter implementation. Any dependency addition must be
  documented in Decision Log and PR evidence.

## Revision Note

2026-02-09: Created initial Phase 10 execution plan focused on Rust production-readiness hardening
plus Go `find` MVP, aligned to strict TDD, Contract System v2, and AGENTS dogfooding constraints.
2026-02-09: Completed Phase 10 implementation and closure evidence (Rust hardening slices, Go
definition adapter, docs/perf/dogfood refresh, and full validator pass).
