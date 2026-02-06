# Performance Baseline

This document defines lightweight, repeatable timing checks for `repo-scout`.

The goal is regression detection, not benchmark-grade precision.

## Scope

Track wall-clock timings for:

- indexing (`index`)
- core lookup (`find`, `refs`)
- Phase 2 queries (`impact`, `context`, `tests-for`, `verify-plan`)

## Commands

From repository root:

```bash
just perf-baseline-core run
just perf-baseline-full run src/query/mod.rs "update run and verify refs behavior"
```

Equivalent manual commands:

```bash
/usr/bin/time -p cargo run --release -- index --repo .
/usr/bin/time -p cargo run --release -- find run --repo . --json
/usr/bin/time -p cargo run --release -- refs run --repo . --json
/usr/bin/time -p cargo run --release -- impact run --repo . --json
/usr/bin/time -p cargo run --release -- context --task "update run and verify refs behavior" --repo . --budget 1200 --json
/usr/bin/time -p cargo run --release -- tests-for run --repo . --json
/usr/bin/time -p cargo run --release -- verify-plan --changed-file src/query/mod.rs --repo . --json
```

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
