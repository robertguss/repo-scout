# Gates And Tooling Matrix

This document defines required local gates and informational tooling lanes.

## Required Local Gate Commands

- `just check`
- `just docs-consistency .`
- `just phase18-docs-freshness .`
- `just phase18-maintenance-pack .`
- `just phase15-convergence-pack .`
- `just phase16-deterministic-replay .`
- `just phase16-benchmark-pack .`
- `just phase16-known-issues-budget .`
- `just phase16-release-checklist .`
- `just phase16-large-repo-benchmark .`
- `just phase16-large-repo-replay .`

## Informational Lanes

Python and TypeScript contract lanes are currently inactive in the language manifest.

When tools are available locally, run and record informational results:

- Python lane: `ruff format --check .`, `ruff check . --output-format=full`, `mypy .`, `pytest -q`
- TypeScript lane: `npx tsc --noEmit`, `npx eslint . --max-warnings 0`, `npx prettier --check .`, `npm test`

Missing tools are logged as warnings with owner and follow-up plan.
