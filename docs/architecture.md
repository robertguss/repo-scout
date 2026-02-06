# Architecture

This document describes the current `repo-scout` architecture after Phase 2.

## High-Level Flow

1. CLI parses a command in `src/cli.rs`.
2. Store bootstrap (`src/store/mod.rs`) opens `<repo>/.repo-scout/index.db`, ensures schema, and reads schema version.
3. `index` performs incremental indexing (`src/indexer/mod.rs`).
4. Query commands read SQLite tables and apply deterministic ranking/ordering (`src/query/mod.rs`).
5. Output is rendered as terminal text or JSON (`src/output.rs`).

## Module Responsibilities

- `src/main.rs`
  - Command dispatch and command handlers.
- `src/cli.rs`
  - Clap definitions for command/argument surface.
- `src/store/mod.rs`
  - Index path resolution, DB bootstrap, corruption hinting.
- `src/store/schema.rs`
  - Schema DDL and schema version metadata (`SCHEMA_VERSION = 2`).
- `src/indexer/files.rs`
  - Repository discovery and ignore-aware file walking.
- `src/indexer/text.rs`
  - Token occurrence extraction with line/column.
- `src/indexer/rust_ast.rs`
  - Rust AST extraction for definitions/references.
- `src/indexer/mod.rs`
  - Incremental indexing coordinator, stale-row pruning, symbol/edge persistence.
- `src/query/mod.rs`
  - `find`, `refs`, `impact`, `context`, `tests-for`, `verify-plan` implementations.
- `src/output.rs`
  - Human-readable and JSON serialization paths.

## Storage Model

Primary tables:

- `meta`
  - Key/value metadata including `schema_version`.
- `indexed_files`
  - Per-file hash used for incremental skip decisions.
- `text_occurrences`
  - Token fallback source for text matching and test heuristics.
- `ast_definitions`
  - Rust definition entries (`find` primary path).
- `ast_references`
  - Rust reference entries (`refs` primary path).
- `symbols_v2`
  - Rich symbol metadata (kind/container/span/signature).
- `symbol_edges_v2`
  - Symbol graph edges (`calls`, `contains`, `imports`, `implements`).

## Incremental Indexing Lifecycle

For each indexing run:

1. Discover source files and build live path set.
2. Prune stale DB rows for deleted files.
3. For each live file:
   - compute hash,
   - skip unchanged files,
   - otherwise delete existing rows for that file and reinsert fresh text/AST/symbol/edge rows in one transaction.
4. Upsert file hash into `indexed_files`.

Lifecycle guarantees covered by integration tests include stale-file pruning, rename handling, and schema migration safety.

## Query Strategies

### `find`

- Prefer exact AST definitions.
- Fall back to text exact token, then text substring.

### `refs`

- Prefer exact AST references.
- Fall back to text exact token, then text substring.

### `impact`

- Resolve all matching symbols in `symbols_v2`.
- Walk incoming edges in `symbol_edges_v2`.
- Emit one-hop impacted neighbors with normalized relationship labels.

### `context`

- Extract task keywords (ASCII alnum + `_`, lowercased, deduped, min length 3).
- Add direct symbol definition hits.
- Expand one-hop neighbors from edges.
- Sort deterministically and truncate to `max(1, budget / 200)`.

### `tests-for`

- Find direct symbol occurrences in test-like files.
- Group by file path, score by hit count, return deterministic ordering.

### `verify-plan`

- Normalize and dedupe `--changed-file` inputs.
- Suggest targeted test commands from:
  - changed file itself when it is a runnable test target,
  - tests referencing symbols defined in changed files.
- Keep best evidence for duplicate commands.
- Always append `cargo test` full-suite gate.

## Determinism

Determinism is enforced by:

- explicit SQL ordering and tie-breakers,
- stable confidence/label vocabularies,
- repository-relative path normalization,
- fixed JSON field shapes.

## Corruption Recovery

Store bootstrap maps SQLite corruption signatures (`DatabaseCorrupt`, `NotADatabase`) into an actionable error telling users to delete the index DB file and rerun `index`.
