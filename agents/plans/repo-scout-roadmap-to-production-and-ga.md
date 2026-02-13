# `repo-scout` Sequential Roadmap to Production-Ready Languages and High-Bar GA

## Goal

Deliver a low-risk, sequential roadmap that takes `repo-scout` from current state to
production-ready support for four languages (Rust, Go, Python, TypeScript), then closes with one
cross-language High-Bar/GA phase so the project is mostly complete and maintenance-only afterward.

## Strategy Decisions (Locked)

- Primary objectives: product capability + quality hardening.
- Delivery mode: low-risk, gate-based sequencing (not broad parallel language development).
- Language order: Rust first (depth), then Go, then Python, then TypeScript.
- Historical Phase 10 Go scope: minimal but useful (`find` definitions only).
- Per-language exit bar: `Production-ready` (not just core parity).
- Final release posture: one cross-language `High-Bar/GA` phase after all languages are
  production-ready.

## Current State Review Snapshot (2026-02-10)

Evidence from local audit and commands:

- Quality gates are currently green:
  `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `validate_tdd_cycle.sh --base origin/main --allow-empty-range`, and evidence validator.
- Contract-TDD validator is healthy but requires commits for strict range checks:
  `validate_tdd_cycle.sh --base origin/main` fails on empty ranges unless `--allow-empty-range`.
- Core architecture is stable with schema `v3`, deterministic query paths, and adapter-based
  extraction for Rust/TypeScript/Python/Go.
- Recent phase-completion updates (11-14):
  - Rust production-ready closure is complete with module-qualified call resolution hardening,
    deterministic behavior checks, and Rust performance guardrails.
  - Go production-ready closure is complete with AST-backed `refs` plus import-alias-aware
    graph/impact attribution.
  - Python production-ready closure is complete with strict explicit-`pytest` runner-aware
    recommendations and relative-import caller attribution in `diff-impact`.
  - TypeScript production-ready closure is complete with strict explicit Jest/Vitest
    runner-aware recommendations and directory-import (`./module` -> `./module/index.ts`)
    caller attribution in `diff-impact`.
- All four language-specific production-ready closures are complete, and cross-language convergence
  (Phase 15) is complete with three slices:
  Go `_test.go` filtering parity, Go runnable recommendation parity, and the integrated
  convergence-pack command-contract gate.
- Phase 16 High-Bar hardening closure includes deterministic replay validation via
  `scripts/check_phase16_deterministic_replay.sh` and `just phase16-deterministic-replay`.
- Phase 16 also includes benchmark-pack regression budgets:
  `scripts/check_phase16_benchmark_pack.sh`, `just phase16-benchmark-pack`, and
  `legacy phase16 thresholds doc (removed)`.
- Phase 16 also includes known-issues budget enforcement:
  `scripts/check_phase16_known_issues_budget.sh`, `just phase16-known-issues-budget`, and
  `legacy known-issues budget doc (removed)` (closure posture now enforces `max_deferred: 0`).
- Phase 16 also includes larger-repo benchmark guardrails:
  `scripts/check_phase16_large_repo_benchmark.sh`, `just phase16-large-repo-benchmark`, and
  `legacy large-repo thresholds doc (removed)`.
- Phase 16 also includes release-checklist closure gating:
  `scripts/check_phase16_release_checklist.sh`, `just phase16-release-checklist`, and
  `legacy release checklist doc (removed)`.
- Phase 16 also includes large-repo deterministic replay scenarios:
  `scripts/check_phase16_large_repo_replay.sh`, `just phase16-large-repo-replay`, and
  `legacy performance baseline doc (removed)`.

Implication: the project has a strong base and can move quickly; remaining work is now
maintenance/backlog rather than core capability delivery.

## Definitions

### Production-Ready (Language Exit Bar)

A language is `Production-ready` in this roadmap when all are true:

1. Stable `index` + accurate `find` + usable `refs` behavior for common repository patterns.
2. Integration tests cover valid, invalid, boundary, and determinism cases.
3. Performance checks show no major regression versus prior baseline.
4. Documentation and dogfooding transcripts match real behavior.
5. Contract validators and CI gates pass without exceptions.

### High-Bar / GA (Final Cross-Language Exit)

The project is `High-Bar/GA` ready when all are true:

1. Large-repo benchmark pack passes across all four languages.
2. Known-issues budget is explicitly defined and within target.
3. Release checklist gates (quality, docs, evidence, rollback plan) all pass.
4. Cross-language behavior is deterministic and contract-stable.

## Phase Roadmap

## Phase 10 (Completed): Rust Hardening + Go `find` MVP

Purpose:
Ship immediate Go value while continuing Rust reliability hardening.

Scope:

- Rust hardening for deterministic/recommendation correctness defects.
- Add Go adapter and AST-backed Go definition indexing for `find`.
- Keep schema stable and avoid Go `refs` in this phase.

Exit gate:

- Rust hardening tests added and green.
- `find` returns Go definitions with deterministic JSON output.
- No regression in existing Rust/TypeScript/Python behavior.

Primary artifact:

- `agents/plans/repo-scout-phase10-execplan.md`

## Phase 11 (Completed): Rust Production-Ready Closure

Purpose:
Finish Rust hardening to full production-ready bar.

Scope:

- Close remaining Rust correctness and determinism edge cases.
- Expand Rust fixture corpus for realistic repository shapes.
- Tighten performance baseline coverage and threshold checks.
- Ensure docs and dogfooding fully reflect Rust behavior.

Exit gate:

- Rust meets `Production-ready` definition end-to-end.
- All Rust-focused quality/perf gates pass repeatedly.

Risk posture:

- Tier 1 by default; escalate to Tier 2 if schema or persistence invariants are touched.

## Phase 12 (Completed): Go Production-Ready Closure

Purpose:
Evolve Go from `find` MVP to production-ready language support.

Scope:

- Add Go `refs` capability (AST-first where feasible, deterministic fallback otherwise).
- Add Go graph/impact compatibility needed for practical workflows.
- Expand Go fixtures (modules, packages, methods, interfaces, aliases).
- Hardening for determinism, ranking stability, and test coverage.

Exit gate:

- Go meets `Production-ready` definition.
- Go commands are usable in day-to-day dogfooding.

## Phase 13 (Completed): Python Production-Ready Closure

Purpose:
Take existing Python support to production-ready quality.

Scope:

- Complete runner-aware recommendation correctness for Python (`pytest`) under strict detection.
- Broaden test-like path and runnable-target handling where needed.
- Improve Python semantic edge-case handling and regression tests.
- Refresh Python docs, dogfood, and perf evidence.

Exit gate:

- Python meets `Production-ready` definition.
- Targeted test recommendations are runnable and deterministic in explicit runner contexts.

## Phase 14 (Completed): TypeScript Production-Ready Closure

Purpose:
Take existing TypeScript support to production-ready quality.

Scope:

- Complete strict runner-aware recommendations for Jest/Vitest contexts.
- Harden namespace/member/import-resolution edge cases.
- Expand fixture corpus for monorepo and alias-heavy structures.
- Refresh TypeScript docs, dogfood, and perf evidence.

Exit gate:

- TypeScript meets `Production-ready` definition.
- Node-targeted recommendation behavior is deterministic and strict-mode safe.

## Phase 15 (Completed): Cross-Language Production Convergence

Purpose:
Unify behavior and operator experience across all four languages before GA hardening.

Scope:

- Normalize cross-language scope/test filtering semantics.
- Verify schema and command-contract consistency across languages.
- Consolidate docs for mixed-language workflows.
- Build a cross-language benchmark and dogfood corpus used by GA phase.

Exit gate:

- All four languages simultaneously satisfy production-ready criteria under one integrated
  validation pack.

## Phase 16 (Completed): High-Bar / GA Hardening

Purpose:
Perform final release-grade hardening after production-ready parity exists.

Scope:

- Large-repo benchmark runs and regression budgets across all languages.
- Known-issues budget triage and closure/deferral decisions with ownership.
- Release-checklist completion (quality, evidence, rollback, docs, CI).
- Final deterministic replay checks for critical command scenarios.

Exit gate:

- `High-Bar/GA` definition is satisfied.
- Remaining work is maintenance/backlog only, not core capability gaps.

## Cross-Phase Rules (Low-Risk Mode)

1. Only one language-depth stream is active at a time.
2. Every feature slice follows strict Red -> Green -> Refactor with evidence.
3. No schema-version changes unless explicitly planned and risk-tier escalated.
4. Dogfooding commands run before/after each milestone.
5. Contract validators run before PR updates.
6. Any ambiguous risk decision defaults upward (stricter control).

## Recommended Artifact Plan

Create one ExecPlan per phase:

- `agents/plans/repo-scout-phase10-execplan.md` (created)
- `agents/plans/repo-scout-phase11-execplan.md` (created)
- `agents/plans/repo-scout-phase12-execplan.md` (created)
- `agents/plans/repo-scout-phase13-execplan.md` (created)
- `agents/plans/repo-scout-phase14-execplan.md` (created)
- `agents/plans/repo-scout-phase15-execplan.md` (created, complete)
- `agents/plans/repo-scout-phase16-execplan.md` (created, complete)
- `agents/plans/repo-scout-phase17-execplan.md` (created, complete)
- `agents/plans/repo-scout-phase18-execplan.md` (created, complete)

Each plan should include `Contract Inputs`, `AGENTS.md Constraints`, and `Risk Tier and Required
Controls` per Tiger-era policy.

## Completion View

At roadmap completion, `repo-scout` will have:

- Production-ready Rust, Go, Python, and TypeScript support.
- One consolidated high-bar release-hardening pass completed.
- Stable command/JSON contracts and deterministic behavior across languages.
- Mature test, doc, and evidence artifacts sufficient for maintenance-mode operation.

Current status (2026-02-10): Rust/Go/Python/TypeScript production-ready closures are complete
(Phases 11-14). Phase 15 convergence is complete (Go `_test.go` cross-language filtering parity,
Go runnable recommendation parity, and integrated convergence-pack validation gate). Phase 16
High-Bar/GA hardening is complete with six closure slices: integrated deterministic replay gate for
repeated `find`/`refs`/`tests-for`/`verify-plan`/`diff-impact` outputs, benchmark-pack timing gate
and threshold budgets, known-issues budget triage/ownership gate, larger-repo benchmark
guardrails, release-checklist closure gating, and large-repo deterministic replay scenarios; zero
deferred known issues remain at closure. Phase 17 docs-consistency closure is complete. Phase 18
maintenance/backlog hardening is complete with backlog governance and docs freshness guardrails
(`legacy maintenance backlog doc (removed)`, `legacy maintenance cadence doc (removed)`,
`scripts/check_phase18_maintenance_pack.sh`, `scripts/check_phase18_docs_freshness.sh`).
