# `repo-scout` Sequential Roadmap to Production-Ready Languages and High-Bar GA

## Goal

Deliver a low-risk, sequential roadmap that takes `repo-scout` from current state to
production-ready support for four languages (Rust, Go, Python, TypeScript), then closes with one
cross-language High-Bar/GA phase so the project is mostly complete and maintenance-only afterward.

## Strategy Decisions (Locked)

- Primary objectives: product capability + quality hardening.
- Delivery mode: low-risk, gate-based sequencing (not broad parallel language development).
- Language order: Rust first (depth), then Go, then Python, then TypeScript.
- Go scope for next phase: minimal but useful (`find` definitions only).
- Per-language exit bar: `Production-ready` (not just core parity).
- Final release posture: one cross-language `High-Bar/GA` phase after all languages are
  production-ready.

## Current State Review Snapshot (2026-02-09)

Evidence from local audit and commands:

- Quality gates are currently green:
  `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and evidence validator.
- Contract-TDD validator is healthy but requires commits for strict range checks:
  `validate_tdd_cycle.sh --base origin/main` fails on empty ranges unless `--allow-empty-range`.
- Core architecture is stable with schema `v3`, deterministic query paths, and adapter-based
  extraction for Rust/TypeScript/Python/Go.
- Recent Phase 10 updates:
  - test-target synthesis now avoids emitting non-Rust `cargo test --test` targets and only maps
    direct `tests/<file>.rs` paths to runnable Rust integration-test commands.
  - test-like path classification now includes common TS/Python conventions
    (`*.test.ts`, `*.spec.ts`, `test_*.py`, `*_test.py`) in addition to existing Rust/test-dir
    patterns (`tests/`, `/tests/`, `*_test.rs`).
  - Go adapter support exists for AST-backed definition indexing used by `find`.
- Planned runner-aware recommendation expansion remains a residual hardening thread for later
  production-readiness phases.

Implication: the project has a strong base and can move quickly, but remaining work is mainly
language-depth and hardening rather than new architectural foundations.

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

## Phase 10 (Current): Rust Hardening + Go `find` MVP

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

## Phase 11: Rust Production-Ready Closure

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

## Phase 12: Go Production-Ready Closure

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

## Phase 13: Python Production-Ready Closure

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

## Phase 14: TypeScript Production-Ready Closure

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

## Phase 15: Cross-Language Production Convergence

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

## Phase 16: High-Bar / GA Hardening

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
- `agents/plans/repo-scout-phase11-execplan.md`
- `agents/plans/repo-scout-phase12-execplan.md`
- `agents/plans/repo-scout-phase13-execplan.md`
- `agents/plans/repo-scout-phase14-execplan.md`
- `agents/plans/repo-scout-phase15-execplan.md`
- `agents/plans/repo-scout-phase16-execplan.md`

Each plan should include `Contract Inputs`, `AGENTS.md Constraints`, and `Risk Tier and Required
Controls` per Tiger-era policy.

## Completion View

At roadmap completion, `repo-scout` will have:

- Production-ready Rust, Go, Python, and TypeScript support.
- One consolidated high-bar release-hardening pass completed.
- Stable command/JSON contracts and deterministic behavior across languages.
- Mature test, doc, and evidence artifacts sufficient for maintenance-mode operation.
