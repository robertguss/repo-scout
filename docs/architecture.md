# Architecture

## High-level flow

1. File walker discovers repository files.
2. Text index records searchable token occurrences.
3. Language adapters extract AST-backed symbols and relationships.
4. Query layer resolves navigation and analysis commands.
5. SQLite store provides deterministic persistence and retrieval.

## Repository layout

- CLI and command routing: `src/main.rs`, `src/cli.rs`
- Indexing: `src/indexer/`
- Query logic: `src/query/`
- Persistence and schema: `src/store/`
- Output formatting: `src/output.rs`
- Integration tests: `tests/`

## Data store

The index database lives at:

- `<target-repo>/.repo-scout/index.db`

This file is generated local state and should not be committed.

## Design properties

- deterministic ranking and output
- local-only operation (no remote service required)
- command surface suitable for humans and agents
