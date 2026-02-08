# repo-scout Validation Report (Phase 7 Checkpoint)

- Date (UTC): `2026-02-08 02:16:46Z`
- Repository: `/Users/robertguss/Projects/experiments/repo-scout`
- Branch: `codex/phase7-plan-and-semantic-precision`
- HEAD: `bed9fe3` (`Add Phase 7 exec plan for semantic precision and benchmarks`)

## Scope

Validated end-to-end behavior across:

- build/format/lint/test gates,
- all CLI commands (`index`, `status`, `find`, `refs`, `impact`, `context`, `tests-for`, `verify-plan`, `diff-impact`, `explain`),
- terminal + JSON output contracts,
- error-path behavior,
- performance baseline timings,
- coverage summary,
- Phase 7 semantic-precision probes from the exec plan scenarios.

## Executive Summary

- Core app behavior is stable and working through the currently implemented milestone surface (up to Milestone 30).
- `cargo test` is fully green (`103` tests).
- Manual CLI matrix checks are fully green (`43/43`).
- JSON command/schema envelopes are consistent with docs (`v1/v2/v3` split).
- Coverage baseline meets documented threshold (`85.32%` regions, `90.30%` lines, `86.82%` functions).
- Phase 7 semantic-precision outcomes described in `agents/repo-scout-phase7-execplan.md` are **not** implemented yet in this branch (plan exists; implementation/tests do not).

## Validation Evidence

### Build/Test/Lint Gates

- `cargo fmt --all -- --check` -> pass
- `cargo build` -> pass
- `cargo test` -> pass (`103` tests)
- `rustup run stable cargo llvm-cov --workspace --all-targets --summary-only` -> pass
- `cargo clippy --all-targets --all-features -- -D warnings` -> **fails** (see findings)

### Dogfood Baseline (per AGENTS.md)

- `cargo run -- index --repo .` -> pass (`schema_version: 3`, `indexed_files: 10`, `skipped_files: 68`)
- `cargo run -- find verify_plan_for_changed_files --repo . --json` -> pass (`schema_version: 1`)
- `cargo run -- refs verify_plan_for_changed_files --repo . --json` -> pass (`schema_version: 1`)

### Full CLI Matrix

All command-level checks passed (`43` total), including:

- root and per-subcommand help output,
- terminal mode and JSON mode for all query commands,
- scoped flags (`--code-only`, `--exclude-tests`, `--max-results`, `--changed-line`, `--changed-symbol`, `--exclude-changed`, `--include-support`, `--include-snippets`),
- expected failure paths (missing required args, invalid `--changed-line`).

### JSON Contract Checks

Verified runtime command/schema envelope values:

- `find` -> `schema_version=1`
- `refs` -> `schema_version=1`
- `impact` -> `schema_version=2`
- `context` -> `schema_version=2`
- `tests-for` -> `schema_version=2`
- `verify-plan` -> `schema_version=2`
- `diff-impact` -> `schema_version=3`
- `explain` -> `schema_version=3`

### Coverage Snapshot

From `cargo llvm-cov --summary-only`:

- regions: `85.32%`
- lines: `90.30%`
- functions: `86.82%`

## Performance Baseline (Release, wall clock)

| Command | Real (s) |
|---|---:|
| `index --repo .` | 18.21 |
| `find run --json` | 0.31 |
| `find run --max-results 10 --json` | 0.29 |
| `refs run --json` | 0.29 |
| `refs run --max-results 10 --json` | 0.38 |
| `impact run --json` | 0.31 |
| `context ... --budget 1200 --json` | 0.31 |
| `context ... --exclude-tests --code-only --json` | 0.41 |
| `tests-for run --json` | 0.29 |
| `tests-for run --include-support --json` | 0.31 |
| `verify-plan --changed-file src/query/mod.rs --json` | 0.29 |
| `verify-plan` scoped (`--changed-line`, `--changed-symbol`) | 0.31 |
| `diff-impact --changed-file src/query/mod.rs --json` | 0.31 |
| `diff-impact --max-distance 3 --json` | 0.31 |
| `diff-impact --changed-line ...` (terminal) | 0.28 |
| `diff-impact` scoped (`--changed-symbol`, `--exclude-changed`, `--max-results`) | 0.29 |
| `refs verify_plan_for_changed_files --code-only --exclude-tests --json` | 0.31 |
| `explain run --json` | 0.32 |

## Phase Status Assessment

| Phase | Status | Evidence |
|---|---|---|
| 1-6 (implemented milestone surface) | Pass | Milestone tests through `tests/milestone30_query_focus.rs` all pass; CLI matrix all green. |
| 7 (semantic precision + benchmark guardrails) | Not implemented in current branch | `agents/repo-scout-phase7-execplan.md` still has Milestones 32-36 unchecked; no `tests/milestone32_*.rs`..`milestone35_*.rs` files; semantic-gap scenarios still reproduce. |

## Findings / Refinements (Prioritized)

1. **[P0] Phase 7 deliverables are not present in code despite plan file existing.**
   - `agents/repo-scout-phase7-execplan.md` has pending Milestones 32-36.
   - Expected Phase 7 test files/fixtures referenced in plan are absent (`tests/milestone32_semantic_contracts.rs`, `tests/milestone33_typescript_semantics.rs`, `tests/milestone34_python_semantics.rs`, `tests/milestone35_quality_benchmark.rs`, `tests/fixtures/phase7/semantic_precision/`).
   - Reproduced planned semantic gap directly:
     - TypeScript probe: `diff-impact --changed-file src/util_a.ts` -> only 1 row (`changed_symbol`), `called_by=0`.
     - Python probe: `diff-impact --changed-file src/pkg_a/util.py` -> only 1 row (`changed_symbol`), `called_by=0`.

2. **[P1] Clippy quality gate is red under repository’s strict profile (`-D warnings`).**
   - Failing locations include:
     - `tests/common/mod.rs:16` (`collapsible_if`)
     - `src/indexer/languages/python.rs:210` (`question_mark`)
     - `src/indexer/languages/python.rs:480` (`double_ended_iterator_last`)
     - `src/indexer/languages/rust.rs:224` (`double_ended_iterator_last`)
     - `src/indexer/languages/typescript.rs:296` (`question_mark`)
     - `src/indexer/rust_ast.rs:295` (`collapsible_if`)

3. **[P2] `diff-impact --include-tests` is effectively a compatibility no-op and cannot be disabled via CLI.**
   - Runtime comparison on current repo:
     - without flag: `include_tests=true`, `results=67`
     - with `--include-tests`: `include_tests=true`, `results=67`
   - Current behavior is documented, but UX remains ambiguous for users expecting a toggle.

4. **[P2] `diff-impact` terminal output is low-utility for manual usage.**
   - Terminal output prints metadata/counts only (no per-result rows), unlike other commands.
   - This creates a usability gap for interactive debugging unless `--json` + tooling is used.

5. **[P3] Docs framing still states Phase 6 as the current architecture state.**
   - `README.md` and `docs/architecture.md` are internally consistent with current behavior, but if Phase 7 is considered “implemented,” docs are out of date.

## Conclusion

The application is robust and regression-safe for the currently implemented feature set. The major gap is phase-level: Phase 7 execution is still pending in this branch, and the exact semantic-precision deficits called out in the Phase 7 plan are reproducible.
