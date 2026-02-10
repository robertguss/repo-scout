# Build `repo-scout` Phase 16 High-Bar / GA Hardening

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase15-execplan.md` and
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is scoped to release-hardening work
after cross-language production convergence.

## Purpose / Big Picture

Phase 16 proves release stability with reusable high-bar gates, so operators can replay the same
critical commands and trust deterministic outputs across Rust, Go, Python, and TypeScript fixture
repositories.

User-visible outcomes for the first six Phase 16 slices: one command
(`just phase16-deterministic-replay` or `bash scripts/check_phase16_deterministic_replay.sh`) runs
deterministic replay checks for `find`, `refs`, `tests-for`, `verify-plan`, and `diff-impact`
across the integrated cross-language fixture pack, and one command
(`just phase16-benchmark-pack` or `bash scripts/check_phase16_benchmark_pack.sh`) enforces
cross-language timing budgets for the same command set, and one command
(`just phase16-known-issues-budget` or `bash scripts/check_phase16_known_issues_budget.sh`)
enforces known-issues budget/ownership thresholds, and one command
(`just phase16-large-repo-benchmark` or `bash scripts/check_phase16_large_repo_benchmark.sh`)
enforces repository-scale benchmark guardrails, and one command
(`just phase16-release-checklist` or `bash scripts/check_phase16_release_checklist.sh`) enforces
release closure gates, and one command
(`just phase16-large-repo-replay` or `bash scripts/check_phase16_large_repo_replay.sh`) enforces
repository-scale deterministic replay checks for critical commands.

## Progress

- [x] (2026-02-10 02:28Z) Re-read `AGENTS.md`, `agents/PLANS.md`, and
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md` to confirm the next roadmap phase
      is Phase 16.
- [x] (2026-02-10 02:29Z) Declared risk tier `1` for the first Phase 16 slice and ran required
      pre-slice dogfooding commands for symbol `verify_plan_for_changed_files`.
- [x] (2026-02-10 02:30Z) Added failing integration tests in
      `tests/milestone64_phase16_ga_replay.rs` for Phase 16 execplan presence, deterministic replay
      script + Just wiring, and perf-baseline documentation coverage.
- [x] (2026-02-10 02:30Z) Observed strict Red via
      `cargo test --test milestone64_phase16_ga_replay -- --nocapture` (3 failures).
- [x] (2026-02-10 02:33Z) Implemented deterministic replay gate assets:
      `scripts/check_phase16_deterministic_replay.sh`, `Justfile` target
      `phase16-deterministic-replay`, `docs/performance-baseline.md`, and
      `agents/plans/repo-scout-phase16-execplan.md`.
- [x] (2026-02-10 02:35Z) Observed Green and gate viability via
      `cargo test --test milestone64_phase16_ga_replay -- --nocapture`,
      `bash scripts/check_phase16_deterministic_replay.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`,
      and `just phase16-deterministic-replay .`.
- [x] (2026-02-10 02:36Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find verify_plan_for_changed_files --repo .`,
      `cargo run -- refs verify_plan_for_changed_files --repo .`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:37Z) Declared risk tier `1` for Phase 16 Milestone 65 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 02:38Z) Added failing integration tests in
      `tests/milestone65_phase16_benchmark_pack.rs` for benchmark thresholds doc, benchmark gate
      script + Just wiring, and perf-baseline documentation coverage.
- [x] (2026-02-10 02:38Z) Observed strict Red via
      `cargo test --test milestone65_phase16_benchmark_pack -- --nocapture` (3 failures).
- [x] (2026-02-10 02:39Z) Implemented benchmark-pack gate assets:
      `scripts/check_phase16_benchmark_pack.sh`, `docs/performance-thresholds-phase16.md`,
      `Justfile` target `phase16-benchmark-pack`, and docs updates.
- [x] (2026-02-10 02:39Z) Observed Green and gate viability via
      `cargo test --test milestone65_phase16_benchmark_pack -- --nocapture`,
      `bash scripts/check_phase16_benchmark_pack.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`,
      and `just phase16-benchmark-pack .`.
- [x] (2026-02-10 02:39Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`, `cargo test`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:40Z) Declared risk tier `1` for Phase 16 Milestone 66 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 02:40Z) Added failing integration tests in
      `tests/milestone66_phase16_known_issues_budget.rs` for known-issues budget artifact,
      budget-gate script + Just wiring, and roadmap visibility.
- [x] (2026-02-10 02:40Z) Observed strict Red via
      `cargo test --test milestone66_phase16_known_issues_budget -- --nocapture` (3 failures).
- [x] (2026-02-10 02:43Z) Implemented known-issues budget assets:
      `docs/known-issues-budget-phase16.md`,
      `scripts/check_phase16_known_issues_budget.sh`,
      `Justfile` target `phase16-known-issues-budget`, and roadmap/docs updates.
- [x] (2026-02-10 02:43Z) Observed Green and gate viability via
      `cargo test --test milestone66_phase16_known_issues_budget -- --nocapture`,
      `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`,
      and `just phase16-known-issues-budget .`.
- [x] (2026-02-10 02:43Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`, `cargo test`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:44Z) Declared risk tier `1` for Phase 16 Milestone 67 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 02:45Z) Added failing integration tests in
      `tests/milestone67_phase16_large_repo_benchmark.rs` for large-repo thresholds doc,
      benchmark script + Just wiring, perf-baseline coverage, and roadmap visibility.
- [x] (2026-02-10 02:45Z) Observed strict Red via
      `cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture` (4 failures).
- [x] (2026-02-10 02:47Z) Implemented large-repo benchmark assets:
      `docs/performance-thresholds-phase16-large-repo.md`,
      `scripts/check_phase16_large_repo_benchmark.sh`,
      `Justfile` target `phase16-large-repo-benchmark`, and roadmap/docs updates.
- [x] (2026-02-10 02:47Z) Observed Green and gate viability via
      `cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture`,
      `bash scripts/check_phase16_large_repo_benchmark.sh --repo .`,
      and `just phase16-large-repo-benchmark .`.
- [x] (2026-02-10 02:47Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`, `cargo test`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 02:48Z) Declared risk tier `1` for Phase 16 Milestone 68 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 02:48Z) Added failing integration tests in
      `tests/milestone68_phase16_release_checklist_gate.rs` for release-checklist artifact,
      script + Just wiring, and roadmap visibility.
- [x] (2026-02-10 02:48Z) Observed strict Red via
      `cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture` (3 failures).
- [x] (2026-02-10 02:50Z) Implemented release-checklist gate assets:
      `docs/release-checklist-phase16.md`,
      `scripts/check_phase16_release_checklist.sh`,
      `Justfile` target `phase16-release-checklist`, and roadmap/docs updates.
- [x] (2026-02-10 02:50Z) Observed Green and gate viability via
      `cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture`,
      `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`,
      and `just phase16-release-checklist .`.
- [x] (2026-02-10 03:02Z) Declared risk tier `1` for Phase 16 Milestone 69 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 03:03Z) Added failing integration tests in
      `tests/milestone69_phase16_large_repo_replay.rs` for large-repo replay script + Just wiring,
      perf-baseline visibility, and roadmap visibility.
- [x] (2026-02-10 03:03Z) Observed strict Red via
      `cargo test --test milestone69_phase16_large_repo_replay -- --nocapture` (3 failures).
- [x] (2026-02-10 03:05Z) Implemented large-repo replay assets:
      `scripts/check_phase16_large_repo_replay.sh`,
      `Justfile` target `phase16-large-repo-replay`, and roadmap/docs updates.
- [x] (2026-02-10 03:05Z) Observed Green and gate viability via
      `cargo test --test milestone69_phase16_large_repo_replay -- --nocapture`,
      `bash scripts/check_phase16_large_repo_replay.sh --repo .`,
      and `just phase16-large-repo-replay .`.
- [x] (2026-02-10 03:11Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`, `cargo test`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-10 03:13Z) Declared risk tier `1` for Phase 16 Milestone 70 and ran required
      pre-slice dogfooding commands for symbol `select_full_suite_command`.
- [x] (2026-02-10 03:14Z) Added failing integration tests in
      `tests/milestone70_phase16_known_issues_closure.rs` for zero-deferred budget enforcement,
      PH16-003 closure evidence, and release-checklist closure evidence.
- [x] (2026-02-10 03:14Z) Observed strict Red via
      `cargo test --test milestone70_phase16_known_issues_closure -- --nocapture` (3 failures).
- [x] (2026-02-10 03:15Z) Implemented known-issues closure updates:
      `docs/known-issues-budget-phase16.md` (`max_deferred: 0`, PH16-003 `closed`) and
      `docs/release-checklist-phase16.md` (known-issues deferred=0 evidence).
- [x] (2026-02-10 03:15Z) Observed Green and gate viability via
      `cargo test --test milestone70_phase16_known_issues_closure -- --nocapture`,
      `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`,
      and `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`.
- [x] (2026-02-10 03:21Z) Completed post-slice dogfood and full validation gates:
      `cargo run -- index --repo .`, `cargo run -- find select_full_suite_command --repo .`,
      `cargo run -- refs select_full_suite_command --repo .`, `cargo test`, `cargo fmt`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`,
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.

## Surprises & Discoveries

- Observation: no Phase 16 ExecPlan file existed yet even though the roadmap names
  `agents/plans/repo-scout-phase16-execplan.md` as the next artifact.
  Evidence: Red failure in
  `milestone64_phase16_execplan_exists_and_is_contract_scoped` with missing-file error.

- Observation: the repository already had a cross-language convergence fixture pack, so the first
  GA deterministic replay gate could be added without introducing a new fixture corpus.
  Evidence: Phase 15 pack available under `tests/fixtures/phase15/convergence_pack`.
- Observation: milestone65 Red failed immediately with missing script and threshold documentation,
  confirming the benchmark gate assets were not present before this slice.
  Evidence: missing-file failures for
  `scripts/check_phase16_benchmark_pack.sh` and `docs/performance-thresholds-phase16.md`.
- Observation: known-issues budget gating needed explicit machine-checkable thresholds and an
  unambiguous row format to avoid drift between human triage and script enforcement.
  Evidence: milestone66 Red failed on missing budget artifact/script and was resolved by adding
  explicit `max_open`/`max_deferred`/`max_unowned` keys plus `| PH16-... |` rows.
- Observation: repository-scale benchmark checks needed doc-parsed threshold keys to avoid script
  drift as Phase 16 budgets evolve.
  Evidence: milestone67 Red failed on missing threshold doc/script and was resolved by adding
  `max_*_seconds` keys in `docs/performance-thresholds-phase16-large-repo.md` parsed by the
  benchmark gate script.
- Observation: release checklist closure gating needed explicit pass/fail status keys rather than
  prose-only completion notes to keep release readiness script-checkable.
  Evidence: milestone68 Red failed on missing checklist/script and was resolved by adding
  `quality_gate`/`evidence_gate`/`rollback_plan`/`docs_gate`/`ci_gate` keys in
  `docs/release-checklist-phase16.md` parsed by `check_phase16_release_checklist.sh`.
- Observation: fixture-pack deterministic replay alone did not cover repository-scale command
  determinism for `context` and workspace-wide query paths.
  Evidence: milestone69 introduced a dedicated repository-scale replay gate script with
  deterministic checks for `find`/`refs`/`tests-for`/`verify-plan`/`diff-impact`/`context`.
- Observation: Phase 16 known-issues closure posture remained permissive (`max_deferred: 3`) even
  after all hardening gates were implemented.
  Evidence: milestone70 Red failed on missing zero-deferred budget enforcement and unresolved
  PH16-003 closure evidence, then passed after tightening to `max_deferred: 0` and closing PH16-003.

## Decision Log

- Decision: classify this Phase 16 slice as risk tier `1`.
  Rationale: initial GA hardening changes are additive script/plan/docs gates, with no schema,
  migration, or query-engine behavioral edits.
  Date/Author: 2026-02-10 / Codex

- Decision: build deterministic replay checks on top of the existing Phase 15 integrated fixture
  pack instead of creating new Phase 16 fixtures first.
  Rationale: this keeps Phase 16 startup low risk and immediately validates stability for already
  production-ready cross-language command contracts.
  Date/Author: 2026-02-10 / Codex

- Decision: implement the benchmark gate over the existing Phase 15 convergence fixture pack plus
  workspace-level index/find/refs timing checks.
  Rationale: this provides immediate cross-language timing coverage with low maintenance overhead
  while preserving one shared fixture corpus for Phase 16 hardening scripts.
  Date/Author: 2026-02-10 / Codex

- Decision: implement known-issues budget enforcement as a markdown artifact +
  lightweight shell validator (`docs/known-issues-budget-phase16.md` +
  `scripts/check_phase16_known_issues_budget.sh`) rather than introducing new schema/tooling.
  Rationale: this keeps triage ownership visible in-doc while still enabling deterministic local/CI
  gating with minimal operational overhead.
  Date/Author: 2026-02-10 / Codex

- Decision: implement larger-repo benchmark gating against the repository-scale workflow path
  (`--repo .`) with doc-defined thresholds and release-mode timing checks.
  Rationale: this closes the “beyond fixture-pack” hardening gap while keeping the gate repeatable
  and low-maintenance for contributors.
  Date/Author: 2026-02-10 / Codex
- Decision: implement release-checklist closure enforcement as a markdown artifact +
  lightweight shell validator (`docs/release-checklist-phase16.md` +
  `scripts/check_phase16_release_checklist.sh`) instead of adding new schema or storage.
  Rationale: this keeps release readiness transparent in-document while adding deterministic local
  and CI gating with minimal operational overhead.
  Date/Author: 2026-02-10 / Codex
- Decision: add a dedicated repository-scale deterministic replay gate
  (`scripts/check_phase16_large_repo_replay.sh`) rather than overloading the fixture-pack replay
  script with workspace-specific behavior.
  Rationale: this keeps fixture-pack and repository-scale scopes explicit and minimizes accidental
  coupling between language fixtures and real-world repo workflows.
  Date/Author: 2026-02-10 / Codex
- Decision: close PH16-003 and tighten the known-issues budget to `max_deferred: 0` for Phase 16
  closure.
  Rationale: all planned Phase 16 hardening gates now exist and pass, so deferred issue posture is
  no longer needed and would weaken closure guarantees.
  Date/Author: 2026-02-10 / Codex

## Outcomes & Retrospective

Phase 16 closure is complete. This plan iteration implements six executable GA hardening gates:
cross-language deterministic replay checks, benchmark-pack timing guardrails, known-issues budget
enforcement, large-repo benchmark guardrails, release-checklist closure gating, and large-repo
deterministic replay checks.

Implemented outcomes for these slices:

- `scripts/check_phase16_deterministic_replay.sh` validates repeated JSON output equality for
  `find`, `refs`, `tests-for`, `verify-plan`, and `diff-impact`.
- `just phase16-deterministic-replay` exposes the replay gate for routine local and CI usage.
- `scripts/check_phase16_benchmark_pack.sh` enforces threshold budgets for `index`, `find`,
  `refs`, `tests-for`, `verify-plan`, and `diff-impact` across workspace + cross-language fixtures.
- `just phase16-benchmark-pack` exposes the benchmark-pack timing gate for local and CI usage.
- `docs/performance-thresholds-phase16.md` defines conservative benchmark budgets for the Phase 16
  guardrail script.
- `docs/performance-baseline.md` documents both Phase 16 replay and benchmark commands.
- `scripts/check_phase16_known_issues_budget.sh` enforces `max_open`, `max_deferred`, and
  `max_unowned` thresholds for Phase 16 issue triage.
- `just phase16-known-issues-budget` exposes the known-issues budget gate for routine local and CI
  usage.
- `docs/known-issues-budget-phase16.md` now enforces `max_deferred: 0` and records PH16-003 as
  closed with repository-scale hardening evidence.
- `scripts/check_phase16_large_repo_benchmark.sh` enforces repository-scale benchmark thresholds
  for `index`, `find`, `refs`, `context`, `verify-plan`, and `diff-impact`.
- `just phase16-large-repo-benchmark` exposes the large-repo benchmark gate for routine local and
  CI usage.
- `docs/performance-thresholds-phase16-large-repo.md` defines repository-scale benchmark budgets.
- `scripts/check_phase16_release_checklist.sh` enforces quality/evidence/rollback/docs/CI release
  status gates.
- `just phase16-release-checklist` exposes the release checklist gate for routine local and CI
  usage.
- `docs/release-checklist-phase16.md` defines explicit release checklist gate statuses and
  evidence links.
- `scripts/check_phase16_large_repo_replay.sh` validates repeated JSON output equality for
  repository-scale `find`, `refs`, `tests-for`, `verify-plan`, `diff-impact`, and `context`.
- `just phase16-large-repo-replay` exposes the repository-scale replay gate for routine local and
  CI usage.

Remaining Phase 16 roadmap work after these slices:

- maintenance/backlog only; no known core capability gaps remain for High-Bar/GA closure.

## Context and Orientation

This Phase 16 slice touches:

- `scripts/check_phase16_deterministic_replay.sh` (new reusable replay gate),
- `scripts/check_phase16_benchmark_pack.sh` (new benchmark-pack timing gate),
- `scripts/check_phase16_known_issues_budget.sh` (known-issues budget gate),
- `scripts/check_phase16_large_repo_benchmark.sh` (large-repo benchmark gate),
- `scripts/check_phase16_release_checklist.sh` (release-checklist gate),
- `scripts/check_phase16_large_repo_replay.sh` (large-repo deterministic replay gate),
- `Justfile` (new operator entrypoint),
- `tests/milestone64_phase16_ga_replay.rs` (integration-level Red/Green coverage),
- `tests/milestone65_phase16_benchmark_pack.rs` (integration-level Red/Green coverage),
- `tests/milestone66_phase16_known_issues_budget.rs` (integration-level Red/Green coverage),
- `tests/milestone67_phase16_large_repo_benchmark.rs` (integration-level Red/Green coverage),
- `tests/milestone68_phase16_release_checklist_gate.rs` (integration-level Red/Green coverage),
- `tests/milestone69_phase16_large_repo_replay.rs` (integration-level Red/Green coverage),
- `tests/milestone70_phase16_known_issues_closure.rs` (integration-level Red/Green coverage),
- `docs/performance-baseline.md` (command contract docs),
- `docs/performance-thresholds-phase16.md` (Phase 16 benchmark budgets),
- `docs/performance-thresholds-phase16-large-repo.md` (Phase 16 repository-scale benchmark budgets),
- `docs/release-checklist-phase16.md` (Phase 16 release checklist status gates),
- `docs/known-issues-budget-phase16.md` (Phase 16 triage/ownership budgets),
- `agents/plans/repo-scout-roadmap-to-production-and-ga.md` (phase status updates),
- `docs/dogfood-log.md` and `README.md` (operator guidance refresh).

## Contract Inputs

Phase 16 execution consumes and aligns with:

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
- risk tier declaration before production edits,
- pre/post dogfood command loops for the same target symbol,
- integration-style tests under `tests/` with milestone naming,
- no `unwrap()`/`expect()`/`panic!` additions in `src/`,
- contract validator commands run before closure reporting.

Dogfood commands used before implementation:

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo . --json
    cargo run -- refs select_full_suite_command --repo . --json

Dogfood commands used after implementation:

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo .
    cargo run -- refs verify_plan_for_changed_files --repo .
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    cargo run -- index --repo .
    cargo run -- find select_full_suite_command --repo .
    cargo run -- refs select_full_suite_command --repo .
    cargo test

## Risk Tier and Required Controls

Phase 16 slice risk tier: `1` (moderate).

Rationale: additive release-hardening scripts/docs/tests only. No schema, persistence, or core
query algorithm changes.

Tier 1 controls applied:

- strict per-slice Red/Green/Refactor evidence,
- deterministic integration checks for gate wiring,
- additive minimal-scope implementation,
- full-suite regression validation and contract validators.

Escalation rule: if a later Phase 16 slice changes schema/storage or core ranking/query behavior,
pause and escalate to Tier 2 controls before implementation.

## Strict TDD Contract

No production edits for this feature slice are allowed until failing automated tests exist for the
exact slice.

Phase 16 slice 64 feature slices:

- 64A: require explicit Phase 16 ExecPlan artifact in-repo with Tiger-era mandatory sections.
- 64B: require deterministic replay gate script and `Justfile` wiring.
- 64C: require docs/performance command-contract entry for the replay gate.

Phase 16 slice 65 feature slices:

- 65A: require explicit benchmark threshold documentation for Phase 16 guardrails.
- 65B: require benchmark-pack script and `Justfile` wiring with threshold enforcement and
  `--record` mode.
- 65C: require perf-baseline command-contract entry for the benchmark-pack gate.

Phase 16 slice 66 feature slices:

- 66A: require explicit known-issues budget artifact with ownership and triage decisions.
- 66B: require known-issues budget script and `Justfile` wiring with threshold enforcement and
  `--record` mode.
- 66C: require roadmap visibility for the known-issues budget gate.

Phase 16 slice 67 feature slices:

- 67A: require explicit repository-scale benchmark threshold documentation.
- 67B: require large-repo benchmark script and `Justfile` wiring with threshold enforcement and
  `--record` mode.
- 67C: require perf-baseline and roadmap visibility for the large-repo benchmark gate.

Phase 16 slice 68 feature slices:

- 68A: require explicit release-checklist closure artifact with quality/evidence/rollback/docs/CI
  gate statuses.
- 68B: require release-checklist script and `Justfile` wiring with pass/fail gate enforcement and
  `--record` mode.
- 68C: require roadmap visibility for the release-checklist gate.

Phase 16 slice 69 feature slices:

- 69A: require repository-scale deterministic replay script for real-world command scenarios.
- 69B: require `Justfile` wiring for repository-scale deterministic replay.
- 69C: require perf-baseline and roadmap visibility for the repository-scale replay gate.

Phase 16 slice 70 feature slices:

- 70A: require known-issues budget closure posture with `max_deferred: 0`.
- 70B: require closure evidence that PH16-003 is no longer deferred.
- 70C: require release-checklist evidence to capture known-issues deferred=0 closure state.

## TDD Evidence Log

- Red:
  - `cargo test --test milestone64_phase16_ga_replay -- --nocapture`
    failed with missing Phase 16 execplan file, missing replay script, and missing perf-doc entry.
- Green:
  - `cargo test --test milestone64_phase16_ga_replay -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_deterministic_replay.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`
    passed.
  - `just phase16-deterministic-replay .` passed.
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `cargo test` full suite passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.
- Red:
  - `cargo test --test milestone70_phase16_known_issues_closure -- --nocapture`
    failed with permissive deferred budget, unresolved PH16-003 deferred state, and missing
    release-checklist deferred=0 evidence.
- Green:
  - `cargo test --test milestone70_phase16_known_issues_closure -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`
    passed.
  - `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`
    passed.
  - `cargo run -- index --repo .` passed.
  - `cargo run -- find select_full_suite_command --repo .` passed.
  - `cargo run -- refs select_full_suite_command --repo .` passed.
  - `cargo test` full suite passed.
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.
- Red:
  - `cargo test --test milestone69_phase16_large_repo_replay -- --nocapture`
    failed with missing large-repo replay script and missing perf-baseline/roadmap references.
- Green:
  - `cargo test --test milestone69_phase16_large_repo_replay -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_large_repo_replay.sh --repo .` passed.
  - `just phase16-large-repo-replay .` passed.
- Red:
  - `cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture`
    failed with missing release checklist doc/script and missing roadmap gate reference.
- Green:
  - `cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`
    passed.
  - `just phase16-release-checklist .` passed.
- Red:
  - `cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture`
    failed with missing large-repo benchmark script, missing threshold doc, and missing
    perf-baseline/roadmap references.
- Green:
  - `cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_large_repo_benchmark.sh --repo .` passed.
  - `just phase16-large-repo-benchmark .` passed.
  - `cargo run -- index --repo .` passed.
  - `cargo run -- find select_full_suite_command --repo .` passed.
  - `cargo run -- refs select_full_suite_command --repo .` passed.
  - `cargo test` full suite passed.
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.
- Red:
  - `cargo test --test milestone66_phase16_known_issues_budget -- --nocapture`
    failed with missing known-issues budget artifact, missing budget gate script, and missing
    roadmap gate reference.
- Green:
  - `cargo test --test milestone66_phase16_known_issues_budget -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`
    passed.
  - `just phase16-known-issues-budget .` passed.
  - `cargo run -- index --repo .` passed.
  - `cargo run -- find select_full_suite_command --repo .` passed.
  - `cargo run -- refs select_full_suite_command --repo .` passed.
  - `cargo test` full suite passed.
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.
- Red:
  - `cargo test --test milestone65_phase16_benchmark_pack -- --nocapture`
    failed with missing benchmark-pack script, missing threshold doc, and missing perf-baseline
    entry.
- Green:
  - `cargo test --test milestone65_phase16_benchmark_pack -- --nocapture` passed.
- Refactor/non-regression:
  - `bash scripts/check_phase16_benchmark_pack.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`
    passed.
  - `just phase16-benchmark-pack .` passed.
  - `cargo run -- index --repo .` passed.
  - `cargo run -- find select_full_suite_command --repo .` passed.
  - `cargo run -- refs select_full_suite_command --repo .` passed.
  - `cargo test` full suite passed.
  - `cargo fmt` passed.
  - `cargo clippy --all-targets --all-features -- -D warnings` passed.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` passed.
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` passed.

## Plan of Work

1. Keep deterministic replay and benchmark-pack scripts runnable and synchronized with fixture-pack
   command contracts.
2. Keep release-checklist and repository-scale deterministic replay gates runnable and documented.
3. Preserve strict per-slice Red/Green/Refactor evidence and post-slice dogfooding.

## Concrete Steps

Run from repository root:

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json
    cargo test --test milestone64_phase16_ga_replay -- --nocapture
    cargo test --test milestone64_phase16_ga_replay -- --nocapture
    bash scripts/check_phase16_deterministic_replay.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack
    cargo test --test milestone65_phase16_benchmark_pack -- --nocapture
    cargo test --test milestone65_phase16_benchmark_pack -- --nocapture
    bash scripts/check_phase16_benchmark_pack.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack
    just phase16-benchmark-pack .
    cargo test --test milestone66_phase16_known_issues_budget -- --nocapture
    cargo test --test milestone66_phase16_known_issues_budget -- --nocapture
    bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md
    just phase16-known-issues-budget .
    cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture
    cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture
    bash scripts/check_phase16_large_repo_benchmark.sh --repo .
    just phase16-large-repo-benchmark .
    cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture
    cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture
    bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md
    just phase16-release-checklist .
    cargo test --test milestone69_phase16_large_repo_replay -- --nocapture
    cargo test --test milestone69_phase16_large_repo_replay -- --nocapture
    bash scripts/check_phase16_large_repo_replay.sh --repo .
    just phase16-large-repo-replay .
    cargo test --test milestone70_phase16_known_issues_closure -- --nocapture
    cargo test --test milestone70_phase16_known_issues_closure -- --nocapture
    bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md
    bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings
    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo .
    cargo run -- refs verify_plan_for_changed_files --repo .
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## Validation and Acceptance

Acceptance criteria for this slice:

- `tests/milestone64_phase16_ga_replay.rs` passes and asserts script/Just/docs/plan wiring.
- `scripts/check_phase16_deterministic_replay.sh` passes against the convergence fixture pack.
- `just phase16-deterministic-replay` executes the same gate successfully.
- `tests/milestone65_phase16_benchmark_pack.rs` passes and asserts benchmark script/Just/docs
  wiring.
- `scripts/check_phase16_benchmark_pack.sh` passes against the convergence fixture pack.
- `just phase16-benchmark-pack` executes the same gate successfully.
- `tests/milestone66_phase16_known_issues_budget.rs` passes and asserts known-issues
  script/Just/docs/roadmap wiring.
- `scripts/check_phase16_known_issues_budget.sh` passes against
  `docs/known-issues-budget-phase16.md`.
- `just phase16-known-issues-budget` executes the same gate successfully.
- `tests/milestone67_phase16_large_repo_benchmark.rs` passes and asserts large-repo benchmark
  script/Just/docs/roadmap wiring.
- `scripts/check_phase16_large_repo_benchmark.sh` passes against the repository-scale workflow
  path.
- `just phase16-large-repo-benchmark` executes the same gate successfully.
- `tests/milestone68_phase16_release_checklist_gate.rs` passes and asserts release-checklist
  script/Just/docs/roadmap wiring.
- `scripts/check_phase16_release_checklist.sh` passes against
  `docs/release-checklist-phase16.md`.
- `just phase16-release-checklist` executes the same gate successfully.
- `tests/milestone69_phase16_large_repo_replay.rs` passes and asserts large-repo replay
  script/Just/docs/roadmap wiring.
- `scripts/check_phase16_large_repo_replay.sh` passes against the repository-scale workflow path.
- `just phase16-large-repo-replay` executes the same gate successfully.
- `tests/milestone70_phase16_known_issues_closure.rs` passes and asserts zero-deferred known-issues
  closure posture and release-checklist evidence.
- `scripts/check_phase16_known_issues_budget.sh` passes with deferred threshold `0`.
- `scripts/check_phase16_release_checklist.sh` passes with explicit known-issues deferred=0
  evidence.
- post-slice dogfood commands and full `cargo test` pass.

Validation evidence is satisfied by:

- passing `cargo test --test milestone64_phase16_ga_replay -- --nocapture`,
- passing `bash scripts/check_phase16_deterministic_replay.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`,
- passing `cargo test --test milestone65_phase16_benchmark_pack -- --nocapture`,
- passing `bash scripts/check_phase16_benchmark_pack.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack`,
- passing `cargo test --test milestone66_phase16_known_issues_budget -- --nocapture`,
- passing `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`,
- passing `cargo test --test milestone67_phase16_large_repo_benchmark -- --nocapture`,
- passing `bash scripts/check_phase16_large_repo_benchmark.sh --repo .`,
- passing `cargo test --test milestone68_phase16_release_checklist_gate -- --nocapture`,
- passing `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`,
- passing `cargo test --test milestone69_phase16_large_repo_replay -- --nocapture`,
- passing `bash scripts/check_phase16_large_repo_replay.sh --repo .`,
- passing `cargo test --test milestone70_phase16_known_issues_closure -- --nocapture`,
- passing `bash scripts/check_phase16_known_issues_budget.sh --repo . --doc docs/known-issues-budget-phase16.md`,
- passing `bash scripts/check_phase16_release_checklist.sh --repo . --doc docs/release-checklist-phase16.md`,
- passing `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`,
- passing contract validators listed above.

## Review and CI Gates

Before PR merge/update for this phase:

- run `checklists/PR_CONTRACT_CHECKLIST.md`,
- run contract validators and ensure `.github/workflows/contract-gates.yml` green,
- include Red/Green/Refactor evidence in PR body headings per
  `.github/pull_request_template.md`.

Tier 1 posture does not require mandatory adversarial checklist completion, but later higher-risk
Phase 16 slices must escalate and adopt required controls.
