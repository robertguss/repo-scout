# Phase 16 Large-Repo Benchmark Thresholds

This document defines release-mode wall-clock thresholds for Phase 16 large-repo benchmark checks.

Unlike fixture-pack guardrails, this gate targets the repository-scale workflow path and verifies
timings on heavier end-to-end commands over the working repository.

## Threshold Budgets

- max_index_seconds: 60.0
- max_find_seconds: 8.0
- max_refs_seconds: 8.0
- max_context_seconds: 15.0
- max_verify_plan_seconds: 15.0
- max_diff_impact_seconds: 15.0

## Measurement Notes

- build once in release mode (`cargo build --release`) before timing,
- measure with `/usr/bin/time -p` and parse `real` seconds,
- use warm-cache local runs for repeatability,
- use `--record` mode when recalibrating thresholds.

## Record Mode

```bash
bash scripts/check_phase16_large_repo_benchmark.sh --repo . --record
```
