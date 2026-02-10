# Build `repo-scout` Phase 18 Maintenance-Mode Hardening and Backlog Governance

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase17-execplan.md` and starts from the current
post-Phase-16/17 stable baseline.

## Purpose / Big Picture

Phase 18 formalizes maintenance-mode operations now that production closures and GA hardening are
complete. The focus is governance, repeatability, and low-friction guardrails for ongoing changes.

User-visible outcomes for this phase:

- one canonical maintenance backlog artifact with explicit prioritization and ownership,
- one executable maintenance gate pack for routine local/CI health checks,
- explicit documentation freshness expectations and validation hooks.

This phase does not introduce schema-version churn or new query command families.

## Progress

- [x] (2026-02-10) Re-read `AGENTS.md`, `agents/PLANS.md`,
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md`, and
      `agents/plans/repo-scout-phase17-execplan.md` to anchor scope and constraints.
- [x] (2026-02-10) Ran planning baseline dogfood commands:
      `cargo run -- index --repo .`,
      `cargo run -- find docs_consistency --repo . --json`,
      `cargo run -- refs docs_consistency --repo . --json`.
- [x] (2026-02-10) Authored this Phase 18 ExecPlan as planning-only work.
- [ ] Milestone 73 strict TDD maintenance backlog policy artifact and contract tests.
- [ ] Milestone 74 strict TDD maintenance gate-pack script + Just/CI wiring.
- [ ] Milestone 75 strict TDD documentation freshness guardrails and cadence checks.
- [ ] Milestone 76 docs, dogfood evidence, validator closure, and retrospective updates.

## Surprises & Discoveries

- Observation: project behavior and quality gates are currently healthy; remaining risk is mostly
  process drift (status/docs/backlog ownership) rather than runtime correctness gaps.
- Observation: `validate_tdd_cycle.sh --base origin/main` still requires
  `--allow-empty-range` on empty local ranges; maintenance workflows should explicitly account for
  that developer experience.

## Decision Log

- Decision: classify Phase 18 as risk tier `1`.
  Rationale: changes target process artifacts, scripts, and CI wiring, not schema or irreversible
  data operations.
  Date/Author: 2026-02-10 / Codex

- Decision: keep schema versions (`1`/`2`/`3`) unchanged throughout Phase 18.
  Rationale: maintenance hardening should not perturb existing downstream automation contracts.
  Date/Author: 2026-02-10 / Codex

- Decision: execute Phase 18 as strict red-green-refactor per maintenance slice, including script
  and workflow changes.
  Rationale: process automation changes can silently regress if not contract-tested.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Planning outcome:

- Phase 18 scope is constrained to maintenance operations and governance guardrails.
- Runtime feature expansion is explicitly out of scope.

Expected completion outcome:

- maintainers can run one maintenance pack command for routine confidence,
- backlog ownership and priorities are explicit and machine-checkable,
- docs freshness expectations are validated, reducing stale-status regressions.

Expected residual work:

- optional future feature phases only when backlog evidence justifies capability expansion.

## Context and Orientation

Primary modules/artifacts expected to be touched in Phase 18:

- `docs/maintenance-backlog-phase18.md` (new)
- `docs/maintenance-cadence-phase18.md` (new)
- `scripts/check_phase18_maintenance_pack.sh` (new)
- `scripts/check_phase18_docs_freshness.sh` (new)
- `Justfile`
- `.github/workflows/contract-gates.yml`
- `docs/dogfood-log.md`
- `agents/plans/repo-scout-phase18-execplan.md`
- `tests/milestone73_maintenance_backlog_policy.rs` (new)
- `tests/milestone74_maintenance_gate_pack.rs` (new)
- `tests/milestone75_docs_freshness_guardrails.rs` (new)

## Contract Inputs

Phase 18 implementation must consume and align with:

- Core policy: `contracts/core/RISK_TIER_POLICY.md`
- Language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Task framing template: `templates/TASK_PACKET_TEMPLATE.md`
- Test planning template: `templates/TEST_PLAN_TEMPLATE.md`
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

Effective constraints enforced by this plan:

- strict Red -> Green -> Refactor for each feature slice,
- risk tier declaration before implementation,
- dogfood commands before and after each milestone,
- integration-style tests in `tests/`,
- no schema/index DB commit drift,
- contract validators required before closure reporting.

If instructions conflict, the stricter contract/system rule wins.

## Risk Tier and Required Controls

Phase 18 risk tier: `1` (moderate).

Controls required:

- red/green/refactor evidence captured per milestone,
- dogfood pre/post transcripts captured in `docs/dogfood-log.md`,
- full quality gates (`fmt`, `clippy -D warnings`, `cargo test`) before closure,
- contract validators run before PR evidence closure.

## Strict TDD Contract

No production/process implementation changes are allowed for a Phase 18 slice until the owning
failing test/check is observed.

Feature slices in this phase are:

- backlog policy contract checks,
- maintenance gate-pack script/wiring,
- docs freshness guardrails.

For each slice capture:

- Red transcript: failing test/check,
- Green transcript: same test/check passing after minimal change,
- Refactor transcript: full-suite `cargo test` and quality gates passing.

## Plan of Work

### Milestone 73: Maintenance Backlog Policy Artifact

Milestone goal: establish one deterministic maintenance backlog source of truth.

Feature slice 73A adds failing tests asserting the existence/shape of a backlog artifact with
explicit fields (id, priority, owner, status, target window, notes).

Feature slice 73B implements `docs/maintenance-backlog-phase18.md` and minimal supporting updates
so tests pass.

### Milestone 74: Maintenance Gate Pack

Milestone goal: provide one repeatable command for maintenance-mode health checks.

Feature slice 74A adds failing tests for script existence and required command coverage.

Feature slice 74B implements `scripts/check_phase18_maintenance_pack.sh` plus `Justfile` wiring
(`just phase18-maintenance-pack`) and CI invocation.

### Milestone 75: Documentation Freshness Guardrails

Milestone goal: prevent stale operational status/docs over time.

Feature slice 75A adds failing tests for freshness policy artifact and script behavior.

Feature slice 75B implements `docs/maintenance-cadence-phase18.md` and
`scripts/check_phase18_docs_freshness.sh` (deterministic checks for required docs/status markers).

### Milestone 76: Closure Evidence and Validation

Milestone goal: complete docs/evidence refresh and run all required gates.

Feature slice 76A updates docs and dogfood transcript artifacts.

Feature slice 76B executes full post-change validation:

- `cargo run -- index --repo .`
- `cargo run -- find <symbol> --repo .`
- `cargo run -- refs <symbol> --repo .`
- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `bash scripts/validate_tdd_cycle.sh --base origin/main`
- `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

## Validation and Acceptance

Phase 18 is complete when all are true:

- milestone73/74/75 integration tests are green,
- maintenance pack and docs freshness scripts pass on repo root,
- docs and roadmap references are aligned with Phase 18 outputs,
- required contract validators pass,
- post-change dogfood loop passes.

## Idempotence and Recovery

Phase 18 changes are additive and idempotent:

- rerunning maintenance scripts should yield deterministic results on unchanged inputs,
- no schema or persisted index format changes are introduced,
- rollback is limited to reverting Phase 18 docs/script/test/workflow edits.
