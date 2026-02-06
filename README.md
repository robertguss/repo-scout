# repo-scout

`repo-scout` is a local CLI for indexing source repositories and querying symbol matches quickly.

This project currently ships a hybrid approach:

- Language-agnostic text indexing for all files.
- Rust AST extraction (Tree-sitter) for function definitions and call references.
- Deterministic terminal and JSON output for scripting and agent use.

## Status

This is an actively evolving v0. The core workflow is implemented and tested:

- `index`: build or update a local index.
- `status`: show index location and schema version.
- `find`: search for likely definitions.
- `refs`: search for likely references.

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

- Prefer Rust AST definitions (`ast_definition`) when present.
- Otherwise use text fallback ranking:
  - exact token matches first (`exact_symbol_name`)
  - substring matches next (`text_substring_match`)

### `refs <symbol>`

Returns likely references for `symbol`.

Current behavior:

- Prefer Rust AST call references (`ast_reference`) when present.
- Otherwise use the same text fallback ranking as `find`.

### `--json`

Supported on `find` and `refs`. Emits deterministic JSON with `schema_version`, `command`, `query`, and `results`.

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
