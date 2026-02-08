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
  `verify-plan --max-targeted`, context token-overlap matching, multi-hop `diff-impact --max-distance`)
- Phase 6 change-scope/output-focus controls (`context --exclude-tests --code-only`,
  `verify-plan --changed-line/--changed-symbol`, `diff-impact --changed-symbol --exclude-changed --max-results`,
  `find/refs --max-results`)

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
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --changed-line src/query/mod.rs:132:220 --repo .
/usr/bin/time -p cargo run --release -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
/usr/bin/time -p cargo run --release -- refs verify_plan_for_changed_files --repo . --code-only --exclude-tests --json
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
