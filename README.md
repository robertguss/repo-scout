# repo-scout

`repo-scout` is a local CLI for indexing source repositories and querying symbol matches quickly.

This project currently ships a hybrid approach:

- Language-agnostic text indexing for all files.
- Rust AST extraction (Tree-sitter) for definitions (`fn`, `struct`, `enum`, `trait`, `mod`, `const`, `type`, `use`) plus call references.
- Deterministic terminal and JSON output for scripting and agent use.

## Status

This is an actively evolving v0. The core workflow is implemented and tested:

- `index`: build or update a local index.
- `status`: show index location and schema version.
- `find`: search for likely definitions.
- `refs`: search for likely references.
- `impact`: inspect one-hop graph impact around a symbol.
- `context`: build a budgeted context bundle for an editing task.
- `tests-for`: map a symbol to likely test targets.
- `verify-plan`: suggest deterministic validation steps for changed files.

## Quick Start

Prerequisites:

- Rust toolchain (stable).

Build:

```bash
cargo build
```

Run help:

```bash
cargo run -- --help
```

Index a repository:

```bash
cargo run -- index --repo /path/to/repo
```

Query it:

```bash
cargo run -- find launch --repo /path/to/repo
cargo run -- refs launch --repo /path/to/repo
cargo run -- impact launch --repo /path/to/repo
cargo run -- context --task "modify launch flow and update callers" --repo /path/to/repo --budget 1200
cargo run -- tests-for launch --repo /path/to/repo
cargo run -- verify-plan --changed-file src/lib.rs --repo /path/to/repo
```

JSON output:

```bash
cargo run -- find launch --repo /path/to/repo --json
```

## Command Reference

### `index`

Builds or updates `.repo-scout/index.db` under the target repository.

Output fields:

- `index_path`
- `schema_version`
- `indexed_files`
- `skipped_files`

`indexed_files` is the number of changed/new files processed this run.
`skipped_files` is the number of unchanged files skipped by hash comparison.

### `status`

Prints the index path and schema version for a repository.

### `find <symbol>`

Returns likely definitions for `symbol`.

Current behavior:

- Prefer Rust AST definitions (`ast_definition`) when present, including methods and non-function item kinds.
- Otherwise use text fallback ranking:
  - exact token matches first (`exact_symbol_name`)
  - substring matches next (`text_substring_match`)

### `refs <symbol>`

Returns likely references for `symbol`.

Current behavior:

- Prefer Rust AST call references (`ast_reference`) when present.
- Otherwise use the same text fallback ranking as `find`.

### `impact <symbol>`

Returns first-order graph neighbors likely impacted by changing `symbol`.

Current behavior:

- Uses persisted symbol graph edges (`calls`, `contains`, `imports`, `implements`).
- Emits deterministic one-hop results with relationship labels.

### `context --task <text> --repo <PATH> [--budget <N>]`

Returns a ranked, budget-limited context bundle for a task description.

Current behavior:

- Extracts task keywords and prioritizes direct symbol matches.
- Expands one graph hop for nearby context.
- Truncates deterministically based on budget.

### `--json`

Supported on `find`, `refs`, `impact`, `context`, `tests-for`, and `verify-plan`. Emits deterministic JSON with a stable top-level command schema and ordered results.

### `tests-for <symbol>`

Returns likely test targets for `symbol`.

Current behavior:

- Searches test-like files for exact symbol hits.
- Deduplicates by test target and applies deterministic confidence tiers.

### `verify-plan --changed-file <path> --repo <PATH> [--json]`

Returns recommended validation commands for changed files.

Current behavior:

- Uses changed-file symbol definitions to find nearby targeted integration tests.
- Emits only runnable top-level integration test commands (`cargo test --test <name>`).
- Always includes a full-suite safety gate (`cargo test`).

## How It Works

At a high level:

1. Walk repository files while honoring standard ignore rules.
2. Compute and store per-file hashes.
3. Re-index only files whose hash changed.
4. Extract:
   - text token occurrences for all files,
   - Rust AST definitions/references for `.rs` files.
5. Query SQLite tables with deterministic ordering.

See detailed docs:

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/cli-reference.md`](docs/cli-reference.md)
- [`docs/json-output.md`](docs/json-output.md)
- [`docs/dogfood-log.md`](docs/dogfood-log.md)
- [`docs/performance-baseline.md`](docs/performance-baseline.md)

## Error Recovery

If the index database is corrupted or not a valid SQLite file, `repo-scout` prints a recovery hint with the exact path and instructs you to delete the file, then rerun `index`.

## Testing

Run all tests:

```bash
cargo test
```

The suite includes milestone-based integration tests for:

- command surface and schema bootstrap,
- incremental indexing,
- Rust AST extraction,
- deterministic JSON/ranking,
- end-to-end flow and corruption recovery.

## Dogfood Operating Procedure

`repo-scout` should be used to build `repo-scout`. For every feature or bugfix in this repository, run a dogfood loop before and after edits.

Pre-edit loop:

```bash
cargo run -- index --repo .
cargo run -- find <symbol> --repo . --json
cargo run -- refs <symbol> --repo . --json
```

Post-edit loop:

```bash
cargo run -- index --repo .
cargo run -- find <symbol> --repo .
cargo run -- refs <symbol> --repo .
cargo test
```

Rules:

- If dogfooding exposes incorrect behavior (stale results, missing results, noisy ranking, unstable JSON), add a failing integration test first and then fix it with strict red-green-refactor.
- Record at least one dogfood transcript in PR notes or in planning artifacts for each milestone.
- Do not mark a milestone complete unless dogfood commands succeed and all tests pass.

## Justfile Shortcuts

Common workflows are available through `just`:

```bash
just dogfood-pre launch
just dogfood-post launch
just tdd-red milestone6_delete_prunes_rows
just tdd-green milestone6_delete_prunes_rows
just tdd-refactor
just perf-baseline launch
```
