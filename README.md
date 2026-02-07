# repo-scout

`repo-scout` is a local, deterministic CLI for indexing a repository and answering code-navigation questions fast.

Phase 4 is fully implemented and adds precision/disambiguation/noise-control workflows on the
existing command surface.

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

# Phase 4 controls
cargo run -- refs launch --repo /path/to/repo --code-only --exclude-tests
cargo run -- diff-impact --changed-file src/lib.rs --changed-line src/lib.rs:20:80 --repo /path/to/repo
cargo run -- diff-impact --changed-file src/lib.rs --include-imports --repo /path/to/repo
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
- `find <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests]`
  - Prefers AST definitions (`ast_definition`), then falls back to text ranking.
- `refs <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests]`
  - Prefers AST references (`ast_reference`), then falls back to text ranking.
  - Scope flags apply to text fallback only; AST-priority behavior is unchanged.
- `impact <SYMBOL> --repo <PATH> [--json]`
  - Returns one-hop incoming graph neighbors (`called_by`, `contained_by`, `imported_by`, `implemented_by`).
- `context --task <TEXT> --repo <PATH> [--budget <N>] [--json]`
  - Ranks direct symbol hits + graph neighbors for the task, truncated by budget.
- `tests-for <SYMBOL> --repo <PATH> [--json]`
  - Finds test-like files that directly reference a symbol.
- `verify-plan --changed-file <PATH> --repo <PATH> [--json]`
  - Produces deterministic verification steps (targeted test commands + `cargo test`).
- `diff-impact --changed-file <PATH> --repo <PATH> [--max-distance <N>] [--include-tests] [--include-imports] [--changed-line <path:start[:end]>] [--json]`
  - Emits changed-symbol rows plus deterministic one-hop impacted symbols/test targets.
  - By default, changed-symbol seeds exclude import definitions unless `--include-imports` is set.
  - `--changed-line` limits changed-symbol seeds to symbols overlapping the provided ranges.
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
