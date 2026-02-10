# Architecture

This document describes the current `repo-scout` architecture after Phase 14.

## High-Level Flow

1. CLI parses a command in `src/cli.rs`.
2. Store bootstrap (`src/store/mod.rs`) opens `<repo>/.repo-scout/index.db`, ensures schema, and
   reads schema version.
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
  - Language adapters (`rust`, `typescript`, `python`, `go`) and normalized extraction contracts.
  - Rust adapter now performs deterministic module-qualified candidate resolution across
    `crate::`/`self::`/`super::` prefixes and both `<module>.rs` + `<module>/mod.rs` layouts.
  - TypeScript/Python adapters now include module-aware alias hints for namespace/member and
    module-alias attribute call resolution, including TypeScript directory-import candidates for
    `index.ts`/`index.tsx` paths.
  - Go adapter now provides definition extraction plus AST-backed call references and deterministic
    import-alias-aware selector call-edge candidates.
- `src/indexer/mod.rs`
  - Incremental indexing coordinator, stale-row pruning, adapter dispatch, deferred edge resolution,
    and symbol/edge persistence.
- `src/query/mod.rs`
  - `find`, `refs`, `impact`, `context`, `tests-for`, `verify-plan`, `diff-impact`, and `explain`
    implementations.
- `src/output.rs`
  - Human-readable and JSON serialization paths (`diff-impact` terminal output is row-oriented in
    Phase 8).

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
   - otherwise delete existing rows for that file and reinsert fresh text/AST/symbol/edge rows in
     one transaction.
4. Resolve deferred cross-file edges after all files are indexed.
5. Upsert file hash into `indexed_files`.

Lifecycle guarantees covered by integration tests include stale-file pruning, rename handling, and
schema migration safety.

## Query Strategies

### `find`

- Prefer exact AST definitions.
- Fall back to text exact token, then text substring.
- Optional fallback-only scope controls:
  - `--code-only` keeps `.rs`, `.ts`, `.tsx`, `.py`, `.go` paths.
  - `--exclude-tests` drops test-like paths (`tests/`, `/tests/`, `*_test.rs`, `*_test.go`,
    `*.test.ts`, `*.test.tsx`, `*.spec.ts`, `*.spec.tsx`, `test_*.py`, `*_test.py`, `*_tests.py`).
- Fallback ties are path-class ranked (`code`, then `test-like`, then `docs/other`).
- Optional deterministic output cap: `--max-results <N>`.

### `refs`

- Prefer exact AST references.
- Go call identifiers/selector fields now contribute AST reference rows.
- Fall back to text exact token, then text substring.
- Uses the same fallback-only scope controls as `find` (`--code-only`, `--exclude-tests`).
- Uses the same fallback path-class tie-break and deterministic cap behavior as `find`
  (`--max-results`).

### `impact`

- Resolve all matching symbols in `symbols_v2`.
- Walk incoming edges in `symbol_edges_v2`.
- Emit one-hop impacted neighbors with normalized relationship labels.
- Apply deterministic semantic score calibration by relationship/provenance.

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
- Runner-aware command synthesis is strict:
  - Go `_test.go` targets are runnable via deterministic package commands (`go test ./<dir>` or
    `go test .` for root package tests).
  - Python targets are runnable only in explicit pytest contexts.
  - TypeScript targets are runnable only when `package.json` unambiguously signals one Node runner
    (`jest` or `vitest`).
- Score and sort deterministically with runnable targets first.

### `verify-plan`

- Normalize and dedupe `--changed-file` inputs.
- Parse and normalize repeatable `--changed-line path:start[:end]`.
- Apply additive repeatable `--changed-symbol` filters.
- Skip generic changed symbols to reduce low-signal targeted recommendations.
- Suggest targeted test commands from:
  - changed file itself when it is a runnable test target,
  - tests referencing symbols defined in changed files.
- Apply deterministic targeted capping (`DEFAULT_VERIFY_PLAN_MAX_TARGETED = 8` or `--max-targeted`
  override).
- Preserve changed runnable test targets regardless cap value.
- Keep best evidence for duplicate commands.
- Append deterministic full-suite gate by runner context (`cargo test` by default; `go test ./...`
  in Go-only scopes; `pytest` in explicit Python runner contexts for Python-only scope; `npx vitest run`
  / `npx jest` in explicit unambiguous TypeScript-only Node runner contexts).

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
- Resolve Rust module-qualified calls (`crate::`, `self::`, `super::`, module prefixes) with
  deterministic file-path candidates for both `<module>.rs` and `<module>/mod.rs`.
- Resolve TypeScript namespace/member and Python module-alias attribute calls with module-aware
  hints so duplicate-name callees do not cross-link ambiguously.
- Resolve TypeScript directory imports (`./module`) with deterministic direct + `index.ts`/`index.tsx`
  candidate paths so caller attribution is preserved in `diff-impact`.
- Resolve Python relative-import identifier calls (`from .module import symbol`) to preserve
  caller attribution in changed-file impact walks.
- Resolve Go import-alias selector calls with deterministic import-path candidate files so duplicate
  function names across packages remain attributable.
- Optionally remove changed-symbol output rows with `--exclude-changed`.
- Optionally cap results deterministically with `--max-results`.
- Attach ranked test targets by default (`include_tests = true`), disable with `--exclude-tests`, or
  keep explicit default behavior with `--include-tests` (conflicting flags are rejected by clap).
- Apply deterministic semantic score calibration so resolved semantic caller rows rank above
  fallback test-target rows.
- Sort mixed result kinds deterministically.
- Render terminal output as deterministic row-level `impacted_symbol`/`test_target` lines.

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

## Quality Gates

Phase 8 requires both quality gates to stay green for release-readiness:

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

## Corruption Recovery

Store bootstrap maps SQLite corruption signatures (`DatabaseCorrupt`, `NotADatabase`) into an
actionable error telling users to delete the index DB file and rerun `index`.
