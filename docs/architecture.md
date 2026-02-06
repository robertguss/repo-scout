# Architecture

This document describes the current architecture for `repo-scout` after Phase 2 Milestones 6 and 7.

## High-Level Flow

1. CLI parses a command (`index`, `status`, `find`, `refs`).
2. Store bootstrap ensures `.repo-scout/index.db` exists and schema is initialized.
3. `index` performs file discovery + incremental processing.
4. `find`/`refs` query SQLite with deterministic ordering.
5. Output is rendered as human-readable text or JSON.

## Module Map

- `src/main.rs`
  - Command dispatch and top-level orchestration.
- `src/cli.rs`
  - clap command/argument definitions.
- `src/store/mod.rs`
  - Index database path resolution and bootstrap.
  - Corruption detection and user-facing recovery hinting.
- `src/store/schema.rs`
  - SQLite schema creation and schema version metadata.
- `src/indexer/files.rs`
  - Repository walk and per-file hashing.
- `src/indexer/text.rs`
  - Language-agnostic token extraction with line/column locations.
- `src/indexer/rust_ast.rs`
  - Rust Tree-sitter parsing for symbol definitions (functions, types, modules, imports) and call references.
- `src/indexer/mod.rs`
  - Incremental indexing coordinator and table upserts.
- `src/query/mod.rs`
  - Query retrieval, ranking, and result labeling.
- `src/output.rs`
  - Terminal and JSON formatting.

## Storage Model (SQLite)

Current tables:

- `meta`
  - Key/value metadata (`schema_version`).
- `indexed_files`
  - `file_path` + `content_hash` for incremental re-index checks.
- `text_occurrences`
  - Token-level fallback matches with `line` and `column`.
- `ast_definitions`
  - Rust AST definition entries used by `find`.
- `ast_references`
  - Rust AST reference entries (call-site identifiers).
- `symbols_v2`
  - Rich symbol metadata for Phase 2: kind, container, start/end span, and optional signature summary.
- `symbol_edges_v2`
  - Phase 2 graph table for symbol-to-symbol edges (`calls`, `contains`, `imports`, `implements`).

Indexes exist for common symbol lookups in text and AST tables.

## Incremental Indexing

For each discovered file:

1. Compute BLAKE3 hash.
2. Compare with `indexed_files.content_hash`.
3. If unchanged: increment `skipped_files`.
4. If changed:
   - delete old occurrences for that file,
   - insert fresh text occurrences,
   - insert Rust AST entries when file extension is `.rs`,
   - upsert the new file hash.

Each file update is transactional.

## Query Strategy

### `find`

1. Return AST definitions when present (`why_matched=ast_definition`, `confidence=ast_exact`).
2. Else return ranked text fallback:
   - exact token matches first (`exact_symbol_name`, higher score),
   - substring token matches next (`text_substring_match`, lower score).

### `refs`

1. Return AST references when present (`why_matched=ast_reference`, `confidence=ast_likely`).
2. Else use the same ranked text fallback as `find`.

## Determinism

Output determinism is enforced through:

- repository-relative paths,
- explicit SQL `ORDER BY` clauses,
- stable JSON field shapes.

## Corruption Recovery

Store bootstrap maps SQLite corruption signatures (`DatabaseCorrupt`, `NotADatabase`) to an actionable error message with the index path and delete-and-rerun guidance.
