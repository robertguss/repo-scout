# Phase 16 Benchmark Pack Thresholds

This document defines conservative wall-clock guardrails for the Phase 16 benchmark pack script:
`scripts/check_phase16_benchmark_pack.sh`.

These thresholds target deterministic regression detection for release builds, not micro-benchmark
precision.

## Measurement Policy

- build once in release mode (`cargo build --release`) before timing,
- use `/usr/bin/time -p` and parse `real` seconds,
- run checks on warm-cache local development environments,
- prefer stable, low-variance fixture repositories for cross-language command coverage.

## Threshold Budgets

Workspace checks (`--repo <repo-root>`):

- `index`: `<= 25.0s`
- `find --json`: `<= 4.0s`
- `refs --json`: `<= 4.0s`

Cross-language fixture checks (`tests/fixtures/phase15/convergence_pack/*`):

- `index`: `<= 3.0s` per fixture
- `find --json`: `<= 2.5s` per fixture
- `refs --json`: `<= 2.5s` per fixture
- `tests-for --json`: `<= 2.5s` per fixture
- `verify-plan --json`: `<= 2.5s` per fixture
- `diff-impact --json`: `<= 2.5s` per fixture

These budgets are intentionally conservative to minimize false positives on varied contributor
machines while still catching clear regressions.

## Record Mode

Use record mode to inspect current timings without enforcing pass/fail thresholds:

```bash
bash scripts/check_phase16_benchmark_pack.sh --repo . --fixtures tests/fixtures/phase15/convergence_pack --record
```
