# Build `repo-scout` Phase 17 Documentation Truth Sync and Consistency Gates

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

## Purpose / Big Picture

Phase 17 closes documentation drift and adds a lightweight automation gate so future roadmap/status
drift is detected early in CI. This phase does not change CLI behavior or schema contracts.

User-visible outcomes:

- docs and plan artifacts consistently represent current post-Phase-16 closure state,
- changelog posture moves to `Unreleased` + released entries,
- contributors can run one deterministic docs-consistency gate locally and in CI.

## Progress

- [x] (2026-02-10) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md` for scope/constraint alignment.
- [x] (2026-02-10) Ran pre-slice dogfood baseline:
      `cargo run -- index --repo .`,
      `cargo run -- find docs_consistency_check --repo . --json`,
      `cargo run -- refs docs_consistency_check --repo . --json`.
- [x] (2026-02-10) Added failing milestone71 docs-status alignment tests
      (`tests/milestone71_docs_status_alignment.rs`) and observed strict Red.
- [x] (2026-02-10) Completed Milestone 71 documentation + plan consistency closure:
      `README.md`, `docs/architecture.md`, `CHANGELOG.md`,
      `agents/plans/repo-scout-phase9-execplan.md`, and `agents/plans/README.md`.
- [x] (2026-02-10) Added failing milestone72 gate-wiring tests
      (`tests/milestone72_docs_consistency_gate.rs`) and observed strict Red.
- [x] (2026-02-10) Completed Milestone 72 docs-consistency script and wiring:
      `scripts/check_docs_consistency.sh`, `Justfile`, and
      `.github/workflows/contract-gates.yml`.
- [x] (2026-02-10) Completed post-change dogfood/quality/contract validation and evidence refresh:
      `cargo run -- index/find/refs`, `cargo test`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/check_docs_consistency.sh --repo .`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      and `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.

## Surprises & Discoveries

- Observation: the repository implementation/test surface is current and green, while plan/doc
  artifacts have isolated status drift points (for example Phase 9 open checkboxes and README/architecture
  posture language).

## Decision Log

- Decision: classify Phase 17 risk tier as `1`. Rationale: this phase updates docs/process assets
  and CI checks without schema/behavioral runtime changes. Date/Author: 2026-02-10 / Codex
- Decision: keep CLI/schema contracts unchanged and add additive process checks only.
  Rationale: objective is documentation truth sync and drift prevention, not feature expansion.
  Date/Author: 2026-02-10 / Codex
- Decision: treat Phase 9 as superseded/closed via later implemented phases while retaining it as a
  historical artifact.
  Rationale: avoids duplicate implementation work and removes operator confusion.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Completion outcome:

- README, architecture docs, changelog, and plan inventory align with current closure posture.
- Phase 9 no longer appears as an open implementation phase.
- new `scripts/check_docs_consistency.sh` + `just docs-consistency` + CI step provide repeatable
  drift detection.

## Context and Orientation

Primary files in scope:

- `README.md`
- `CHANGELOG.md`
- `docs/architecture.md`
- `docs/dogfood-log.md`
- `agents/plans/README.md`
- `agents/plans/repo-scout-phase9-execplan.md`
- `agents/plans/repo-scout-phase17-execplan.md`
- `scripts/check_docs_consistency.sh`
- `Justfile`
- `.github/workflows/contract-gates.yml`
- `tests/milestone71_docs_status_alignment.rs`
- `tests/milestone72_docs_consistency_gate.rs`

## Contract Inputs

Phase 17 implementation must consume and reference:

- Core policy: `contracts/core/RISK_TIER_POLICY.md`
- Language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Task template: `templates/TASK_PACKET_TEMPLATE.md`
- Test template: `templates/TEST_PLAN_TEMPLATE.md`
- Evidence template: `templates/EVIDENCE_PACKET_TEMPLATE.md`
- TDD validator: `scripts/validate_tdd_cycle.sh`
- Evidence validator: `scripts/validate_evidence_packet.sh`
- CI gate reference: `.github/workflows/contract-gates.yml`

Required validator commands before PR merge:

    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## AGENTS.md Constraints

Consulted file:

- `AGENTS.md`

Effective constraints for this phase:

- strict red-green-refactor per feature slice,
- declare risk tier before implementation,
- run dogfood pre/post commands with `repo-scout` first,
- integration-style tests in `tests/`,
- avoid schema/index DB drift,
- run contract validators before closure.

## Risk Tier and Required Controls

Phase 17 risk tier: `1` (moderate).

Controls:

- strict red/green/refactor evidence for milestone71 and milestone72,
- post-slice dogfood and full quality gates,
- contract validator runs before closure reporting,
- idempotent scripts/docs updates with deterministic pass/fail checks.

## Strict TDD Contract

No production/process implementation edits are allowed for milestone72 gate wiring until owning
failing tests exist and are observed.

Feature slices:

- Milestone 71: status-alignment contract assertions for README/architecture/plan artifacts.
- Milestone 72: docs-consistency script + Just + CI integration contract assertions.

For each slice:

- Red: failing test observed,
- Green: minimal implementation,
- Refactor: full-suite `cargo test` + quality gates.

## Plan of Work

### Milestone 71: Documentation and Plan Consistency Closure

Goals:

- update README to product-first current-state posture,
- update architecture framing to post-Phase-16 closure,
- add changelog `Unreleased` section,
- mark Phase 9 superseded and remove open checkbox ambiguity,
- update plan inventory policy for superseded handling.

### Milestone 72: Automated Documentation Consistency Gate

Goals:

- add milestone72 failing tests for script/wiring requirements,
- implement `scripts/check_docs_consistency.sh`,
- wire `just docs-consistency`,
- wire CI invocation in `.github/workflows/contract-gates.yml`.

## Validation and Acceptance

- `cargo test --test milestone71_docs_status_alignment -- --nocapture`
- `cargo test --test milestone72_docs_consistency_gate -- --nocapture`
- `bash scripts/check_docs_consistency.sh --repo .`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `bash scripts/validate_tdd_cycle.sh --base origin/main`
- `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

## Idempotence and Recovery

Phase 17 is additive and idempotent:

- rerunning `check_docs_consistency.sh` should be deterministic for unchanged docs,
- no runtime schema or index-format changes are introduced,
- rollback can be done by reverting only Phase 17 docs/tests/script/workflow edits.
