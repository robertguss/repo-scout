# repo-scout

`repo-scout` is a local, deterministic CLI for indexing a repository and answering code-navigation questions fast.

Phase 6 is fully implemented and adds change-scope precision and output-focus controls on top of
the Phase 5 recommendation-quality and multi-hop impact-fidelity workflows.

## What It Does

- Incrementally indexes repositories into `<repo>/.repo-scout/index.db`.
- Extracts language-agnostic token occurrences from all files.
- Extracts Rust, TypeScript, and Python symbol/graph metadata through language adapters.
- Supports deterministic terminal and JSON output for automation.

Available commands:

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

## Quick Start

Prerequisite: stable Rust toolchain.

```bash
cargo build
cargo run -- --help
```

Index a repo:

```bash
cargo run -- index --repo /path/to/repo
```

Run core queries:

```bash
cargo run -- find launch --repo /path/to/repo
cargo run -- refs launch --repo /path/to/repo
cargo run -- impact launch --repo /path/to/repo
cargo run -- context --task "modify launch flow and update callers" --repo /path/to/repo --budget 1200
cargo run -- tests-for launch --repo /path/to/repo
cargo run -- verify-plan --changed-file src/lib.rs --repo /path/to/repo
cargo run -- diff-impact --changed-file src/lib.rs --repo /path/to/repo
cargo run -- explain impact_matches --repo /path/to/repo

# Phase 5/6 controls
cargo run -- refs launch --repo /path/to/repo --code-only --exclude-tests
cargo run -- refs launch --repo /path/to/repo --code-only --exclude-tests --max-results 10
cargo run -- find launch --repo /path/to/repo --max-results 10
cargo run -- context --task "update launch flow and reduce test noise" --repo /path/to/repo --budget 1200 --exclude-tests --code-only
cargo run -- tests-for launch --repo /path/to/repo --include-support
cargo run -- verify-plan --changed-file src/lib.rs --changed-line src/lib.rs:20:80 --changed-symbol launch --repo /path/to/repo --max-targeted 6
cargo run -- diff-impact --changed-file src/lib.rs --changed-line src/lib.rs:20:80 --changed-symbol launch --exclude-changed --max-results 12 --repo /path/to/repo
cargo run -- diff-impact --changed-file src/lib.rs --include-imports --repo /path/to/repo
cargo run -- diff-impact --changed-file src/lib.rs --repo /path/to/repo --max-distance 3
```

JSON output is supported by query commands:

```bash
cargo run -- find launch --repo /path/to/repo --json
cargo run -- impact launch --repo /path/to/repo --json
cargo run -- diff-impact --changed-file src/lib.rs --repo /path/to/repo --json
cargo run -- explain impact_matches --repo /path/to/repo --json
```

## Command Behavior Summary

- `index --repo <PATH>`
  - Builds/updates the SQLite index.
  - Prints `index_path`, `schema_version`, `indexed_files`, and `skipped_files`.
- `status --repo <PATH>`
  - Prints index path and schema version.
- `find <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests] [--max-results <N>]`
  - Prefers AST definitions (`ast_definition`), then falls back to text ranking.
- `refs <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests] [--max-results <N>]`
  - Prefers AST references (`ast_reference`), then falls back to text ranking.
  - Fallback ties now prefer code paths over test/docs paths at equal score tiers.
  - `--max-results` applies deterministic truncation after ranking.
  - Scope flags apply to text fallback only; AST-priority behavior is unchanged.
- `impact <SYMBOL> --repo <PATH> [--json]`
  - Returns one-hop incoming graph neighbors (`called_by`, `contained_by`, `imported_by`, `implemented_by`).
- `context --task <TEXT> --repo <PATH> [--budget <N>] [--json] [--exclude-tests] [--code-only]`
  - Uses deterministic token-overlap relevance to rank direct symbol definitions plus graph
    neighbors, truncated by budget.
- `tests-for <SYMBOL> --repo <PATH> [--include-support] [--json]`
  - Returns runnable test targets by default and restores support paths when
    `--include-support` is set.
- `verify-plan --changed-file <PATH> --repo <PATH> [--changed-line <path:start[:end]>] [--changed-symbol <symbol> ...] [--max-targeted <N>] [--json]`
  - Produces deterministic verification steps (bounded targeted test commands + `cargo test`).
  - `--changed-line` and repeatable `--changed-symbol` narrow symbol-derived targeted steps.
  - Default targeted cap is `8`; changed runnable test files are preserved even when
    `--max-targeted=0`.
- `diff-impact --changed-file <PATH> --repo <PATH> [--max-distance <N>] [--include-tests] [--include-imports] [--changed-line <path:start[:end]>] [--changed-symbol <symbol> ...] [--exclude-changed] [--max-results <N>] [--json]`
  - Emits changed-symbol rows plus deterministic bounded multi-hop impacted symbols/test targets.
  - By default, changed-symbol seeds exclude import definitions unless `--include-imports` is set.
  - `--changed-line` and repeatable `--changed-symbol` narrow changed-symbol seeds.
  - `--exclude-changed` omits `distance=0` changed-symbol rows from output.
  - `--max-results` applies deterministic post-sort truncation.
- `explain <SYMBOL> --repo <PATH> [--include-snippets] [--json]`
  - Produces a deterministic symbol dossier with spans, signature, and relationship counts.

## JSON Schemas

Current schema versions:

- `find`, `refs`: `schema_version = 1`
- `impact`, `context`, `tests-for`, `verify-plan`: `schema_version = 2`
- `diff-impact`, `explain`: `schema_version = 3`

See full contracts in `docs/json-output.md`.

## Justfile Workflows

This repo ships `just` shortcuts for both contributors and day-to-day CLI use.

Examples:

```bash
just build
just fmt
just clippy
just test
just dogfood-pre launch
just dogfood-post launch

just index .
just find launch .
just refs launch .
just impact launch .
just context "update launch flow" . 1200
just tests-for launch .
just verify-plan src/lib.rs .
```

## Dogfood Procedure

Before editing a feature slice:

```bash
cargo run -- index --repo .
cargo run -- find <symbol> --repo . --json
cargo run -- refs <symbol> --repo . --json
```

After editing:

```bash
cargo run -- index --repo .
cargo run -- find <symbol> --repo .
cargo run -- refs <symbol> --repo .
cargo test
```

If dogfooding exposes a defect, add a failing integration test first, implement the minimum fix, then refactor with the full suite green.

## Error Recovery

If the index DB is corrupted or not a valid SQLite database, `repo-scout` prints a recovery hint with the exact index path and tells you to delete the DB and rerun `index`.

## More Docs

- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`
- `docs/dogfood-log.md`
- `docs/performance-baseline.md`
