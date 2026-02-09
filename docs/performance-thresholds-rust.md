# Rust Performance Guardrails

This document defines practical, repeatable Rust-focused runtime budgets for Phase 11 production
closure. Budgets are intentionally conservative to avoid flaky failures across developer machines.

## Scope

- Use release-mode commands (`target/release/repo-scout`).
- Prefer warm-cache interpretation: one `cargo build --release` first, then validate command times.
- Treat guardrails as regression detectors, not microbenchmarks.

## Thresholds

| Guardrail                     | Command (conceptual)                                         | Threshold (seconds) |
| ---------------------------- | ------------------------------------------------------------- | ------------------- |
| `repo_index`                | `repo-scout index --repo <repo>`                             | `15.0`              |
| `repo_find_json`            | `repo-scout find run --repo <repo> --json`                   | `2.0`               |
| `repo_refs_json`            | `repo-scout refs run --repo <repo> --json`                   | `2.0`               |
| `fixture_index`             | `repo-scout index --repo <fixture>`                          | `2.0`               |
| `fixture_impact_json`       | `repo-scout impact helper --repo <fixture> --json`           | `2.0`               |
| `fixture_diff_impact_json`  | `repo-scout diff-impact --changed-file src/util/mod.rs ...`  | `2.0`               |

The default fixture for Rust guardrails is
`tests/fixtures/phase11/rust_production/corpus`.

## Validator

Run:

```bash
bash scripts/check_rust_perf_guardrails.sh --repo . --fixture tests/fixtures/phase11/rust_production/corpus
```

For baseline recording without pass/fail gating:

```bash
bash scripts/check_rust_perf_guardrails.sh --repo . --fixture tests/fixtures/phase11/rust_production/corpus --record
```
