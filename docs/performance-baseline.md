# Performance Baseline

This document defines lightweight, repeatable timing checks for `repo-scout`.

The goal is regression detection, not benchmark-grade precision.

## Scope

Track wall-clock timings for:

- indexing (`index`)
- core lookup (`find`, `refs`)
- Phase 2 queries (`impact`, `context`, `tests-for`, `verify-plan`)
- Phase 4 precision controls (`diff-impact --include-imports/--changed-line`,
  `refs --code-only --exclude-tests`)
- Phase 5 recommendation/fidelity controls (`tests-for --include-support`,
  `verify-plan --max-targeted`, context token-overlap matching, multi-hop
  `diff-impact --max-distance`)
- Phase 6 change-scope/output-focus controls (`context --exclude-tests --code-only`,
  `verify-plan --changed-line/--changed-symbol`,
  `diff-impact --changed-symbol --exclude-changed --max-results`, `find/refs --max-results`)
- Phase 7/8 semantic precision and hardening controls (TypeScript/Python module-aware `diff-impact`,
  calibrated `impact`/`diff-impact` semantic ranking, explicit `diff-impact --exclude-tests`,
  deterministic row-level terminal output checks, fixture benchmark pack under
  `tests/fixtures/phase8/semantic_precision`)
- Phase 10 Go `find` MVP checks (Go AST-backed definition indexing and deterministic `find` JSON
  on fixture corpus under `tests/fixtures/phase10/go_find`)
- Phase 12 Go production-closure checks (Go AST-backed `refs` plus import-alias-aware
  `impact`/`diff-impact` traversal on fixture corpus under `tests/fixtures/phase12/go_refs`)
- Phase 13 Python production-closure checks (explicit-pytest runner-aware `tests-for`/`verify-plan`
  and relative-import `diff-impact` traversal on fixture corpus under
  `tests/fixtures/phase13/python_recommendations`)
- Phase 11 Rust production-closure guardrails (`scripts/check_rust_perf_guardrails.sh`,
  `docs/performance-thresholds-rust.md`, and fixture corpus under
  `tests/fixtures/phase11/rust_production/corpus`)

## Commands

From repository root:

```bash
just perf-baseline-core run
just perf-baseline-full run src/query/mod.rs "update run and verify refs behavior"
just perf-rust-guardrails .
just perf-rust-record .
```

Equivalent manual commands:

```bash
/usr/bin/time -p cargo run --release -- index --repo .
/usr/bin/time -p cargo run --release -- find run --repo . --json
/usr/bin/time -p cargo run --release -- find run --repo . --max-results 10 --json
/usr/bin/time -p cargo run --release -- refs run --repo . --json
/usr/bin/time -p cargo run --release -- refs run --repo . --max-results 10 --json
/usr/bin/time -p cargo run --release -- impact run --repo . --json
/usr/bin/time -p cargo run --release -- context --task "update run and verify refs behavior" --repo . --budget 1200 --json
/usr/bin/time -p cargo run --release -- context --task "update run and verify refs behavior" --repo . --budget 1200 --exclude-tests --code-only --json
/usr/bin/time -p cargo run --release -- tests-for run --repo . --json
/usr/bin/time -p cargo run --release -- tests-for run --repo . --include-support --json
/usr/bin/time -p cargo run --release -- verify-plan --changed-file src/query/mod.rs --repo . --json
/usr/bin/time -p cargo run --release -- verify-plan --changed-file src/query/mod.rs --repo . --max-targeted 6 --json
/usr/bin/time -p cargo run --release -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo . --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo .
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --changed-line src/query/mod.rs:132:220 --repo .
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
/usr/bin/time -p cargo run --release -- refs verify_plan_for_changed_files --repo . --code-only --exclude-tests --json
/usr/bin/time -p cargo run --release -- index --repo tests/fixtures/phase8/semantic_precision
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json
/usr/bin/time -p cargo run --release -- impact helper --repo tests/fixtures/phase8/semantic_precision --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/indexer/languages/typescript.rs --repo . --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/indexer/languages/python.rs --repo . --json
/usr/bin/time -p cargo run --release -- refs helper --repo . --code-only --exclude-tests --max-results 10 --json
/usr/bin/time -p cargo run --release -- index --repo tests/fixtures/phase10/go_find
/usr/bin/time -p cargo run --release -- find SayHello --repo tests/fixtures/phase10/go_find --json
/usr/bin/time -p cargo run --release -- find Greeter --repo tests/fixtures/phase10/go_find --code-only --exclude-tests --json
/usr/bin/time -p cargo run --release -- index --repo tests/fixtures/phase12/go_refs
/usr/bin/time -p cargo run --release -- refs Helper --repo tests/fixtures/phase12/go_refs --json
/usr/bin/time -p cargo run --release -- impact SayHello --repo tests/fixtures/phase12/go_refs --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/util/util.go --repo tests/fixtures/phase12/go_refs --json
/usr/bin/time -p cargo run --release -- index --repo tests/fixtures/phase13/python_recommendations
/usr/bin/time -p cargo run --release -- tests-for compute_plan --repo tests/fixtures/phase13/python_recommendations --json
/usr/bin/time -p cargo run --release -- verify-plan --changed-file src/service.py --repo tests/fixtures/phase13/python_recommendations --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/pkg/util.py --repo tests/fixtures/phase13/python_recommendations --json
bash scripts/check_rust_perf_guardrails.sh --repo . --fixture tests/fixtures/phase11/rust_production/corpus
bash scripts/check_rust_perf_guardrails.sh --repo . --fixture tests/fixtures/phase11/rust_production/corpus --record
```

## Coverage Check

Use `cargo-llvm-cov` for repeatable coverage summaries:

```bash
rustup run stable cargo llvm-cov --workspace --all-targets --summary-only
```

Latest local baseline (`2026-02-08`):

- regions: `85.32%`
- lines: `90.30%`
- functions: `86.82%`

## Recording

Capture results in `docs/dogfood-log.md` when:

- starting major work,
- finishing major work,
- observing suspicious slowdown.

Recommended fields:

- date/time,
- machine/load notes,
- command,
- elapsed wall time,
- any unusual conditions (cold cache, background workload, etc.).
