# Build `repo-scout` Phase 12 Go Production-Ready Closure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase11-execplan.md` and the roadmap in
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is intentionally scoped to Go
production closure after Rust production closure.

## Purpose / Big Picture

Phase 12 closes the remaining Go language gap between definition-only support and practical daily
query usage. After this phase, Go users can run `refs`, `impact`, and `diff-impact` and get
AST-backed call-site and caller-attribution behavior in common package-alias workflows, including
duplicate function names across packages.

User-visible outcome: Go selector calls (for example `utilpkg.Helper()` and
`utilpkg.Greeter{}.SayHello()`) now produce deterministic AST references and stable graph edges, so
`refs`, `impact`, and `diff-impact` report expected `called_by` rows instead of falling back to
text-only matching.

## Progress

- [x] (2026-02-10 16:24Z) Re-read `AGENTS.md`, `agents/PLANS.md`,
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md`, and
      `agents/plans/repo-scout-phase11-execplan.md` to anchor Phase 12 boundary and controls.
- [x] (2026-02-10 16:26Z) Declared risk tier `1` before implementation and ran required pre-slice
      dogfooding commands on repository root (`index/find/refs`).
- [x] (2026-02-10 16:29Z) Added failing Phase 12 Go fixture + integration tests in
      `tests/milestone59_go_refs.rs` and `tests/fixtures/phase12/go_refs/`.
- [x] (2026-02-10 16:30Z) Observed strict Red failure for all new slices via
      `cargo test --test milestone59_go_refs -- --nocapture`.
- [x] (2026-02-10 16:35Z) Implemented Go adapter hardening in `src/indexer/languages/go.rs`:
      AST references, import-alias-aware call edge candidates, deterministic dedupe/sorting, and
      Go `interface`/`type_alias` kind extraction.
- [x] (2026-02-10 16:35Z) Observed Green for milestone59 via
      `cargo test --test milestone59_go_refs -- --nocapture`.
- [x] (2026-02-10 16:37Z) Completed non-regression slices:
      `cargo test --test milestone50_go_find -- --nocapture` and
      `cargo test --test milestone49_rust_hardening -- --nocapture`.
- [x] (2026-02-10 16:42Z) Completed refactor/full-suite gates and validators:
      `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`, and
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 16:46Z) Ran post-implementation dogfood commands on repository root and
      `tests/fixtures/phase12/go_refs`, then updated docs and this ExecPlan.

## Surprises & Discoveries

- Observation: pre-change Go `refs` behavior was consistently text fallback and did not emit
  AST-backed rows from Go call sites.
  Evidence: `cargo test --test milestone59_go_refs -- --nocapture` red output showed
  `milestone59_go_refs_are_ast_backed_for_selector_calls` failure.

- Observation: duplicate `Helper` symbols across imported packages dropped expected
  `diff-impact` caller rows without import-alias-aware edge targeting.
  Evidence: red failure in
  `milestone59_go_diff_impact_prefers_import_alias_target_for_duplicate_functions`.

- Observation: Go `type_spec` extraction alone is insufficient for production-readiness because
  interfaces and aliases need distinct symbol kinds for explainability and contracts.
  Evidence: red failure in
  `milestone59_go_type_kinds_capture_interface_and_alias_definitions` (`type` vs `interface`).

## Decision Log

- Decision: keep Phase 12 schema-stable (`SCHEMA_VERSION = 3`) and resolve Go closure through
  adapter extraction and deterministic edge synthesis only.
  Rationale: roadmap scope and risk policy favor low-blast-radius behavior closure without
  persistence migration.
  Date/Author: 2026-02-10 / Codex

- Decision: implement import-alias disambiguation via deterministic candidate file paths instead of
  repository-wide module graph reconstruction.
  Rationale: candidate-path strategy gives practical caller attribution under duplicate names with
  low complexity and stable deterministic ordering.
  Date/Author: 2026-02-10 / Codex

- Decision: include fallback language-scoped call targets after import-aware candidates.
  Rationale: preserves attribution for non-import selector/method calls while avoiding hard
  dependency on perfect alias resolution in all Go syntax variants.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Phase 12 shipped with strict TDD evidence and no schema changes. Go now supports AST-backed
references for call identifiers/selector fields, deterministic call-edge construction for practical
`impact`/`diff-impact`, and richer type-kind extraction (`interface`, `type_alias`) needed for
query fidelity.

All new and existing tests passed (`cargo test`), quality gates passed (`clippy`, `fmt`), and
dogfood runs on `tests/fixtures/phase12/go_refs` showed expected `called_by` rows for `Run` when
`src/util/util.go` changes.

Residual roadmap work remains in Phases 13-16 (Python closure, TypeScript closure, cross-language
convergence, GA hardening).

## Context and Orientation

`repo-scout` is a local deterministic CLI. Indexing is orchestrated in `src/indexer/mod.rs` and
language extraction logic is in `src/indexer/languages/`. Query behavior for `find`, `refs`,
`impact`, and `diff-impact` is in `src/query/mod.rs`.

For this phase, the key implementation file is `src/indexer/languages/go.rs`, which converts
tree-sitter Go AST nodes into:

- symbol definitions (`symbols_v2` / `ast_definitions`),
- reference hits (`ast_references`), and
- graph edges (`symbol_edges_v2`).

The fixture corpus for this phase is under `tests/fixtures/phase12/go_refs/` and intentionally
contains duplicate `Helper` symbols across packages plus selector calls to validate disambiguation.

## Contract Inputs

Phase 12 implementation consumed and aligned with:

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

- Strict Red -> Green -> Refactor for each feature slice.
- Risk tier declared before production edits.
- Dogfooding commands run before and after implementation slices.
- Integration-style tests in `tests/` with milestone naming.
- No `unwrap()`/`expect()`/`panic!` introduced in `src/` production code.
- Contract validators run before closure reporting.

Dogfood commands used before implementation:

    cargo run -- index --repo .
    cargo run -- find GoLanguageAdapter --repo . --json
    cargo run -- refs GoLanguageAdapter --repo . --json

Dogfood commands used after implementation:

    cargo run -- index --repo .
    cargo run -- find GoLanguageAdapter --repo .
    cargo run -- refs GoLanguageAdapter --repo .
    cargo test

## Risk Tier and Required Controls

Phase 12 risk tier: `1` (moderate).

Rationale: the phase changes language extraction, reference indexing, and edge synthesis behavior,
but does not change schema versions, migrations, or destructive data operations.

Tier 1 controls applied:

- per-slice Red/Green/Refactor evidence,
- task/test/evidence template alignment,
- rollback plan and idempotence checks,
- at least one reviewer expectation,
- adversarial checklist recommended for review.

Escalation rule: if schema/persistence invariants require change, pause and escalate to Tier 2
controls before proceeding.

## Strict TDD Contract

No production code was edited until failing tests existed for the exact Phase 12 slices.

Feature slices implemented:

- Go AST-backed references for selector-call symbols.
- Go import-alias call-edge disambiguation under duplicate function names.
- Go interface/type-alias kind extraction.
- Deterministic Go `refs`/`impact` JSON stability.

## TDD Evidence Log

- Red:
  - `cargo test --test milestone59_go_refs -- --nocapture` failed with 4 failures covering all new
    Phase 12 slices.
- Green:
  - `cargo test --test milestone59_go_refs -- --nocapture` passed after Go adapter updates.
- Refactor/non-regression:
  - `cargo test --test milestone50_go_find -- --nocapture` passed.
  - `cargo test --test milestone49_rust_hardening -- --nocapture` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `cargo test` full suite passed.

## Plan of Work

### Milestone 59: Go Refs/Graph Production Closure

Milestone goal: move Go from definition-only support to practical production-ready `refs` and graph
behavior with deterministic outputs and duplicate-name disambiguation.

Feature slice 59A: add failing/then passing tests proving Go selector calls emit AST references in
`refs`.

Feature slice 59B: add failing/then passing tests proving `diff-impact` includes Go `called_by`
rows when duplicate target function names exist in different packages and calls are import-aliased.

Feature slice 59C: add failing/then passing tests proving Go interfaces and type aliases are
persisted with correct `kind` values.

Feature slice 59D: lock deterministic repeatability for Go `refs` and `impact` JSON payloads.

## Concrete Steps

Run from repository root:

    cargo test --test milestone59_go_refs -- --nocapture
    cargo test --test milestone50_go_find -- --nocapture
    cargo test --test milestone49_rust_hardening -- --nocapture
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

Go fixture dogfood:

    cargo run -- index --repo tests/fixtures/phase12/go_refs
    cargo run -- refs Helper --repo tests/fixtures/phase12/go_refs --json
    cargo run -- impact SayHello --repo tests/fixtures/phase12/go_refs --json
    cargo run -- diff-impact --changed-file src/util/util.go --repo tests/fixtures/phase12/go_refs --json

## Validation and Acceptance

Acceptance criteria for this phase:

- Go `refs` returns AST-backed rows for call sites in fixture workflows.
- Go `impact` and `diff-impact` include deterministic `called_by` rows for alias-qualified selector
  calls.
- Duplicate function names across Go packages do not silently drop expected caller attribution in
  fixture scenarios.
- Go interface and type alias definitions persist as `kind = interface` and `kind = type_alias`.
- Existing Go `find` and adjacent Rust hardening suites remain green.
- Full suite and quality gates pass with schema unchanged.

## Idempotence and Recovery

Phase 12 changes are additive and idempotent. Re-indexing identical files should keep stable symbol
and edge counts without unbounded growth.

Recovery approach if regressions appear:

- revert only Phase 12 Go adapter changes in `src/indexer/languages/go.rs`,
- keep failing milestone59 tests to preserve defect contract,
- reintroduce fixes slice-by-slice until full-suite pass is restored.

No migration or persistent destructive operations were introduced.

## Review and CI Gates

Before merge:

- complete `checklists/PR_CONTRACT_CHECKLIST.md`,
- complete `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` (recommended Tier 1 control),
- ensure `.github/workflows/contract-gates.yml` outcomes are green,
- ensure PR evidence sections in `.github/pull_request_template.md` include Red/Green/Refactor
  transcripts for milestone59.

## Interfaces and Dependencies

Touched implementation interfaces:

- `src/indexer/languages/go.rs`
- `tests/milestone59_go_refs.rs`
- `tests/fixtures/phase12/go_refs/src/app/main.go`
- `tests/fixtures/phase12/go_refs/src/util/util.go`
- `tests/fixtures/phase12/go_refs/src/other/other.go`

Documentation/process touch points:

- `README.md`
- `docs/cli-reference.md`
- `docs/architecture.md`
- `docs/json-output.md`
- `docs/performance-baseline.md`
- `docs/dogfood-log.md`
- `agents/plans/repo-scout-phase12-execplan.md`

No new third-party dependencies were added.

## Revision Note

2026-02-10: Created and completed Phase 12 execution plan for Go production-ready closure with
strict TDD evidence, adapter hardening, fixture expansion, dogfood transcripts, and docs/process
updates.
