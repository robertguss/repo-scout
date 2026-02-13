# Build `repo-scout` Phase 20 Agent-First CLI Contract and Workflow Runtime

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows:

1. `agents/plans/repo-scout-phase19-execplan.md`
2. `docs/plans/2026-02-13-agent-first-cli-spec.md`

## Purpose / Big Picture

Phase 20 turns `repo-scout` into an agent-first analysis interface where machine contracts are
primary. After this phase, coding agents can execute deterministic index/query/refactor workflows
without guessing command semantics or stitching fragile per-command parsers.

User-visible outcomes for agent workflows:

1. Uniform JSON envelopes and error contracts across command families.
2. Explicit index freshness semantics (no silent empty-success on missing/stale analysis state).
3. Stronger precision and explainability for refactoring diagnostics (`dead`, `test-gaps`,
   `boundary`, `coupling`, `rename-check`).
4. New orchestration primitives (`resolve`, `query`, `refactor-plan`) to reduce subprocess churn.

## Contract Inputs

This plan is governed by:

1. `AGENTS.md`.
2. `agents/PLANS.md`.
3. `contracts/core/RISK_TIER_POLICY.md`.
4. `contracts/languages/RUST_CODING_CONTRACT.md`.
5. `templates/TASK_PACKET_TEMPLATE.md`.
6. `templates/TEST_PLAN_TEMPLATE.md`.
7. `templates/EVIDENCE_PACKET_TEMPLATE.md`.
8. `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.
9. `docs/plans/2026-02-13-agent-first-cli-spec.md`.

Template mapping:

1. Task packet mapping: sections `Purpose / Big Picture`, `Scope`, `Constraints`,
   `Risk Tier and Required Controls`, `Acceptance Criteria`, and rollback sections.
2. Test plan mapping: milestone feature slices each define Red -> Green -> Refactor commands,
   including boundary/negative assertions.
3. Evidence packet mapping: each slice requires red/green/refactor transcript capture, then closure
   validators.

## AGENTS.md Constraints

Consulted path: `AGENTS.md`.

Effective constraints enforced by this plan:

1. Strict TDD at feature-slice granularity: no production code before failing tests exist.
2. Risk tier declaration required before implementation.
3. Dogfooding required before and after slices:
   `cargo run -- index --repo .`,
   `cargo run -- find <target_symbol> --repo . --json`,
   `cargo run -- refs <target_symbol> --repo . --json`,
   then post-slice non-JSON runs and `cargo test`.
4. Integration-style tests in `tests/` with milestone-focused naming.
5. In `src/` production code, do not introduce `unwrap()`/`expect()`/`panic!()`.
6. Required validators before PR updates:
   `bash scripts/validate_tdd_cycle.sh --base origin/main` and
   `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.

## Risk Tier and Required Controls

Tier: `2`.

Rationale: this phase changes cross-cutting CLI contracts and recommendation semantics used directly
by agents to drive edits. Contract regressions can cause high-blast-radius automation failures or
incorrect refactoring actions.

Tier 2 controls required and applied:

1. Red -> Green -> Refactor evidence for each feature slice.
2. Task packet/test plan/evidence completeness in PR body.
3. Adversarial review checklist completion via
   `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.
4. Security and performance impact notes captured in evidence.
5. Explicit rollback conditions and reversible migration strategy.

## Scope

In scope:

1. `src/cli.rs` global argument contract evolution and new command wiring.
2. `src/main.rs` execution flow and standardized error handling.
3. `src/output.rs` unified success/error envelopes and schema metadata.
4. `src/query/mod.rs` and submodules for confidence/provenance payload extensions.
5. New or expanded modules for schema registry, symbol resolution, and batch query execution.
6. Integration tests under `tests/` for all new contracts and migration guarantees.
7. Docs updates (`docs/cli-reference.md`, `docs/json-output.md`, `docs/agent-playbook-codex.md`).

Out of scope:

1. schema v4 persistence migration unless required by a demonstrated blocker,
2. non-CLI UI work,
3. mandatory MCP daemon shipping in this phase (may be prototyped only).

## Constraints

1. Existing command names must remain backward-compatible.
2. Contract changes must provide migration windows when field renames/removals are needed.
3. Deterministic ordering must be documented and preserved for JSON arrays.
4. Precision-first defaults are mandatory for refactoring diagnostics.

## Acceptance Criteria

1. `status`, `index`, and major query/refactor commands expose deterministic JSON envelopes with
   schema identifiers.
2. Index-missing and index-stale states are machine-distinguishable from no-result states.
3. Refactor diagnostics include confidence/rationale metadata and required behavior fixes from Phase
   19 carry through.
4. New `resolve`, `query`, and `refactor-plan` command contracts exist with integration coverage.
5. Full test suite and required validators pass.

## Test Expectations

Required integration tests:

1. contract tests for envelope/error semantics and exit code behavior,
2. deterministic JSON tests for updated command outputs,
3. regression tests preserving existing command names and core behavior,
4. performance-oriented smoke tests for batch query throughput against equivalent subprocess loops.

Required performance/security checks:

1. capture baseline vs post-change runtime for representative agent flows,
2. verify low-confidence recommendations are labeled and never implied as certain,
3. verify strict mode exits non-zero on partial/ambiguous states.

## Milestones

### Milestone 1: Contract Foundation (Envelope, Errors, Exit Semantics)

Milestone goal: establish one stable machine contract surface and explicit failure semantics.

Feature slice 1.1 introduces a top-level JSON success/error envelope and schema IDs for selected
commands (`find`, `refs`, `status`) with backward-compatibility gating.

Red:

    cargo test --test milestone110_agent_contract_envelope -- --nocapture

Green:

    Implement envelope structs and response adapters.
    cargo test --test milestone110_agent_contract_envelope -- --nocapture

Refactor:

    cargo test --test milestone110_agent_contract_envelope -- --nocapture
    cargo test

Feature slice 1.2 introduces deterministic error codes and process exit taxonomy for usage/index/
internal/partial states.

Red:

    cargo test --test milestone111_agent_exit_code_contract -- --nocapture

Green:

    Implement standardized errors and mapped exits.
    cargo test --test milestone111_agent_exit_code_contract -- --nocapture

Refactor:

    cargo test --test milestone111_agent_exit_code_contract -- --nocapture
    cargo test

Feature slice 1.3 adds `schema --json` introspection command exposing command schema IDs and
versions.

Red:

    cargo test --test milestone112_schema_registry_command -- --nocapture

Green:

    Implement schema registry command and docs wiring.
    cargo test --test milestone112_schema_registry_command -- --nocapture

Refactor:

    cargo test --test milestone112_schema_registry_command -- --nocapture
    cargo test

### Milestone 2: Index Freshness and Lifecycle Safety

Milestone goal: prevent ambiguous no-data states in agent workflows.

Feature slice 2.1 adds explicit index freshness metadata in `status --json` and index-state checks
in read commands.

Red:

    cargo test --test milestone113_index_freshness_contract -- --nocapture

Green:

    Implement freshness metadata and no-result vs missing/stale differentiation.
    cargo test --test milestone113_index_freshness_contract -- --nocapture

Refactor:

    cargo test --test milestone113_index_freshness_contract -- --nocapture
    cargo test

Feature slice 2.2 adds `--require-index-fresh` and `--auto-index` behavior for selected read
commands.

Red:

    cargo test --test milestone114_index_freshness_flags -- --nocapture

Green:

    Implement flags and command-level integration.
    cargo test --test milestone114_index_freshness_flags -- --nocapture

Refactor:

    cargo test --test milestone114_index_freshness_flags -- --nocapture
    cargo test

### Milestone 3: Precision and Explainability Hardening (Phase 19 Carry-Forward)

Milestone goal: complete refactoring diagnostic trust model under the new contract envelope.

Feature slice 3.1 finalizes `dead` confidence/rationale and conservative/aggressive mode contracts.

Red:

    cargo test --test milestone115_dead_confidence_contract -- --nocapture

Green:

    Implement mode and explainability fields in final envelope.
    cargo test --test milestone115_dead_confidence_contract -- --nocapture

Refactor:

    cargo test --test milestone115_dead_confidence_contract -- --nocapture
    cargo test

Feature slice 3.2 finalizes `test-gaps`, `boundary --public-only`, `coupling` noise defaults, and
`rename-check` semantic-vs-lexical split under deterministic JSON.

Red:

    cargo test --test milestone116_refactor_diagnostic_precision_contract -- --nocapture

Green:

    Implement behavior fixes and payload fields.
    cargo test --test milestone116_refactor_diagnostic_precision_contract -- --nocapture

Refactor:

    cargo test --test milestone116_refactor_diagnostic_precision_contract -- --nocapture
    cargo test

### Milestone 4: Agent Throughput and Identity Commands

Milestone goal: reduce subprocess orchestration overhead and ambiguity.

Feature slice 4.1 adds `resolve` command with canonical `symbol_id` candidates and ambiguity
metadata.

Red:

    cargo test --test milestone117_resolve_symbol_contract -- --nocapture

Green:

    Implement `resolve` query and output model.
    cargo test --test milestone117_resolve_symbol_contract -- --nocapture

Refactor:

    cargo test --test milestone117_resolve_symbol_contract -- --nocapture
    cargo test

Feature slice 4.2 adds `query` batch command with JSON/JSONL request handling and per-request
status.

Red:

    cargo test --test milestone118_query_batch_contract -- --nocapture

Green:

    Implement batch execution and deterministic JSONL responses.
    cargo test --test milestone118_query_batch_contract -- --nocapture

Refactor:

    cargo test --test milestone118_query_batch_contract -- --nocapture
    cargo test

### Milestone 5: Refactor Orchestration Command

Milestone goal: provide one conservative end-to-end planning call for agents.

Feature slice 5.1 adds `refactor-plan` command composing diagnostics and preflight reports into a
single ranked plan object.

Red:

    cargo test --test milestone119_refactor_plan_contract -- --nocapture

Green:

    Implement orchestration pipeline with risk/confidence labels.
    cargo test --test milestone119_refactor_plan_contract -- --nocapture

Refactor:

    cargo test --test milestone119_refactor_plan_contract -- --nocapture
    cargo test

### Milestone 6: Closure, Dogfood, and Validation

Milestone goal: verify practical agent utility and finalize contract documentation.

Feature slice 6.1 updates docs and dogfood transcripts for agent-first flows.

Red:

    cargo test --test milestone120_agent_workflow_docs_contract -- --nocapture

Green:

    Update docs and evidence artifacts.
    cargo test --test milestone120_agent_workflow_docs_contract -- --nocapture

Refactor:

    cargo test --test milestone120_agent_workflow_docs_contract -- --nocapture
    cargo test

Required closure commands:

    cargo run -- index --repo .
    cargo run -- schema --json --repo .
    cargo run -- status --repo . --json
    cargo run -- resolve run --repo . --json
    cargo run -- query --repo . --format jsonl --input tests/fixtures/phase20/query_batch_sample.jsonl
    cargo run -- refactor-plan src/main.rs --repo . --json
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## Rollback Conditions

Rollback is required if:

1. JSON contract changes break existing automation without a compatibility window,
2. index freshness semantics produce false stale/missing signals in normal workflows,
3. batch/orchestration commands introduce non-deterministic outputs,
4. refactoring diagnostics regress precision from Phase 19 baseline.

## Rollback Plan

1. Revert Phase 20 command-contract and orchestration commits in reverse milestone order.
2. Re-run baseline command contract tests and full suite.
3. Keep failing Phase 20 contract tests in a dedicated follow-up branch if partial rollback is
   required.

## Progress

- [x] (2026-02-13) Reviewed agent-first spec and prior phase plans.
- [x] (2026-02-13) Authored Phase 20 ExecPlan with contract-first milestones.
- [ ] Milestone 1 implementation and tests.
- [ ] Milestone 2 implementation and tests.
- [ ] Milestone 3 implementation and tests.
- [ ] Milestone 4 implementation and tests.
- [ ] Milestone 5 implementation and tests.
- [ ] Milestone 6 closure and validators.

## Surprises & Discoveries

- Observation: current command breadth is high, but machine-contract consistency and explicit
  failure semantics are the dominant blockers for autonomous agent use.
- Observation: precision defects and contract defects compound; both must be addressed in one
  contract-first phase to avoid repeated migration churn.

## Decision Log

- Decision: designate Phase 20 as Tier 2.
  Rationale: contract-level changes affect cross-command automation and refactoring safety.
  Date/Author: 2026-02-13 / Codex

- Decision: implement contract foundation before throughput/orchestration commands.
  Rationale: agent integrations need stable envelope/error semantics before new high-level features.
  Date/Author: 2026-02-13 / Codex

- Decision: fold Phase 19 precision goals into Milestone 3 under the agent-first envelope.
  Rationale: avoids dual migrations and keeps JSON contract evolution coherent.
  Date/Author: 2026-02-13 / Codex

## Outcomes & Retrospective

Not started. Must be updated at completion with:

1. delivered contract and command outcomes,
2. evidence/validator results,
3. residual gaps and proposed follow-up phases.
