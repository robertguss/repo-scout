# Build `repo-scout` Phase 14 TypeScript Production-Ready Closure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase13-execplan.md` and the roadmap in
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is intentionally scoped to
TypeScript production closure under the low-risk sequential policy.

## Purpose / Big Picture

Phase 14 closes the remaining TypeScript production-readiness gaps in two areas: strict Node
runner-aware recommendation synthesis (`jest`/`vitest`) and practical import-resolution edge cases
for caller attribution.

User-visible outcome: in explicit and unambiguous Node test-runner contexts, `tests-for` and
`verify-plan` emit runnable TypeScript test commands deterministically, while `diff-impact` keeps
caller attribution for directory-style imports (`./module` -> `./module/index.ts`).

## Progress

- [x] (2026-02-10 00:35Z) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md` to confirm the next unimplemented
      roadmap phase is Phase 14.
- [x] (2026-02-10 00:36Z) Declared risk tier `1` before implementation and ran required pre-slice
      dogfooding commands for symbol `test_command_for_target`.
- [x] (2026-02-10 00:40Z) Added failing Phase 14 integration tests in
      `tests/milestone61_typescript_production.rs` for strict Jest/Vitest recommendation behavior
      and TypeScript directory-import caller attribution.
- [x] (2026-02-10 00:41Z) Observed strict Red via
      `cargo test --test milestone61_typescript_production -- --nocapture` (4 failures).
- [x] (2026-02-10 00:47Z) Implemented minimal production changes for Green in
      `src/query/mod.rs` and `src/indexer/languages/typescript.rs`; milestone61 suite is green.
- [x] (2026-02-10 00:50Z) Added repeatable fixture corpus under
      `tests/fixtures/phase14/typescript_production/` and refactored milestone61 tests to use
      `include_str!` fixture sources.
- [x] (2026-02-10 00:57Z) Completed full closure gates and evidence refresh:
      `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`,
      post-change dogfood on repository root + Phase 14 fixtures, and docs refresh.

## Surprises & Discoveries

- Observation: TypeScript test targets were already discoverable by path pattern but remained
  non-runnable because command synthesis had no Node runner context.
  Evidence: milestone61 Red showed `tests-for` and `verify-plan` not emitting any `npx` test
  commands despite explicit `package.json` runner signals.

- Observation: TypeScript member-call attribution for `import * as util from "./util"` failed when
  callee lived in `src/util/index.ts` because import hints only resolved `<module>.ts`.
  Evidence: milestone61 Red failure in
  `milestone61_diff_impact_resolves_typescript_directory_index_import_calls`.

- Observation: adding multi-candidate import paths (`<module>.ts` + `<module>/index.ts` and TSX
  variants) keeps old behavior while covering directory imports without schema changes.
  Evidence: milestone15/milestone33/milestone37 non-regression suites remained green after the
  TypeScript adapter update.

## Decision Log

- Decision: classify Phase 14 as risk tier `1`.
  Rationale: work changes recommendation and extraction behavior but does not require schema,
  migration, or persistence format changes.
  Date/Author: 2026-02-10 / Codex

- Decision: enforce strict Node runner detection with unambiguous selection only.
  Rationale: roadmap requires strictness; ambiguous `jest`+`vitest` signals must remain
  conservative and avoid guessing.
  Date/Author: 2026-02-10 / Codex

- Decision: use deterministic Node command forms:
  - Vitest targeted: `npx vitest run <target>`
  - Vitest full suite: `npx vitest run`
  - Jest targeted: `npx jest --runTestsByPath <target>`
  - Jest full suite: `npx jest`
  Rationale: explicit commands are stable and align with strict runner-aware behavior.
  Date/Author: 2026-02-10 / Codex

- Decision: resolve relative TypeScript imports to deterministic candidate path sets rather than a
  single inferred file path.
  Rationale: candidate-path strategy preserves existing `.ts` behavior while adding `index.ts`
  coverage with low blast radius.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Phase 14 implementation is complete with strict Red -> Green -> Refactor evidence and no schema
change.

Implemented outcomes so far:

- `tests-for` can classify TypeScript test targets as runnable in explicit unambiguous
  `jest`/`vitest` contexts.
- `verify-plan` can emit Node runner-targeted commands and Node full-suite gates for TypeScript-only
  changed scopes in explicit unambiguous contexts.
- ambiguous Node contexts intentionally remain conservative (no runnable targeted Node command,
  default full-suite fallback retained).
- TypeScript directory imports now keep caller attribution by emitting deterministic import-path
  candidates including `index.ts`/`index.tsx`.

Validation outcome:

- milestone61 suite is green after strict Red capture.
- TypeScript/Python/Rust non-regression suites stayed green.
- full quality and contract validators passed.
- dogfood commands on repository root and Phase 14 fixture corpus showed expected runnable
  recommendation commands and `diff-impact` caller rows.

## Context and Orientation

`repo-scout` query recommendation behavior is implemented in `src/query/mod.rs`.
TypeScript extraction and call-edge synthesis are implemented in
`src/indexer/languages/typescript.rs`.

Phase 14 work touches:

- runner detection and command synthesis in `src/query/mod.rs`,
- TypeScript import hint resolution in `src/indexer/languages/typescript.rs`,
- strict integration contracts in `tests/milestone61_typescript_production.rs`,
- repeatable fixture corpus in `tests/fixtures/phase14/typescript_production/`.

## Contract Inputs

Phase 14 implementation consumes and aligns with:

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
- dogfooding commands run before and after implementation,
- integration-style tests in `tests/` with milestone naming,
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

Phase 14 risk tier: `1` (moderate).

Rationale: behavior and ranking/recommendation semantics change in query/indexer paths, but schema
and persistence versions remain unchanged.

Tier 1 controls applied:

- strict per-slice Red/Green/Refactor evidence,
- deterministic behavior assertions in integration tests,
- additive low-blast-radius implementation with rollback-safe scope,
- full suite + lint + contract validators before closure.

Escalation rule: if schema/migration/persistence invariants require change, pause and escalate to
Tier 2 controls before proceeding.

## Strict TDD Contract

No production code is edited for a feature slice until failing tests for that slice are observed.

Feature slices in this phase:

- strict Node runner-aware TypeScript command synthesis in `tests-for`,
- strict Node runner-aware targeted/full-suite behavior in `verify-plan`,
- strict ambiguity handling for mixed Jest/Vitest signals,
- TypeScript directory-import caller attribution (`./module` -> `./module/index.ts`) in
  `diff-impact`.

## TDD Evidence Log

- Red:
  - `cargo test --test milestone61_typescript_production -- --nocapture` failed with 4 failures
    covering Vitest/Jest recommendation behavior and directory-import caller attribution.
- Green:
  - `cargo test --test milestone61_typescript_production -- --nocapture` passed after query and
    TypeScript adapter updates.
- Refactor/non-regression:
  - `cargo test --test milestone15_typescript -- --nocapture` passed.
  - `cargo test --test milestone33_typescript_semantics -- --nocapture` passed.
  - `cargo test --test milestone37_semantic_precision -- --nocapture` passed.
  - `cargo test --test milestone60_python_recommendations -- --nocapture` passed.
  - `cargo test --test milestone49_rust_hardening -- --nocapture` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `cargo test` full suite passed.

## Plan of Work

### Milestone 61: TypeScript Runner-Aware Recommendation Closure

Milestone goal: ship strict deterministic Node runner-aware recommendation behavior for TypeScript
without regressing existing Python/Rust behavior.

Feature slices:

- 61A: failing then passing tests for `tests-for` runnable-target classification in explicit Vitest
  contexts and strict non-runnable behavior in ambiguous contexts.
- 61B: failing then passing tests for `verify-plan` targeted/full-suite command synthesis in
  explicit Vitest/Jest contexts and strict fallback in ambiguous contexts.

### Milestone 62: TypeScript Import-Resolution Edge Closure

Milestone goal: preserve caller attribution for directory imports common in TypeScript repositories.

Feature slices:

- 62A: failing then passing test proving `diff-impact` emits `called_by` rows when changed callee
  is imported as `./module` and defined in `./module/index.ts`.

## Concrete Steps

Run from repository root:

    cargo test --test milestone61_typescript_production -- --nocapture
    cargo test --test milestone15_typescript -- --nocapture
    cargo test --test milestone33_typescript_semantics -- --nocapture
    cargo test --test milestone37_semantic_precision -- --nocapture
    cargo test --test milestone60_python_recommendations -- --nocapture
    cargo test --test milestone49_rust_hardening -- --nocapture
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

Fixture dogfood:

    cargo run -- index --repo tests/fixtures/phase14/typescript_production/vitest
    cargo run -- tests-for computePlan --repo tests/fixtures/phase14/typescript_production/vitest --json
    cargo run -- verify-plan --changed-file src/service.ts --repo tests/fixtures/phase14/typescript_production/vitest --json
    cargo run -- index --repo tests/fixtures/phase14/typescript_production/jest
    cargo run -- verify-plan --changed-file src/service.ts --repo tests/fixtures/phase14/typescript_production/jest --json
    cargo run -- index --repo tests/fixtures/phase14/typescript_production/index_import
    cargo run -- diff-impact --changed-file src/util/index.ts --repo tests/fixtures/phase14/typescript_production/index_import --json

## Validation and Acceptance

Acceptance criteria for this phase:

- In explicit Vitest contexts, `tests-for` includes TypeScript runnable targets by default.
- In explicit Vitest/Jest contexts, `verify-plan` includes targeted runner commands and runner-
  appropriate full-suite gates.
- In ambiguous Node runner contexts, Node targeted commands are not emitted and conservative
  fallback behavior remains deterministic.
- TypeScript directory imports to `index.ts` preserve `diff-impact` caller attribution.
- Existing TypeScript/Python/Rust non-regression tests remain green.

## Idempotence and Recovery

Phase 14 changes are additive and deterministic. Re-indexing the same repository should preserve
stable JSON and terminal ordering.

Rollback strategy if needed:

- revert only Phase 14 files (`src/query/mod.rs`, `src/indexer/languages/typescript.rs`,
  `tests/milestone61_typescript_production.rs`, `tests/fixtures/phase14/typescript_production/`,
  docs updates),
- keep failing milestone61 tests to preserve defect contract,
- re-run milestone61 Red to re-isolate regression.

## Planned / Updated Artifacts

- `agents/plans/repo-scout-phase14-execplan.md`
- `tests/milestone61_typescript_production.rs`
- `tests/fixtures/phase14/typescript_production/README.md`
- `tests/fixtures/phase14/typescript_production/vitest/package.json`
- `tests/fixtures/phase14/typescript_production/vitest/src/service.ts`
- `tests/fixtures/phase14/typescript_production/vitest/tests/service.test.ts`
- `tests/fixtures/phase14/typescript_production/jest/package.json`
- `tests/fixtures/phase14/typescript_production/jest/src/service.ts`
- `tests/fixtures/phase14/typescript_production/jest/src/service.spec.ts`
- `tests/fixtures/phase14/typescript_production/ambiguous/package.json`
- `tests/fixtures/phase14/typescript_production/ambiguous/src/service.ts`
- `tests/fixtures/phase14/typescript_production/ambiguous/tests/service.test.ts`
- `tests/fixtures/phase14/typescript_production/index_import/src/util/index.ts`
- `tests/fixtures/phase14/typescript_production/index_import/src/app.ts`
- `src/query/mod.rs`
- `src/indexer/languages/typescript.rs`
- `README.md`
- `docs/cli-reference.md`
- `docs/architecture.md`
- `docs/json-output.md`
- `docs/performance-baseline.md`
- `docs/dogfood-log.md`

## Revision History

2026-02-10: Created and actively updated Phase 14 execution plan with strict TDD evidence for
TypeScript runner-aware recommendations and import-resolution production closure.
