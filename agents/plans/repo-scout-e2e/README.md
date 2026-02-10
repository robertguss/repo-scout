# repo-scout E2E Release Matrix

This folder defines the local-only, exhaustive e2e validation system for the current `repo-scout` CLI.

## Scope

- Validate all public CLI commands and supported flags.
- Validate all supported languages: Rust, Go, Python, TypeScript (`.ts`) and TypeScript (`.tsx`).
- Validate deterministic replay behavior for JSON commands.
- Validate required release gate command set.
- Record everything observed as pass/warn/fail/info events.

## Risk Tier

- Tier: `1`
- Tier 1 controls are required for this change set.
- Rationale: this work adds test/docs/script infrastructure and process guardrails without changing CLI command behavior or JSON schemas.

## Pass Criteria

- Required matrix and gate checks complete.
- Required artifacts are updated.
- Sign-off requires **zero unresolved findings** (no open findings without resolution or waiver).

## Artifact Index

- `command-matrix.md`
- `language-corpus-matrix.md`
- `gates-and-tooling-matrix.md`
- `runbook.md`
- `issues-log.md`
- `observations.jsonl`
