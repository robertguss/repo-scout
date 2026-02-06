# Performance Baseline (Phase 2 Preflight)

This document defines a lightweight baseline process before major Phase 2
features land. The goal is to catch obvious regressions in index/query latency
while contracts and data model are evolving.

## Scope

Track wall-clock timing for:

- `index` on a fixed local fixture repository.
- one `find` query on that same repository.
- one `refs` query on that same repository.

This is a guardrail, not a precise benchmark suite.

## Commands

From repository root:

```bash
just perf-baseline launch
```

Or run commands manually:

```bash
/usr/bin/time -p cargo run --release -- index --repo .
/usr/bin/time -p cargo run --release -- find launch --repo . --json
/usr/bin/time -p cargo run --release -- refs launch --repo . --json
```

## Recording

Record timings in `docs/dogfood-log.md` when:

- a major milestone starts,
- a major milestone ends,
- you notice unexpected slowdowns during dogfooding.

Recommended log fields:

- date/time,
- fixture/repo used,
- command,
- elapsed wall time,
- notes about machine load or unusual conditions.
