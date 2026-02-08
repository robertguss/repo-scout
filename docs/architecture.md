# Architecture

This document describes the current `repo-scout` architecture after Phase 6.

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
  - Schema DDL and schema version metadata (`SCHEMA_VERSION = 3`).
- `src/indexer/files.rs`
  - Repository discovery and ignore-aware file walking.
- `src/indexer/text.rs`
  - Token occurrence extraction with line/column.
- `src/indexer/rust_ast.rs`
  - Rust AST extraction for definitions/references.
- `src/indexer/languages/`
  - Language adapters (`rust`, `typescript`, `python`) and normalized extraction contracts.
- `src/indexer/mod.rs`
  - Incremental indexing coordinator, stale-row pruning, adapter dispatch, deferred edge resolution,
    and symbol/edge persistence.
- `src/query/mod.rs`
  - `find`, `refs`, `impact`, `context`, `tests-for`, `verify-plan`, `diff-impact`, and `explain`
    implementations.
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
  - AST-backed definition entries (`find` primary path).
- `ast_references`
  - AST-backed reference entries (`refs` primary path).
- `symbols_v2`
  - Rich symbol metadata (kind/language/qualified_symbol/container/span/signature).
- `symbol_edges_v2`
  - Symbol graph edges (`calls`, `contains`, `imports`, `implements`) with provenance metadata.

Edge endpoint resolution is now `SymbolKey`-aware and deterministic:

1. exact `qualified_symbol`,
2. exact `(file_path, symbol)` with non-import preference,
3. unique global `symbol` match (optionally language-scoped),
4. unresolved edges are skipped (fail-safe) rather than arbitrarily linked.

## Incremental Indexing Lifecycle

For each indexing run:

1. Discover source files and build live path set.
2. Prune stale DB rows for deleted files.
3. For each live file:
   - compute hash,
   - skip unchanged files,
   - otherwise delete existing rows for that file and reinsert fresh text/AST/symbol/edge rows in one transaction.
4. Resolve deferred cross-file edges after all files are indexed.
5. Upsert file hash into `indexed_files`.

Lifecycle guarantees covered by integration tests include stale-file pruning, rename handling, and schema migration safety.

## Query Strategies

### `find`

- Prefer exact AST definitions.
- Fall back to text exact token, then text substring.
- Optional fallback-only scope controls:
  - `--code-only` keeps `.rs`, `.ts`, `.tsx`, `.py` paths.
  - `--exclude-tests` drops test-like paths (`tests/`, `/tests/`, `*_test.rs`).
- Fallback ties are path-class ranked (`code`, then `test-like`, then `docs/other`).
- Optional deterministic output cap: `--max-results <N>`.

### `refs`

- Prefer exact AST references.
- Fall back to text exact token, then text substring.
- Uses the same fallback-only scope controls as `find` (`--code-only`, `--exclude-tests`).
- Uses the same fallback path-class tie-break and deterministic cap behavior as `find`
  (`--max-results`).

### `impact`

- Resolve all matching symbols in `symbols_v2`.
- Walk incoming edges in `symbol_edges_v2`.
- Emit one-hop impacted neighbors with normalized relationship labels.

### `context`

- Extract normalized task keywords (ASCII alnum + `_`, lowercased, deduped, stopword-filtered).
- Score direct symbol definition hits using deterministic token-overlap relevance.
- Expand one-hop neighbors from edges with deterministic neighbor scoring.
- Optionally filter rows with `--code-only` and `--exclude-tests`.
- Sort deterministically and truncate to `max(1, budget / 200)`.

### `tests-for`

- Find direct symbol occurrences in test-like files.
- Classify targets as runnable integration test files or support paths.
- By default return runnable targets only; `--include-support` restores support paths additively.
- Score and sort deterministically with runnable targets first.

### `verify-plan`

- Normalize and dedupe `--changed-file` inputs.
- Parse and normalize repeatable `--changed-line path:start[:end]`.
- Apply additive repeatable `--changed-symbol` filters.
- Skip generic changed symbols to reduce low-signal targeted recommendations.
- Suggest targeted test commands from:
  - changed file itself when it is a runnable test target,
  - tests referencing symbols defined in changed files.
- Apply deterministic targeted capping (`DEFAULT_VERIFY_PLAN_MAX_TARGETED = 8` or
  `--max-targeted` override).
- Preserve changed runnable test targets regardless cap value.
- Keep best evidence for duplicate commands.
- Always append `cargo test` full-suite gate.

### `diff-impact`

- Normalize + dedupe changed-file inputs.
- Emit changed symbols from `symbols_v2`.
- Exclude `kind=import` changed-symbol seeds by default.
- Re-include import seeds only when `--include-imports` is set.
- Optionally constrain changed-symbol seeds by `--changed-line path:start[:end]` overlap.
- Optionally constrain changed-symbol seeds by repeatable `--changed-symbol`.
- Expand bounded multi-hop incoming neighbors from `symbol_edges_v2` up to `--max-distance`.
- Use per-seed minimum-distance tracking and changed-seed suppression to avoid cycle-driven
  duplicate growth.
- Optionally remove changed-symbol output rows with `--exclude-changed`.
- Optionally cap results deterministically with `--max-results`.
- Attach ranked test targets (`include_tests = true`; `--include-tests` retained for CLI
  compatibility).
- Sort mixed result kinds deterministically.

### `explain`

- Resolve exact symbol definitions from `symbols_v2`.
- Attach deterministic inbound/outbound edge counters from `symbol_edges_v2`.
- Optionally attach source snippets resolved from the indexed repository root.

## Determinism

Determinism is enforced by:

- explicit SQL ordering and tie-breakers,
- stable confidence/label vocabularies,
- repository-relative path normalization,
- fixed JSON field shapes,
- deterministic handler-stage truncation for capped commands (`find`, `refs`, `diff-impact`).

## Corruption Recovery

Store bootstrap maps SQLite corruption signatures (`DatabaseCorrupt`, `NotADatabase`) into an actionable error telling users to delete the index DB file and rerun `index`.
