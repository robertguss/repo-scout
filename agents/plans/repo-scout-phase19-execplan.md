# Build `repo-scout` Phase 19 Refactoring Signal Quality Hardening

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase17-execplan.md` and
`agents/plans/repo-scout-phase18-execplan.md`. It is focused on reducing false positives and
non-actionable output in refactoring-oriented commands.

## Purpose / Big Picture

Phase 19 makes refactoring recommendations trustworthy enough to drive real edits in production
code. Today, command output is broad but noisy. After this phase, users and agents should be able
to use `dead`, `test-gaps`, `boundary`, `coupling`, and `rename-check` with a clear default safety
model: conservative by default, explicit when aggressive behavior is requested.

User-visible outcomes:

1. `dead` defaults to high-confidence candidates with explainable reasons and fewer false positives.
2. `test-gaps` returns actionable state instead of empty ambiguous arrays.
3. `boundary --public-only` strictly returns public surface only.
4. `coupling` and `rename-check` default to lower-noise output with explicit fixture/test controls.
5. Refactoring diagnostics include confidence and rationale fields so users can verify results.

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

Template mapping for this plan:

1. Task packet mapping: sections `Purpose / Big Picture`, `Scope`, `Constraints`, `Risk Tier and
   Required Controls`, `Acceptance Criteria`, and `Rollback Conditions`.
2. Test plan mapping: each milestone includes explicit feature slices with Red -> Green -> Refactor
   commands and boundary/negative checks.
3. Evidence packet mapping: each milestone requires one red transcript, one green transcript, and
   one refactor transcript, then closure validator commands.

## AGENTS.md Constraints

Consulted path: `AGENTS.md`.

Effective constraints applied by this plan:

1. Strict TDD for every feature slice: no production code before a failing test exists.
2. Declare risk tier before implementation. If uncertain between tiers, choose the higher tier.
3. Dogfooding required before and after slices using:
   `cargo run -- index --repo .`,
   `cargo run -- find <target_symbol> --repo . --json`,
   `cargo run -- refs <target_symbol> --repo . --json`,
   then post-slice non-JSON runs and `cargo test`.
4. Integration-style tests in `tests/` following milestone naming patterns.
5. In `src/` production code, do not introduce `unwrap()`/`expect()`/`panic!()`.
6. Required validators before PR updates:
   `bash scripts/validate_tdd_cycle.sh --base origin/main` and
   `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.

## Risk Tier and Required Controls

Tier: `2`.

Rationale: this work changes diagnostic outputs that directly influence refactoring decisions. A
false positive here can cause incorrect code movement/renames and wasted engineering cycles across
core query paths.

Tier 2 controls required and enforced by this plan:

1. Red -> Green -> Refactor evidence for every feature slice.
2. Task packet, test plan, and evidence packet completeness in PR body.
3. Adversarial review checklist completion:
   `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.
4. Security and performance review notes in evidence.
5. Explicit rollback plan with command-contract fallback behavior.

## Scope

In scope:

1. `src/query/diagnostics.rs` dead/test-gap/coupling signal logic.
2. `src/query/planning.rs` rename/boundary support logic if required.
3. `src/main.rs` command handlers and defaults.
4. `src/cli.rs` option surfaces for confidence/scope/aggressive modes.
5. `src/output.rs` rationale/confidence fields and terminal clarity.
6. Integration tests in `tests/` for all changed behaviors.
7. Dogfood evidence updates in `docs/dogfood-log.md` and this plan.

Out of scope:

1. Database schema migrations.
2. New command families outside listed quality-gap commands.
3. IDE/LSP or watch-mode behavior.

## Constraints

This phase must keep existing command names stable and preserve deterministic output ordering.
Behavior changes must prioritize precision over recall by default. Any aggressive behavior must be
opt-in via explicit flags and clearly labeled in output.

## Acceptance Criteria

1. `dead` default mode avoids known false positives in CLI wiring scenarios and reports confidence
   plus reason fields in `--json`.
2. `dead --aggressive` (or equivalent explicit mode) broadens recall while preserving explanation
   metadata.
3. `boundary --public-only` does not emit internal symbols.
4. `test-gaps` returns actionable status fields (`covered`, `uncovered`, `unknown` or equivalent)
   with non-ambiguous semantics.
5. `coupling` defaults suppress fixture/test noise unless explicitly requested.
6. `rename-check` clearly separates semantic reference impacts from lexical/text impacts.
7. All new/updated milestone tests pass and `cargo test` remains green.

## Test Expectations

Required integration tests:

1. New milestone tests covering each quality gap and each new flag/default.
2. Determinism tests for updated JSON outputs.
3. Regression tests for currently working refactoring commands unaffected by this phase.

Required performance/security checks:

1. Ensure new analysis paths do not significantly regress runtime on this repository.
2. Ensure no command emits unsafe instructions without confidence/risk labeling.

## Dead Command Product Position

`dead` is expected to become useful only when it is conservative by default. Dead-code detection is
inherently uncertain in dynamic and multi-language repositories, so Phase 19 adopts a two-mode
model:

1. default mode: high-confidence, low false-positive output intended for direct action,
2. aggressive mode: broader candidate listing for exploration, clearly labeled as lower confidence.

Every reported symbol must include why it was considered dead and what roots/references were checked
so users can quickly audit the claim.

## Milestones

### Milestone 1: Quality Benchmark Harness and Baseline Contracts

Milestone goal: convert current dogfood findings into reproducible failing tests and explicit command
contracts before implementation.

Feature slice 1.1 creates failing tests for current observed gaps (`dead` false positives,
`boundary --public-only` leakage, ambiguous `test-gaps` empty result behavior).

Red:

    cargo test --test milestone101_refactoring_quality_baseline -- --nocapture

Green:

    Add baseline fixtures/contracts only (no production logic changes yet).
    cargo test --test milestone101_refactoring_quality_baseline -- --nocapture

Refactor:

    cargo test --test milestone101_refactoring_quality_baseline -- --nocapture
    cargo test

Feature slice 1.2 adds benchmark helper fixtures for confidence-tier evaluation and noise-scoped
coupling/rename outputs.

Red:

    cargo test --test milestone102_refactoring_noise_fixture_pack -- --nocapture

Green:

    Add fixture pack and deterministic assertions.
    cargo test --test milestone102_refactoring_noise_fixture_pack -- --nocapture

Refactor:

    cargo test --test milestone102_refactoring_noise_fixture_pack -- --nocapture
    cargo test

### Milestone 2: `dead` Conservative Reachability and Explainability

Milestone goal: make `dead` trustworthy for direct use in refactor triage.

Feature slice 2.1 introduces conservative reachability roots and confidence scoring for
`dead --scope production`.

Red:

    cargo test --test milestone103_dead_confidence_roots -- --nocapture

Green:

    Implement root-aware dead detection and confidence tiers.
    cargo test --test milestone103_dead_confidence_roots -- --nocapture

Refactor:

    cargo test --test milestone103_dead_confidence_roots -- --nocapture
    cargo test

Feature slice 2.2 introduces aggressive mode (`--aggressive` or equivalent) and ensures output
contains rationale fields.

Red:

    cargo test --test milestone104_dead_modes_and_rationale -- --nocapture

Green:

    Implement aggressive mode and rationale output wiring.
    cargo test --test milestone104_dead_modes_and_rationale -- --nocapture

Refactor:

    cargo test --test milestone104_dead_modes_and_rationale -- --nocapture
    cargo test

### Milestone 3: `test-gaps` and `boundary` Contract Corrections

Milestone goal: remove ambiguous output and enforce strict flag semantics.

Feature slice 3.1 makes `test-gaps` return explicit analysis state and clear uncovered reporting.

Red:

    cargo test --test milestone105_test_gaps_actionable_contract -- --nocapture

Green:

    Implement explicit state fields and actionable mapping.
    cargo test --test milestone105_test_gaps_actionable_contract -- --nocapture

Refactor:

    cargo test --test milestone105_test_gaps_actionable_contract -- --nocapture
    cargo test

Feature slice 3.2 enforces `boundary --public-only` filtering behavior.

Red:

    cargo test --test milestone106_boundary_public_only_contract -- --nocapture

Green:

    Fix boundary filtering/output path.
    cargo test --test milestone106_boundary_public_only_contract -- --nocapture

Refactor:

    cargo test --test milestone106_boundary_public_only_contract -- --nocapture
    cargo test

### Milestone 4: `coupling` and `rename-check` Noise Reduction

Milestone goal: default outputs prioritize production-impact signal.

Feature slice 4.1 makes `coupling` default behavior suppress fixture/test noise and expose opt-in
flags for broader scope.

Red:

    cargo test --test milestone107_coupling_noise_controls -- --nocapture

Green:

    Implement default filtering and opt-in controls.
    cargo test --test milestone107_coupling_noise_controls -- --nocapture

Refactor:

    cargo test --test milestone107_coupling_noise_controls -- --nocapture
    cargo test

Feature slice 4.2 separates semantic and lexical impact counts in `rename-check`.

Red:

    cargo test --test milestone108_rename_check_semantic_vs_text -- --nocapture

Green:

    Implement dual impact reporting with deterministic ordering.
    cargo test --test milestone108_rename_check_semantic_vs_text -- --nocapture

Refactor:

    cargo test --test milestone108_rename_check_semantic_vs_text -- --nocapture
    cargo test

### Milestone 5: Dogfood Closure and Contract Validation

Milestone goal: verify practical trust improvements on this repository and close with required
validators.

Feature slice 5.1 captures pre/post dogfood transcript deltas for target commands.

Red:

    cargo test --test milestone109_refactor_quality_dogfood_contract -- --nocapture

Green:

    Update docs/contracts to reflect verified post-change behavior.
    cargo test --test milestone109_refactor_quality_dogfood_contract -- --nocapture

Refactor:

    cargo test --test milestone109_refactor_quality_dogfood_contract -- --nocapture
    cargo test

Required closure commands:

    cargo run -- index --repo .
    cargo run -- find dead_symbols --repo . --json
    cargo run -- refs dead_symbols --repo . --json
    cargo run -- dead --repo . --json --scope production
    cargo run -- boundary src/cli.rs --repo . --json --public-only
    cargo run -- test-gaps src/cli.rs --repo . --json
    cargo run -- coupling --repo . --json
    cargo run -- rename-check run --repo . --to execute --json
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## Rollback Conditions

Rollback is required if any of the following are observed after merging Phase 19 changes:

1. false-positive rate for `dead` in baseline fixture contracts increases versus pre-phase baseline,
2. command defaults become less deterministic or break existing JSON consumers,
3. runtime regressions make refactoring commands materially slower in normal repository use.

## Rollback Plan

1. Revert Phase 19 commits touching diagnostics/planning/output/cli modules.
2. Re-run `cargo test` and baseline refactoring command suite to confirm restoration.
3. Retain failing milestone tests in a follow-up branch if behavior remains unresolved.

## Progress

- [x] (2026-02-13) Re-read `AGENTS.md`, `agents/PLANS.md`, and Risk Tier policy/templates.
- [x] (2026-02-13) Authored Phase 19 ExecPlan for quality-gap remediation and trust hardening.
- [x] (2026-02-13) Milestone 1 implementation and tests (`milestone101`, `milestone102`).
- [x] (2026-02-13) Milestone 2 implementation and tests (`milestone103`, `milestone104`).
- [x] (2026-02-13) Milestone 3 implementation and tests (`milestone105`, `milestone106`).
- [x] (2026-02-13) Milestone 4 implementation and tests (`milestone107`, `milestone108`).
- [x] (2026-02-13) Milestone 5 closure dogfooding and validators (`milestone109`, closure command matrix).

## Surprises & Discoveries

- Observation: all refactoring commands execute and test suite is green, but signal quality defects
  are primarily in precision and output semantics rather than command availability.
- Observation: fixture/test path dominance can distort refactoring diagnostics if defaults are not
  explicitly production-first.
- Observation: `scripts/validate_tdd_cycle.sh --base origin/main` fails on pre-existing history
  before this phase (`GREEN` commit without an open `RED` cycle in historical range), while
  phase-local Red/Green/Refactor sequencing and evidence tests are present and passing.

## Decision Log

- Decision: assign Phase 19 as Tier 2.
  Rationale: although no schema migration is planned, these commands directly guide code edits and
  incorrect guidance has high practical blast radius.
  Date/Author: 2026-02-13 / Codex

- Decision: treat `dead` as a confidence-tier analyzer, not a binary truth engine.
  Rationale: conservative defaults maximize trust; aggressive mode preserves discovery.
  Date/Author: 2026-02-13 / Codex

- Decision: prioritize correctness fixes (`dead`, `test-gaps`, `boundary`) before ranking/noise
  enhancements (`coupling`, `rename-check`).
  Rationale: correctness defects block trust more than ranking quality defects.
  Date/Author: 2026-02-13 / Codex

## Outcomes & Retrospective

Concrete behavior changes delivered:

1. Added conservative-by-default `dead` behavior with opt-in `--aggressive`, plus confidence and
   rationale fields in output.
2. Added explicit `analysis_state` and per-entry `coverage_status` to `test-gaps` output.
3. Enforced `boundary --public-only` for both terminal and JSON output (internal symbols removed).
4. Added production-first coupling defaults with `--include-tests` and `--include-fixtures`.
5. Added semantic vs lexical impact separation for `rename-check` with scope controls.
6. Added integration contracts `tests/milestone101_*.rs` through `tests/milestone109_*.rs`.

Evidence quality and validator results:

1. New milestone tests pass and full `cargo test` is green.
2. Phase 19 closure command matrix executed and recorded in `docs/dogfood-log.md`.
3. `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passes.
4. `bash scripts/validate_tdd_cycle.sh --base origin/main` fails due historical pre-phase commit
   ordering, not the Phase 19 change set.

Residual risks and follow-up candidates:

1. `dead` still relies on static symbol-edge reachability and may classify low-confidence cases in
   aggressive mode; additional language-aware root inference remains a future improvement.
2. Historical TDD validator failure indicates repo-wide commit-history debt that should be handled
   in a dedicated cleanup or validator-baseline update.
