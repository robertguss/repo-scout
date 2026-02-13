# Build `repo-scout` Phase 15 Cross-Language Production Convergence

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase14-execplan.md` and
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is scoped to cross-language
convergence work before final GA hardening.

## Purpose / Big Picture

Phase 15 aligns cross-language operator behavior so the same scope and test-path semantics apply
consistently across Rust, Go, Python, and TypeScript workflows.

User-visible outcomes for implemented slices: Go `_test.go` files now participate in the same
test-like filtering and support-target discovery behavior as the other production-ready languages,
and Go recommendation flows are now runnable by default with deterministic command synthesis in
`tests-for` and `verify-plan`.

## Progress

- [x] (2026-02-10 01:24Z) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md` to confirm the next roadmap phase
      is Phase 15.
- [x] (2026-02-10 01:25Z) Declared risk tier `1` before implementation and ran required pre-slice
      dogfooding commands for symbol `is_test_like_file_name`.
- [x] (2026-02-10 01:28Z) Added failing integration tests in
      `tests/milestone62_cross_language_convergence.rs` covering Go `_test.go` scope filtering and
      `tests-for --include-support` discovery.
- [x] (2026-02-10 01:29Z) Observed strict Red via
      `cargo test --test milestone62_cross_language_convergence -- --nocapture` (2 failures).
- [x] (2026-02-10 01:32Z) Implemented minimal production fix in `src/query/mod.rs`: shared
      test-like classifier now includes Go `_test.go`; `tests-for` now applies the same classifier
      used by query scope filters.
- [x] (2026-02-10 01:33Z) Observed Green via
      `cargo test --test milestone62_cross_language_convergence -- --nocapture`.
- [x] (2026-02-10 01:36Z) Added mixed-language regression coverage proving scope normalization for
      Rust/Go/Python/TypeScript fallback rows in one corpus.
- [x] (2026-02-10 01:44Z) Completed post-slice dogfooding and quality gates:
      `cargo fmt`, `cargo run -- index --repo .`, `cargo run -- find is_test_like_file_name --repo .`,
      `cargo run -- refs is_test_like_file_name --repo .`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 01:48Z) Declared risk tier `1` for the next Phase 15 slice and ran pre-slice
      dogfooding commands for symbol `test_command_for_target`.
- [x] (2026-02-10 01:50Z) Added failing integration tests in
      `tests/milestone62_cross_language_convergence.rs` for Go runnable-target defaults in
      `tests-for` and Go targeted/full-suite command synthesis in `verify-plan`.
- [x] (2026-02-10 01:51Z) Observed strict Red via
      `cargo test --test milestone62_cross_language_convergence -- --nocapture` (2 failures).
- [x] (2026-02-10 01:56Z) Implemented minimal production fix in `src/query/mod.rs`:
      `go_test_command_for_target` synthesis for Go `_test.go` targets and `go test ./...`
      full-suite gate selection for Go-only changed scope.
- [x] (2026-02-10 01:57Z) Observed Green via
      `cargo test --test milestone62_cross_language_convergence -- --nocapture`.
- [x] (2026-02-10 02:05Z) Added reusable fixture corpus under
      `tests/fixtures/phase15/go_recommendations/` and refactored milestone62 Go fixture setup to
      `include_str!` files.
- [x] (2026-02-10 02:14Z) Completed post-slice dogfooding + full validation:
      `cargo run -- index --repo .`, `cargo run -- find go_test_command_for_target --repo .`,
      `cargo run -- refs go_test_command_for_target --repo .`,
      `cargo run -- index --repo tests/fixtures/phase15/go_recommendations`,
      `cargo run -- tests-for PlanPhase62 --repo tests/fixtures/phase15/go_recommendations --json`,
      `cargo run -- verify-plan --changed-file src/service.go --repo tests/fixtures/phase15/go_recommendations --json`,
      `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:05Z) Declared risk tier `1` for convergence-pack validation slice and ran
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 02:07Z) Added failing integration tests in
      `tests/milestone63_cross_language_convergence_pack.rs` for Phase 15 fixture-pack layout,
      convergence-pack script/Just wiring, and integrated per-language command contracts.
- [x] (2026-02-10 02:08Z) Observed strict Red via
      `cargo test --test milestone63_cross_language_convergence_pack -- --nocapture` (compile-time
      fixture-missing failures).
- [x] (2026-02-10 02:10Z) Implemented convergence-pack assets:
      `tests/fixtures/phase15/convergence_pack/`, `scripts/check_phase15_convergence_pack.sh`,
      and `Justfile` target `phase15-convergence-pack`.
- [x] (2026-02-10 02:11Z) Observed Green via
      `cargo test --test milestone63_cross_language_convergence_pack -- --nocapture`.
- [x] (2026-02-10 02:17Z) Completed post-slice dogfooding and full gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`,
      `bash scripts/check_phase15_convergence_pack.sh`,
      `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:36Z) Remaining roadmap work after this plan transitioned to Phase 16
      High-Bar/GA hardening (`agents/plans/repo-scout-phase16-execplan.md` created and active).

## Surprises & Discoveries

- Observation: Go production support existed in language adapters, but shared test-like
  classification omitted `*_test.go`, causing cross-language scope drift for fallback filtering.
  Evidence: Red failure in
  `milestone62_exclude_tests_filters_go_test_suffix_paths` showed `src/service_test.go` still
  present under `--exclude-tests`.

- Observation: `tests-for` used a separate SQL path-pattern list that diverged from
  `is_test_like_path`, so behavior drift could recur as language patterns evolve.
  Evidence: Red failure in
  `milestone62_tests_for_include_support_recognizes_go_test_suffix` showed missing Go support
  targets despite test-like naming.

- Observation: Go `_test.go` targets remained non-runnable by default after slice 62 because
  command synthesis had no Go path.
  Evidence: Red failures in
  `milestone62_tests_for_uses_go_targets_by_default_when_runnable` and
  `milestone62_verify_plan_emits_go_targeted_and_full_suite_steps`.

- Observation: dependent dogfood commands must be sequential; running `index` and query commands in
  parallel against the same fixture can transiently show empty query output.
  Evidence: initial parallel fixture dogfood run returned empty `tests-for` output, while the
  subsequent sequential run produced expected Go runnable target rows.

- Observation: `include_str!`-backed convergence fixtures intentionally fail compilation when pack
  files are missing, producing fast Red evidence for fixture-pack drift.
  Evidence: initial milestone63 Red failed compilation with missing
  `tests/fixtures/phase15/convergence_pack/...` files.

## Decision Log

- Decision: classify this Phase 15 slice as risk tier `1`.
  Rationale: behavior changes are limited to query/recommendation filtering and do not alter
  schema, migration, or persistence invariants.
  Date/Author: 2026-02-10 / Codex

- Decision: use `is_test_like_path` as the single semantic source for `tests-for` classification.
  Rationale: one classifier prevents per-command drift and enforces cross-language semantics from a
  shared function.
  Date/Author: 2026-02-10 / Codex

- Decision: add a mixed-language integration test in the same milestone file to lock behavior
  across all four production-ready languages.
  Rationale: convergence claims need one integrated assertion corpus, not only single-language
  point checks.
  Date/Author: 2026-02-10 / Codex

- Decision: map Go `_test.go` targets to deterministic package-level commands
  (`go test ./<package_dir>` or `go test .`).
  Rationale: `tests-for`/`verify-plan` target rows are file-based; package-level `go test` is the
  minimal runnable deterministic command without parsing test function names.
  Date/Author: 2026-02-10 / Codex

- Decision: select `go test ./...` as the `verify-plan` full-suite gate for Go-only changed scope.
  Rationale: this mirrors existing per-language full-suite selection strategy
  (`cargo test`/`pytest`/`npx vitest run`/`npx jest`) while preserving deterministic fallback order.
  Date/Author: 2026-02-10 / Codex

- Decision: implement the integrated convergence validation pack as a standalone script plus
  integration test (`scripts/check_phase15_convergence_pack.sh` +
  `tests/milestone63_cross_language_convergence_pack.rs`) instead of only prose docs.
  Rationale: executable pack checks are directly reusable by GA hardening and reduce drift risk.
  Date/Author: 2026-02-10 / Codex

- Decision: model the pack as five isolated fixture repos (Rust, Go, Python, TypeScript+Vitest,
  TypeScript+Jest) under one phase directory.
  Rationale: isolated fixtures keep runner/context detection deterministic and make per-language
  contract expectations explicit.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Phase 15 now has three completed convergence slices: cross-language test-like filtering semantics,
Go runnable recommendation parity, and an integrated cross-language command-contract validation
pack.

Implemented outcomes:

- Go `_test.go` paths now classify as test-like under shared scope filters.
- `find`/`refs` fallback query filtering now excludes Go test files when `--exclude-tests` is used.
- `tests-for --include-support` now discovers Go support targets through the same classifier used by
  scope filtering.
- `tests-for` now classifies Go `_test.go` targets as runnable by default via deterministic
  package-level `go test` commands.
- `verify-plan` now emits Go targeted commands (`go test ./<package_dir>`) and selects
  `go test ./...` for Go-only full-suite gates.
- New mixed-language integration coverage validates normalized scope semantics across Rust, Go,
  Python, and TypeScript fallback rows.
- New fixture corpus under `tests/fixtures/phase15/go_recommendations/` supports repeatable
  dogfooding and future Phase 15/16 checks.
- Integrated convergence-pack fixture corpus now exists under
  `tests/fixtures/phase15/convergence_pack/` for Rust, Go, Python, TypeScript+Vitest, and
  TypeScript+Jest scenarios.
- Reusable convergence-pack gate script `scripts/check_phase15_convergence_pack.sh` and Just target
  `phase15-convergence-pack` now validate `tests-for` and `verify-plan` command contracts across
  all four languages simultaneously.
- Integration suite `tests/milestone63_cross_language_convergence_pack.rs` now asserts pack layout,
  script/Just wiring, schema stability, and per-language targeted/full-suite command expectations.

Remaining for full Phase 15 closure:

- phase-level convergence slices are complete; remaining roadmap execution is Phase 16 High-Bar/GA
  hardening (large-repo benchmarks, issue-budget triage, release checklist closure).

## Context and Orientation

Query scope and recommendation logic are implemented in `src/query/mod.rs`.

The Phase 15 slice in this plan touches:

- shared test-like path classifier in `src/query/mod.rs`,
- test-target discovery path in `src/query/mod.rs` (`test_targets_for_symbol`),
- Go runnable command synthesis and full-suite gate selection in `src/query/mod.rs`,
- convergence integration tests in `tests/milestone62_cross_language_convergence.rs`,
- integrated convergence-pack integration tests in
  `tests/milestone63_cross_language_convergence_pack.rs`,
- reusable fixture corpus in `tests/fixtures/phase15/go_recommendations/`,
- integrated fixture corpus in `tests/fixtures/phase15/convergence_pack/`,
- convergence-pack script in `scripts/check_phase15_convergence_pack.sh`,
- Justfile target wiring in `Justfile`,
- phase-level docs updates in `README.md`, `docs/cli-reference.md`, `docs/architecture.md`,
  `docs/json-output.md`, `legacy performance baseline doc (removed)`, and `docs/dogfood-log.md`.

## Contract Inputs

Phase 15 execution consumes and aligns with:

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

Local no-commit-range variant used during implementation:

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
    cargo run -- find is_test_like_file_name --repo . --json
    cargo run -- refs is_test_like_file_name --repo . --json
    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json

Dogfood commands used after implementation:

    cargo run -- index --repo .
    cargo run -- find is_test_like_file_name --repo .
    cargo run -- refs is_test_like_file_name --repo .
    cargo run -- index --repo .
    cargo run -- find go_test_command_for_target --repo .
    cargo run -- refs go_test_command_for_target --repo .
    cargo run -- index --repo tests/fixtures/phase15/go_recommendations
    cargo run -- tests-for PlanPhase62 --repo tests/fixtures/phase15/go_recommendations --json
    cargo run -- verify-plan --changed-file src/service.go --repo tests/fixtures/phase15/go_recommendations --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    bash scripts/check_phase15_convergence_pack.sh
    cargo test

## Risk Tier and Required Controls

Phase 15 slice risk tier: `1` (moderate).

Rationale: changes are behavioral and confined to query/filter logic. No schema version, migrations,
or on-disk model changes are required.

Tier 1 controls applied:

- strict per-slice Red/Green/Refactor evidence,
- deterministic integration tests for changed behavior,
- additive minimal-scope implementation,
- full-suite validation and contract validators.

Escalation rule: if future Phase 15 slices touch schema/persistence invariants, pause and escalate
to Tier 2 controls before implementation.

## Strict TDD Contract

No production code is edited for a feature slice until failing tests for that exact slice are
observed.

Feature slices implemented in this plan iteration:

- 62A: Go `_test.go` path handling under `--exclude-tests` for fallback query results.
- 62B: Go `_test.go` support-target discovery for `tests-for --include-support`.
- 62C: integrated mixed-language scope-normalization guardrail across Rust/Go/Python/TypeScript.
- 63A: Go `_test.go` runnable default classification in `tests-for` without `--include-support`.
- 63B: Go targeted/full-suite command synthesis in `verify-plan` for Go-only changed scope.
- 64A: integrated convergence-pack fixture corpus across Rust/Go/Python/TypeScript runner
  scenarios.
- 64B: executable convergence-pack validator script + Just target wiring.
- 64C: end-to-end cross-language command-contract integration test suite over the pack.

## TDD Evidence Log

- Red:
  - `cargo test --test milestone62_cross_language_convergence -- --nocapture`
    failed with 2 failures before slice 62 production edits.
  - `cargo test --test milestone62_cross_language_convergence -- --nocapture`
    failed with 2 additional failures before slice 63 production edits.
  - `cargo test --test milestone63_cross_language_convergence_pack -- --nocapture`
    failed before slice 64 implementation with compile-time missing-fixture errors.
- Green:
  - `cargo test --test milestone62_cross_language_convergence -- --nocapture` passed after slice
    62 `src/query/mod.rs` update.
  - `cargo test --test milestone62_cross_language_convergence -- --nocapture` passed after slice
    63 `src/query/mod.rs` update.
  - `cargo test --test milestone63_cross_language_convergence_pack -- --nocapture` passed after
    fixture/script/Just wiring implementation.
- Refactor/non-regression:
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `cargo test --test milestone22_recommendation_quality -- --nocapture` passed.
  - `cargo test --test milestone23_verify_plan_precision -- --nocapture` passed.
  - `cargo test --test milestone60_python_recommendations -- --nocapture` passed.
  - `cargo test --test milestone61_typescript_production -- --nocapture` passed.
  - `bash scripts/check_phase15_convergence_pack.sh` passed.
  - `cargo test` full suite passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.

## Plan of Work

Milestones 62 through 64 of Phase 15 implement convergence for shared test-like path semantics, Go
runnable recommendation parity, and an integrated cross-language validation pack.

Work sequence:

1. Add failing integration tests proving current Go `_test.go` behavior diverges from existing
   Rust/Python/TypeScript test filtering expectations and from `tests-for` include-support
   behavior.
2. Apply minimal production change in `src/query/mod.rs`:
   - extend shared file-name classifier with `_test.go`,
   - remove duplicated SQL-only test-path rules from `test_targets_for_symbol`,
   - classify discovery rows through shared `is_test_like_path`.
3. Add mixed-language guardrail test to keep cross-language scope semantics converged.
4. Refresh docs where test-like path semantics are enumerated.
5. Add failing tests for Go runnable defaults and Go targeted/full-suite recommendation commands.
6. Implement minimal Go command synthesis updates in `src/query/mod.rs`.
7. Add reusable fixture corpus for Go recommendation convergence and refresh docs/perf references.
8. Add failing convergence-pack tests spanning fixture layout, script/Just wiring, and per-language
   command contracts.
9. Add integrated convergence-pack fixture corpus across Rust/Go/Python/TypeScript scenarios.
10. Add convergence-pack validator script and Just target, then prove pack contracts with a single
    executable gate.

## Concrete Steps

Run from repository root:

    cargo run -- index --repo .
    cargo run -- find is_test_like_file_name --repo . --json
    cargo run -- refs is_test_like_file_name --repo . --json
    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json
    cargo test --test milestone62_cross_language_convergence -- --nocapture
    cargo test --test milestone62_cross_language_convergence -- --nocapture
    cargo test --test milestone62_cross_language_convergence -- --nocapture
    cargo test --test milestone62_cross_language_convergence -- --nocapture
    cargo test --test milestone63_cross_language_convergence_pack -- --nocapture
    cargo test --test milestone63_cross_language_convergence_pack -- --nocapture
    cargo test --test milestone22_recommendation_quality -- --nocapture
    cargo test --test milestone23_verify_plan_precision -- --nocapture
    cargo test --test milestone60_python_recommendations -- --nocapture
    cargo test --test milestone61_typescript_production -- --nocapture
    cargo fmt
    cargo run -- index --repo .
    cargo run -- find is_test_like_file_name --repo .
    cargo run -- refs is_test_like_file_name --repo .
    cargo run -- index --repo .
    cargo run -- find go_test_command_for_target --repo .
    cargo run -- refs go_test_command_for_target --repo .
    cargo run -- index --repo tests/fixtures/phase15/go_recommendations
    cargo run -- tests-for PlanPhase62 --repo tests/fixtures/phase15/go_recommendations --json
    cargo run -- verify-plan --changed-file src/service.go --repo tests/fixtures/phase15/go_recommendations --json
    bash scripts/check_phase15_convergence_pack.sh
    cargo test
    cargo clippy --all-targets --all-features -- -D warnings
    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## Validation and Acceptance

Acceptance criteria for this slice:

- `--exclude-tests` removes Go `_test.go` fallback rows while preserving non-test Go source rows.
- `tests-for --include-support` includes Go `_test.go` targets as support rows.
- `tests-for` default output includes runnable Go `_test.go` targets as
  `target_kind = integration_test_file`.
- `verify-plan` emits Go targeted steps (`go test ./<package_dir>`) and Go full-suite gates
  (`go test ./...`) for Go-only changed scope.
- Mixed-language `refs --code-only --exclude-tests` results keep source rows and omit test-like
  rows across Rust, Go, Python, and TypeScript in one integrated test corpus.
- Integrated convergence-pack checks verify `tests-for` + `verify-plan` command contracts across
  Rust, Go, Python, TypeScript+Vitest, and TypeScript+Jest fixtures via one script gate.

Validation evidence is satisfied by:

- passing `tests/milestone62_cross_language_convergence.rs`,
- passing `tests/milestone63_cross_language_convergence_pack.rs`,
- passing `bash scripts/check_phase15_convergence_pack.sh`,
- passing repository-wide `cargo test`,
- passing contract validators listed above.

## Review and CI Gates

Before PR merge/update for this phase:

- run `checklists/PR_CONTRACT_CHECKLIST.md`,
- run contract validators and ensure `.github/workflows/contract-gates.yml` green,
- include Red/Green/Refactor evidence in PR body headings per
  `.github/pull_request_template.md`.

Tier 1 posture does not require mandatory adversarial checklist completion, but future higher-risk
Phase 15 slices must escalate and adopt the required controls.
