# Command Matrix

This matrix defines exhaustive command coverage for `repo-scout`.

## Covered Commands

- `index`
- `status`
- `find`
- `refs`
- `impact`
- `context`
- `tests-for`
- `verify-plan`
- `diff-impact`
- `explain`

## Valid Matrix Dimensions

- Terminal and JSON output modes where supported.
- Scope flags (`--code-only`, `--exclude-tests`).
- Limit flags (`--max-results`, `--max-targeted`, `--max-distance`).
- Changed-file forms (`relative`, `./relative`, `absolute`).
- Changed-line and changed-symbol filters.
- Conflict checks (`diff-impact --include-tests` vs `--exclude-tests`).

## Negative/Error Matrix

- negative cases are required for every command family.
- Missing required arguments.
- Invalid repo paths.
- Invalid changed-line values.
- Invalid conflict combinations.
- Unknown command forms.

Each invalid scenario must be recorded with deterministic command, expected failure, and observed failure.
