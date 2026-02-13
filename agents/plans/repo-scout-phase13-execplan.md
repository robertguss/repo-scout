# Build `repo-scout` Phase 13 Python Production-Ready Closure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase12-execplan.md` and the roadmap in
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is intentionally scoped to Python
production closure under the low-risk sequencing policy.

## Purpose / Big Picture

Phase 13 closes the current Python gap where recommendation workflows are Rust-centric (`cargo
test`) and Python import edge resolution misses common relative-import call paths. After this
phase, Python repositories that explicitly declare `pytest` will get runnable Python-targeted
recommendations in `tests-for` and `verify-plan`, plus stronger `diff-impact` attribution for
relative-import call flows.

User-visible outcome: in explicit `pytest` repos, `verify-plan` recommends `pytest` commands
instead of only `cargo test`, and `diff-impact` can attribute callers that use relative imports
like `from .util import helper`.

## Progress

- [x] (2026-02-10 00:07Z) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md` to confirm next-phase scope.
- [x] (2026-02-10 00:10Z) Confirmed next unimplemented phase is Phase 13 and created branch
      `codex/phase13-python-production-closure`.
- [x] (2026-02-10 00:13Z) Declared risk tier `1` before implementation and ran required pre-slice
      dogfooding commands for symbol `test_command_for_target`.
- [x] (2026-02-10 00:18Z) Added failing Phase 13 integration tests in
      `tests/milestone60_python_recommendations.rs` for `pytest` recommendation behavior, Python
      `_tests.py` classification, and relative-import `diff-impact` attribution.
- [x] (2026-02-10 00:18Z) Observed strict Red via
      `cargo test --test milestone60_python_recommendations -- --nocapture` (4 failures).
- [x] (2026-02-10 00:25Z) Implemented minimal production changes for Green in `src/query/mod.rs`,
      `src/indexer/languages/python.rs`, and `src/indexer/mod.rs`; milestone60 suite now passes.
- [x] (2026-02-10 00:27Z) Resolved one regression found during non-regression/full-suite checks
      (`milestone12_diff_impact`) by narrowing symbol-resolution fallback guard to
      `qualified_symbol`-keyed edges only.
- [x] (2026-02-10 00:29Z) Completed non-regression/full-suite/validator gates:
      `cargo test --test milestone34_python_semantics -- --nocapture`,
      `cargo test --test milestone49_rust_hardening -- --nocapture`,
      `cargo test --test milestone12_diff_impact -- --nocapture`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`, and
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 00:29Z) Ran post-implementation dogfooding (`index/find/refs`) and refreshed
      docs/evidence (`README.md`, `docs/cli-reference.md`, `docs/architecture.md`,
      `docs/json-output.md`, `legacy performance baseline doc (removed)`, `docs/dogfood-log.md`) plus a
      repeatable fixture corpus under `tests/fixtures/phase13/python_recommendations/`.

## Surprises & Discoveries

- Observation: even in a repository with `pytest.ini`, `tests-for` currently returns no runnable
  Python targets and `verify-plan` returns only `cargo test`.
  Evidence: local probe on 2026-02-10 with a Python fixture repo and `pytest.ini` produced
  `tests-for ... --json` results `[]` and a `verify-plan` payload containing only full-suite
  `cargo test`.

- Observation: `diff-impact` currently misses caller attribution for relative imports
  (`from .util import helper`) when the imported file changes.
  Evidence: local probe on 2026-02-10 returned only distance-0 changed symbol rows for
  `src/pkg/util.py` and no `called_by` row for caller `run` in `src/pkg/consumer.py`.

- Observation: strict symbol fallback suppression for all `file_path`-scoped keys caused regression
  in existing graph-neighbor contracts (`milestone12_diff_impact`).
  Evidence: `cargo test --test milestone12_diff_impact -- --nocapture` initially failed two tests
  after introducing broad fallback suppression in `resolve_symbol_id_in_tx`.

## Decision Log

- Decision: classify this phase as risk tier `1`.
  Rationale: behavior changes affect recommendation synthesis and Python import resolution but do
  not require schema, migration, or persistence format changes.
  Date/Author: 2026-02-10 / Codex

- Decision: gate Python runnable recommendations behind strict explicit `pytest` detection.
  Rationale: roadmap requires strict runner-aware behavior; explicit config signals avoid guessing
  and preserve deterministic low-noise recommendations.
  Date/Author: 2026-02-10 / Codex

- Decision: suppress symbol-only fallback only for `qualified_symbol` keys (not all scoped
  `file_path` keys) in `resolve_symbol_id_in_tx`.
  Rationale: this preserves deferred resolution for precise cross-file qualified edges (needed for
  relative-import attribution) without regressing existing scoped fallback behavior in legacy
  diff-impact fixtures.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Phase 13 delivered Python production-closure behavior for this roadmap stage with strict TDD
evidence and no schema change.

Shipped outcomes:

- `tests-for` now treats Python targets as runnable only in explicit pytest contexts and supports
  `*_tests.py` alongside existing Python test-name patterns.
- `verify-plan` now emits targeted `pytest <target>` steps and can choose `pytest` full-suite gates
  for explicit Python-only changed scopes, while preserving existing Rust behavior.
- Python relative-import identifier calls (`from .module import symbol`) now preserve caller
  attribution in `diff-impact`.
- Docs and evidence artifacts now describe runner-aware recommendation behavior, and a repeatable
  Phase 13 fixture corpus exists for dogfood/perf checks.

Validation outcome: milestone60 red-to-green cycle complete, non-regression suites passed
(`milestone34`, `milestone49`, `milestone12`), full `cargo test`/`clippy`/`fmt`/contract validators
passed, and post-change dogfooding on repository root commands was successful.

## Context and Orientation

`repo-scout` query logic is centralized in `src/query/mod.rs`. Before this phase, `tests-for` and
`verify-plan` synthesized runnable commands through Rust-only `cargo test` command mapping.
Python AST extraction and call-edge synthesis is in `src/indexer/languages/python.rs`.

Phase 13 work touches:

- recommendation command synthesis in `src/query/mod.rs`,
- Python import path resolution in `src/indexer/languages/python.rs`,
- new integration tests in `tests/` for behavior contracts,
- docs refresh in `docs/` for command behavior updates.

## Contract Inputs

Phase 13 implementation consumes and aligns with:

- `contracts/core/RISK_TIER_POLICY.md`
- `contracts/core/EVIDENCE_REQUIREMENTS.md`
- `contracts/core/REVIEW_CONTRACT.md`
- `contracts/languages/RUST_CODING_CONTRACT.md`
- `contracts/ACTIVE_LANGUAGE_CONTRACTS.md`
- `templates/TASK_PACKET_TEMPLATE.md`
- `templates/TEST_PLAN_TEMPLATE.md`
- `templates/EVIDENCE_PACKET_TEMPLATE.md`
- `checklists/PR_CONTRACT_CHECKLIST.md`
- `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` (recommended for Tier 1)
- `scripts/validate_tdd_cycle.sh`
- `scripts/validate_evidence_packet.sh`
- `.github/workflows/contract-gates.yml`

Required validator commands before PR update:

    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

Local no-commit range variant used during implementation:

    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range

## AGENTS.md Constraints

Consulted file:

- `AGENTS.md`

Applied constraints in this plan:

- strict Red -> Green -> Refactor per feature slice,
- risk tier declared before production edits,
- dogfooding commands run before and after each feature slice,
- integration tests under `tests/` with milestone naming,
- no `unwrap()` / `expect()` / `panic!` introduced in `src/` production code,
- contract validators run before closure reporting.

Dogfood commands used before implementation:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json

Dogfood commands to run after implementation:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo .
    cargo run -- refs test_command_for_target --repo .
    cargo test

## Risk Tier and Required Controls

Phase 13 risk tier: `1` (moderate).

Rationale: language behavior and recommendation outputs change, but schema versions and storage
layout remain unchanged.

Tier 1 controls applied:

- strict per-slice Red/Green/Refactor evidence,
- deterministic output expectations in integration tests,
- no schema changes without escalation,
- rollback-safe additive changes,
- full test/lint/contract validation before closure.

Escalation rule: if schema, migration, or persistence invariants need to change, pause and escalate
to Tier 2 controls before proceeding.

## Strict TDD Contract

No production code will be edited until failing tests exist for the exact Phase 13 slice.

Feature slices for this phase:

- `pytest` strict detection + runnable Python command synthesis for `tests-for`.
- `pytest` strict detection + runnable Python command synthesis for `verify-plan` (targeted and
  full-suite gate selection).
- broadened Python test-file classification for runnable recommendation targets.
- Python relative-import impact resolution (`from .module import symbol`) for caller attribution.

## TDD Evidence Log

- Red:
  - `cargo test --test milestone60_python_recommendations -- --nocapture` failed with 4 failures:
    pytest runnable-target detection, pytest verify-plan command synthesis, `*_tests.py`
    classification, and relative-import diff-impact caller attribution.
- Green:
  - `cargo test --test milestone60_python_recommendations -- --nocapture` passed after query and
    indexer fixes.
- Refactor/non-regression:
  - `cargo test --test milestone34_python_semantics -- --nocapture` passed.
  - `cargo test --test milestone49_rust_hardening -- --nocapture` passed.
  - `cargo test --test milestone12_diff_impact -- --nocapture` passed after narrowing fallback
    suppression.
  - `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`
    passed.

## Plan of Work

### Milestone 60: Python Runner-Aware Recommendation Closure

Milestone goal: in repos with explicit `pytest` configuration, make `tests-for` and `verify-plan`
emit runnable Python-targeted commands deterministically, while preserving existing Rust behavior.

Feature slices:

- 60A: failing then passing test proving `tests-for` returns runnable Python targets when
  `pytest` is explicitly detected.
- 60B: failing then passing test proving `verify-plan` emits targeted `pytest <path>` and full
  suite `pytest` in explicit Python runner contexts.
- 60C: failing then passing test proving additional Python test-file naming conventions are treated
  as runnable targets where appropriate.

### Milestone 61: Python Semantics and Closure Refresh

Milestone goal: close a practical Python semantic edge-case and refresh docs/evidence.

Feature slices:

- 61A: failing then passing test proving `diff-impact` caller attribution works for
  relative-import function calls.
- 61B: docs and dogfood refresh confirming Phase 13 behavior in CLI reference/architecture logs.

## Concrete Steps

Run from repository root:

    cargo test --test milestone60_python_recommendations -- --nocapture
    cargo test --test milestone34_python_semantics -- --nocapture
    cargo test --test milestone49_rust_hardening -- --nocapture
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## Validation and Acceptance

Acceptance criteria for this phase:

- In explicit `pytest` repos, `tests-for` default output includes runnable Python test targets
  (not only support paths).
- In explicit `pytest` repos, `verify-plan` includes targeted `pytest <file>` steps and uses a
  Python full-suite gate when Python is the active changed scope.
- Existing Rust recommendation behavior remains unchanged in prior milestone tests.
- `diff-impact` includes expected `called_by` rows for relative-import Python call paths.
- Docs and dogfood artifacts describe the shipped behavior without schema/version drift.

## Idempotence and Recovery

All Phase 13 changes are additive and deterministic. Re-indexing the same repository should produce
stable recommendation and impact outputs.

If regressions appear:

- revert only Phase 13 files (`src/query/mod.rs`, `src/indexer/languages/python.rs`,
  `tests/milestone60_python_recommendations.rs`, docs edits),
- re-run Phase 13 red tests to isolate slice failure,
- re-apply minimal fixes slice-by-slice under strict TDD.

## Review and CI Gates

Before completion:

- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`
- `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

## Interfaces and Dependencies

No new external crates are planned. Phase 13 uses existing Rust stdlib and current dependency set.
Any dependency addition requires explicit rationale and non-regression evidence.

## Revision Note

2026-02-10: Created initial Phase 13 execution plan for Python production-ready closure with strict
runner-aware recommendation and semantics scope.

2026-02-10: Completed Phase 13 implementation with strict red-green-refactor evidence, docs refresh,
post-change dogfooding, and full gate validation.
