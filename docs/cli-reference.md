# CLI Reference

All examples assume invocation through Cargo:

```bash
cargo run -- <command> ...
```

## Global Help

```bash
cargo run -- --help
```

Current command surface:

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

## Commands

### `index --repo <PATH>`

Build or refresh `<PATH>/.repo-scout/index.db`.

Example:

```bash
cargo run -- index --repo .
```

Terminal output fields:

- `index_path`
- `schema_version`
- `indexed_files`
- `skipped_files`

Notes:

- `indexed_files` counts changed/new files processed in this run.
- `skipped_files` counts unchanged files skipped by content hash.

### `status --repo <PATH>`

Show index location and schema version.

Example:

```bash
cargo run -- status --repo .
```

### `find <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests]`

Find likely symbol definitions.

Ranking strategy:

1. AST definition matches (`why_matched=ast_definition`, `confidence=ast_exact`, `score=1.0`).
2. Text exact token fallback (`exact_symbol_name`, `text_fallback`, `0.8`).
3. Text substring fallback (`text_substring_match`, `text_fallback`, `0.4`).

Scope controls for fallback rows:

- `--code-only`: restricts fallback matches to `.rs`, `.ts`, `.tsx`, `.py` paths.
- `--exclude-tests`: omits fallback matches in test-like paths (`tests/`, `/tests/`, `*_test.rs`).

AST definition matches remain highest priority and are returned unchanged when present.

Example:

```bash
cargo run -- find run --repo .
cargo run -- find run --repo . --json
```

### `refs <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests]`

Find likely references/usages.

Ranking strategy:

1. AST reference matches (`why_matched=ast_reference`, `confidence=ast_likely`, `score=0.95`).
2. Same text fallback sequence as `find`.

Scope controls for fallback rows:

- `--code-only`: restricts fallback matches to `.rs`, `.ts`, `.tsx`, `.py` paths.
- `--exclude-tests`: omits fallback matches in test-like paths (`tests/`, `/tests/`, `*_test.rs`).

AST reference matches remain highest priority and are returned unchanged when present.

Example:

```bash
cargo run -- refs run --repo .
cargo run -- refs run --repo . --json
```

### `impact <SYMBOL> --repo <PATH> [--json]`

Return one-hop incoming graph neighbors likely impacted by changing `SYMBOL`.

Relationship labels are normalized to:

- `called_by`
- `contained_by`
- `imported_by`
- `implemented_by`

Example:

```bash
cargo run -- impact run --repo .
cargo run -- impact run --repo . --json
```

### `context --task <TEXT> --repo <PATH> [--budget <N>] [--json]`

Build a ranked, budgeted context bundle for an editing task.

Behavior:

- Extracts normalized task keywords (lowercased, deduped, stopword-filtered).
- Uses deterministic token-overlap relevance between task keywords and symbol tokens.
- Prioritizes direct symbol definition hits (typically `context_high`, score up to `0.98`).
- Adds one-hop graph neighbors (`context_medium`, score derived from direct match score).
- Truncates to `max(1, budget / 200)` results.

Example:

```bash
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400 --json
```

### `tests-for <SYMBOL> --repo <PATH> [--include-support] [--json]`

Return test targets likely relevant to `SYMBOL`.

Current target discovery:

- file path under `tests/`
- file path containing `/tests/`
- file name matching `*_test.rs`

Output rows include:

- `target`
- `target_kind` (`integration_test_file` or additive `support_test_file`)
- `why_included`
- `confidence`
- `score`

Default behavior returns runnable integration targets only. Set `--include-support` to restore
support paths (for example `tests/common/mod.rs`) in deterministic ranked order.

Example:

```bash
cargo run -- tests-for run --repo .
cargo run -- tests-for run --repo . --json
cargo run -- tests-for run --repo . --include-support --json
```

### `verify-plan --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--max-targeted <N>] [--json]`

Generate deterministic validation steps for changed files.

Behavior:

- Normalizes changed-file paths to repo-relative form.
- Deduplicates repeated changed-file inputs.
- Dampens high-frequency generic changed symbols (for example `Path`, `output`) for better signal.
- Suggests runnable targeted commands only (`cargo test --test <name>` for direct `tests/<file>.rs` targets).
- Caps symbol-derived targeted rows to `8` by default.
- `--max-targeted 0` suppresses symbol-derived targeted rows, while still preserving changed
  runnable test targets and the required full-suite gate.
- Always appends a full-suite safety gate: `cargo test`.

Example:

```bash
cargo run -- verify-plan --changed-file src/query/mod.rs --repo .
cargo run -- verify-plan --changed-file src/query/mod.rs --changed-file ./src/query/mod.rs --repo . --json
cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
```

### `diff-impact --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--max-distance <N>] [--include-tests] [--include-imports] [--changed-line <path:start[:end]>] [--json]`

Generate deterministic changed-file impact results.

Behavior:

- Normalizes and deduplicates changed-file paths.
- Emits changed symbols (`distance = 0`, `relationship = changed_symbol`).
- Excludes `kind=import` from changed-symbol seeds unless `--include-imports` is set.
- Applies `--changed-line` filters to changed-symbol seeds for matching files only.
- Emits bounded multi-hop incoming neighbors (`called_by`, `contained_by`, `imported_by`,
  `implemented_by`) up to `--max-distance`.
- Uses cycle-safe, deterministic dedupe to prevent duplicate growth and changed-symbol echo rows at
  non-zero distances.
- Optionally emits test targets (`result_kind = test_target`).

`--changed-line` parsing rules:

- Format: `path:start[:end]`
- `start`/`end` are 1-based positive line numbers.
- `end` defaults to `start` when omitted.
- Invalid specs return an actionable error that includes the malformed token and expected format.

Examples:

```bash
cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
```

### `explain <SYMBOL> --repo <PATH> [--include-snippets] [--json]`

Return a deterministic symbol dossier.

Behavior:

- Resolves exact symbol definitions.
- Includes signature and span metadata.
- Includes inbound/outbound relationship counters.
- Optionally includes source snippets (`--include-snippets`).

Examples:

```bash
cargo run -- explain impact_matches --repo .
cargo run -- explain impact_matches --repo . --json
cargo run -- explain impact_matches --repo . --include-snippets --json
```

## Exit Codes

- Success: `0`
- Failure: non-zero

Corrupt or invalid index DB errors include a hint with the index path and recovery action (`delete file, rerun index`).
