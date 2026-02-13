# Build `repo-scout` Phase 17 Refactoring Command Suite

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase16-execplan.md`,
`docs/plans/refactoring-features-v3.md`, and
`docs/plans/2026-02-13-future-phases-roadmap.md`. It is scoped to implementing the currently
missing refactoring subcommands in `repo-scout`.

## Purpose / Big Picture

Phase 17 enables operators and coding agents to run end-to-end refactoring workflows directly in
`repo-scout` instead of manually composing low-level commands. After this phase, users can ask:
"what should I refactor?", "is this safe to change?", "what breaks if I move/rename/split this?",
and "did the refactor complete cleanly?".

User-visible outcomes:

1. New diagnosis commands: `anatomy`, `coupling`, `dead`, `test-gaps`.
2. New refactoring intelligence commands: `suggest`, `boundary`.
3. New pre-flight commands: `extract-check`, `move-check`, `rename-check`, `split-check`.
4. New support/verification commands: `test-scaffold`, `safe-steps`, `verify-refactor`, and
   `health --diff`.

Success is demonstrated by running each command against this repository and confirming deterministic
human output plus deterministic `--json` output across repeated runs.

## Contract Inputs

This plan is governed by:

1. `AGENTS.md` (repository root).
2. `agents/PLANS.md`.
3. `contracts/core/RISK_TIER_POLICY.md`.
4. `contracts/languages/RUST_CODING_CONTRACT.md`.
5. `templates/TASK_PACKET_TEMPLATE.md`.
6. `templates/TEST_PLAN_TEMPLATE.md`.
7. `templates/EVIDENCE_PACKET_TEMPLATE.md`.
8. `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.

Template mapping for this plan:

1. Task packet mapping: sections `Purpose / Big Picture`, `Milestones`, `Rollback Plan`,
   `Acceptance Criteria`.
2. Test plan mapping: each milestone includes explicit feature slices with Red -> Green -> Refactor
   commands.
3. Evidence packet mapping: each milestone defines required red/green/refactor transcript capture
   expectations and validator commands.

## AGENTS.md Constraints

Consulted path: `AGENTS.md`.

Effective constraints applied here:

1. Strict TDD is mandatory: no production code for a slice before a failing test exists.
2. Risk tier must be declared before implementation; if uncertain, choose higher.
3. Dogfooding is mandatory before and after each feature slice:
   `cargo run -- index --repo .`,
   `cargo run -- find <target_symbol> --repo . --json`,
   `cargo run -- refs <target_symbol> --repo . --json`,
   then post-slice non-JSON commands and `cargo test`.
4. Required validators before PR:
   `bash scripts/validate_tdd_cycle.sh --base origin/main` and
   `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
5. Integration-test style should follow existing milestone patterns in `tests/`.
6. In production code, do not introduce `unwrap()`/`expect()`/`panic!()` unless contract exception
   applies.

## Risk Tier and Required Controls

Tier: `2`.

Rationale: this phase changes core query behavior and CLI command surface across many code paths,
including verification logic and health comparisons that may drive user refactoring actions.

Tier 2 required controls applied:

1. Red -> Green -> Refactor evidence for every feature slice.
2. Task packet/test plan/evidence packet completeness in PR body.
3. Adversarial review checklist completion via
   `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.
4. Security and performance considerations documented in evidence.
5. Rollback plan defined and executable.

## Scope

In scope:

1. `src/cli.rs` command and argument wiring.
2. `src/main.rs` command handlers.
3. `src/query/diagnostics.rs` (new/expanded).
4. `src/query/planning.rs` (new).
5. `src/query/verification.rs` (new).
6. `src/output.rs` rendering for human and JSON output.
7. `src/query/mod.rs` exports and shared helpers.
8. `src/indexer/languages/*.rs` plus trait definitions for extraction analysis.
9. Integration tests under `tests/` for each new command slice.
10. Documentation updates in `docs/cli-reference.md`, `README.md`, and roadmap docs as needed.

Out of scope:

1. New persistence schema migrations beyond what already exists in schema v4.
2. IDE/LSP integration and watch mode.
3. Non-Rust language contract installation changes.

## Acceptance Criteria

1. All currently missing refactoring subcommands exist and are reachable from `repo-scout --help`.
2. Each new command has integration tests that fail before implementation and pass after.
3. `--json` output for each new command is deterministic and stable across repeated runs.
4. Dogfooding transcripts exist for each implemented command slice.
5. Full validation gates pass, including contract validators.

## Progress

- [x] (2026-02-13 00:00Z) Created Phase 17 ExecPlan with contract inputs, AGENTS constraints,
      risk-tier controls, milestone slices, and validation strategy.
- [x] (2026-02-13 00:38Z) Milestone 1 completed with integration tests:
      `anatomy`, `coupling`, `dead`, `test-gaps`.
- [x] (2026-02-13 00:43Z) Milestone 2 completed with integration tests:
      `suggest`, `boundary`.
- [x] (2026-02-13 00:49Z) Milestone 3 completed with integration tests:
      `move-check`, `rename-check`, `split-check`.
- [x] (2026-02-13 00:53Z) Milestone 4 completed with integration tests:
      `test-scaffold`, `safe-steps`, `verify-refactor`, `health --diff`.
- [x] (2026-02-13 00:59Z) Milestone 5 completed at Phase-17 baseline level with integration tests:
      `extract-check` range parsing, symbol bounds validation, and structured output.
- [x] (2026-02-13 01:06Z) `cargo fmt --all` and `cargo fmt --all -- --check` pass after
      formatting cleanup.
- [x] (2026-02-13 01:15Z) Full suite validation completed via `cargo test`.
- [ ] Phase 17 closure evidence/checklist packet updates in PR body remain to be finalized.

## Surprises & Discoveries

- Observation: command names documented in `docs/plans/refactoring-features-v3.md` are still
  unrecognized by the live CLI.
  Evidence: local command probes return `error: unrecognized subcommand` for all missing commands.

- Observation: current command set already provides reusable building blocks (`refs`, `impact`,
  `tests-for`, `circular`, `related`, `call-path`) that can compose into pre-flight checks with low
  initial risk.
  Evidence: dogfooding outputs from `deps`, `callers`, `callees`, and `verify-plan`.

- Observation: `cargo fmt --check` initially failed due broad formatting drift after feature edits,
  but repo-wide `cargo fmt --all` cleanly resolved the drift.
  Evidence: `cargo fmt --all -- --check` now exits cleanly.

## Decision Log

- Decision: execute Phase 17 in vertical slices by command family, not by broad module rewrites.
  Rationale: this keeps each slice testable, reversible, and aligned with strict TDD.
  Date/Author: 2026-02-13 / Codex

- Decision: classify work as Tier 2 despite no planned schema migration.
  Rationale: blast radius is high due to central query and planning behavior changes.
  Date/Author: 2026-02-13 / Codex

- Decision: implement `extract-check` last with staged language rollout (Rust first, then Go/Python/
  TypeScript) and explicit graceful degradation.
  Rationale: extraction-flow analysis is the most technically complex and highest uncertainty item.
  Date/Author: 2026-02-13 / Codex

## Milestones

## Milestone 0: Command Surface Contract Lock

This milestone freezes the command interface before feature behavior, so downstream work does not
churn CLI contracts.

Feature slice 0.1: add CLI variants and args for every missing command.

Red:

    cargo test --test milestone96_refactoring_cli_surface -- --nocapture

Green:

    Add command variants/args in src/cli.rs and handler stubs in src/main.rs.
    cargo test --test milestone96_refactoring_cli_surface -- --nocapture

Refactor:

    cargo test --test milestone96_refactoring_cli_surface -- --nocapture
    cargo test

Dogfood for symbol `run_verify_refactor`:

    cargo run -- index --repo .
    cargo run -- find run_verify_refactor --repo . --json
    cargo run -- refs run_verify_refactor --repo . --json

## Milestone 1: Diagnosis Commands

Implement `anatomy`, `coupling`, `dead`, and `test-gaps` using `src/query/diagnostics.rs` and new
output formatters.

Feature slice 1.1: `anatomy <file>`.

Red:

    cargo test --test milestone97_anatomy -- --nocapture

Green:

    Implement query + output + handler wiring for anatomy.
    cargo test --test milestone97_anatomy -- --nocapture

Refactor:

    cargo test --test milestone97_anatomy -- --nocapture
    cargo test

Feature slice 1.2: `coupling`.

Red:

    cargo test --test milestone98_coupling -- --nocapture

Green:

    Implement coupling scoring and deterministic sorting.
    cargo test --test milestone98_coupling -- --nocapture

Refactor:

    cargo test --test milestone98_coupling -- --nocapture
    cargo test

Feature slice 1.3: `dead`.

Red:

    cargo test --test milestone99_dead -- --nocapture

Green:

    Implement dead symbol detection scoped to production by default.
    cargo test --test milestone99_dead -- --nocapture

Refactor:

    cargo test --test milestone99_dead -- --nocapture
    cargo test

Feature slice 1.4: `test-gaps`.

Red:

    cargo test --test milestone100_test_gaps -- --nocapture

Green:

    Implement coverage gap analysis and risk tiers.
    cargo test --test milestone100_test_gaps -- --nocapture

Refactor:

    cargo test --test milestone100_test_gaps -- --nocapture
    cargo test

Dogfood for symbols `file_anatomy`, `test_gap_analysis`:

    cargo run -- index --repo .
    cargo run -- find file_anatomy --repo . --json
    cargo run -- refs file_anatomy --repo . --json
    cargo run -- anatomy src/main.rs --repo .
    cargo run -- test-gaps src/main.rs --repo .
    cargo test

## Milestone 2: Refactoring Intelligence Commands

Implement `suggest` and `boundary`.

Feature slice 2.1: `boundary <file>`.

Red:

    cargo test --test milestone101_boundary -- --nocapture

Green:

    Implement visibility/public-surface analysis with external-reference counts.
    cargo test --test milestone101_boundary -- --nocapture

Refactor:

    cargo test --test milestone101_boundary -- --nocapture
    cargo test

Feature slice 2.2: `suggest`.

Red:

    cargo test --test milestone102_suggest -- --nocapture

Green:

    Implement initial weighted ranking with transparent signal attribution.
    cargo test --test milestone102_suggest -- --nocapture

Refactor:

    cargo test --test milestone102_suggest -- --nocapture
    cargo test

Dogfood for symbols `boundary_analysis`, `suggest_refactorings`:

    cargo run -- index --repo .
    cargo run -- find suggest_refactorings --repo . --json
    cargo run -- refs suggest_refactorings --repo . --json
    cargo run -- boundary src/main.rs --repo .
    cargo run -- suggest --repo . --top 10
    cargo test

## Milestone 3: Pre-flight Commands

Implement `move-check`, `rename-check`, and `split-check` by composing existing graph/test
intelligence before adding heuristics.

Feature slice 3.1: `move-check`.

Red:

    cargo test --test milestone103_move_check -- --nocapture

Green:

    Implement move impact report with callsite and dependency checklist.
    cargo test --test milestone103_move_check -- --nocapture

Refactor:

    cargo test --test milestone103_move_check -- --nocapture
    cargo test

Feature slice 3.2: `rename-check`.

Red:

    cargo test --test milestone104_rename_check -- --nocapture

Green:

    Implement rename impact report (AST refs + text occurrences + derived names).
    cargo test --test milestone104_rename_check -- --nocapture

Refactor:

    cargo test --test milestone104_rename_check -- --nocapture
    cargo test

Feature slice 3.3: `split-check`.

Red:

    cargo test --test milestone105_split_check -- --nocapture

Green:

    Implement split grouping analysis and cross-group dependency report.
    cargo test --test milestone105_split_check -- --nocapture

Refactor:

    cargo test --test milestone105_split_check -- --nocapture
    cargo test

Dogfood for symbols `move_check`, `rename_check`, `split_check`:

    cargo run -- index --repo .
    cargo run -- find split_check --repo . --json
    cargo run -- refs split_check --repo . --json
    cargo run -- move-check run_find --to src/query/mod.rs --repo .
    cargo run -- rename-check run_find --to run_find_v2 --repo .
    cargo run -- split-check src/main.rs --auto --repo .
    cargo test

## Milestone 4: Support and Verification Commands

Implement `test-scaffold`, `safe-steps`, `verify-refactor`, and `health --diff`.

Feature slice 4.1: `test-scaffold`.

Red:

    cargo test --test milestone106_test_scaffold -- --nocapture

Green:

    Implement structured test setup output with deterministic ordering.
    cargo test --test milestone106_test_scaffold -- --nocapture

Refactor:

    cargo test --test milestone106_test_scaffold -- --nocapture
    cargo test

Feature slice 4.2: `safe-steps`.

Red:

    cargo test --test milestone107_safe_steps -- --nocapture

Green:

    Implement action-specific safe step generation for extract/move/rename/split.
    cargo test --test milestone107_safe_steps -- --nocapture

Refactor:

    cargo test --test milestone107_safe_steps -- --nocapture
    cargo test

Feature slice 4.3: `verify-refactor`.

Red:

    cargo test --test milestone108_verify_refactor -- --nocapture

Green:

    Implement before/after comparison and warning/error policy (`--strict`).
    cargo test --test milestone108_verify_refactor -- --nocapture

Refactor:

    cargo test --test milestone108_verify_refactor -- --nocapture
    cargo test

Feature slice 4.4: `health --diff`.

Red:

    cargo test --test milestone109_health_diff -- --nocapture

Green:

    Implement baseline save/load and diff renderer with stable keys.
    cargo test --test milestone109_health_diff -- --nocapture

Refactor:

    cargo test --test milestone109_health_diff -- --nocapture
    cargo test

Dogfood for symbols `verify_refactor`, `health_report`:

    cargo run -- index --repo .
    cargo run -- find verify_refactor --repo . --json
    cargo run -- refs verify_refactor --repo . --json
    cargo run -- health --repo . --save-baseline
    cargo run -- health --repo . --diff
    cargo run -- verify-refactor --repo . --before HEAD~1 --after HEAD
    cargo test

## Milestone 5: Extract Analysis Command (`extract-check`)

Implement `extract-check` with bounded, explicit extraction analysis.

Feature slice 5.1: Rust extraction analysis path.

Red:

    cargo test --test milestone110_extract_check_rust -- --nocapture

Green:

    Add Rust adapter extraction analysis and command integration.
    cargo test --test milestone110_extract_check_rust -- --nocapture

Refactor:

    cargo test --test milestone110_extract_check_rust -- --nocapture
    cargo test

Feature slice 5.2: Go/Python/TypeScript support and graceful fallback.

Red:

    cargo test --test milestone111_extract_check_cross_language -- --nocapture

Green:

    Add remaining adapters or explicit "analysis unavailable" with typed reason.
    cargo test --test milestone111_extract_check_cross_language -- --nocapture

Refactor:

    cargo test --test milestone111_extract_check_cross_language -- --nocapture
    cargo test

Dogfood for symbol `analyze_extraction`:

    cargo run -- index --repo .
    cargo run -- find analyze_extraction --repo . --json
    cargo run -- refs analyze_extraction --repo . --json
    cargo run -- extract-check run_find --lines 105-155 --repo .
    cargo test

## Validation Commands (per completed slice and at phase closure)

Run from repository root:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::undocumented_unsafe_blocks
    cargo test --workspace --all-features
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

Tier 2 adversarial review:

    Complete checklist: checklists/ADVERSARIAL_REVIEW_CHECKLIST.md

## Rollback Plan

Rollback triggers:

1. Determinism regressions in new `--json` commands.
2. Incorrect safety recommendations that fail fixture-backed expectations.
3. Any clippy/test/validator regressions not quickly resolvable.

Rollback mechanism:

1. Revert the latest failing feature slice commit(s) only.
2. Re-run milestone test file plus `cargo test`.
3. Re-run contract validators.
4. Re-open the slice with a narrower Red test scope and re-implement.

## Outcomes & Retrospective

Pending completion. At phase close, summarize:

1. Which commands shipped with full behavior.
2. Which commands degraded gracefully and why.
3. Remaining technical debt and next-phase recommendations.
4. Lessons from dogfooding command quality in real refactor sessions.
