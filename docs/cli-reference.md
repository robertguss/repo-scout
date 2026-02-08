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

### `find <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests] [--max-results <N>]`

Find likely symbol definitions.

Ranking strategy:

1. AST definition matches (`why_matched=ast_definition`, `confidence=ast_exact`, `score=1.0`).
2. Text exact token fallback (`exact_symbol_name`, `text_fallback`, `0.8`).
3. Text substring fallback (`text_substring_match`, `text_fallback`, `0.4`).
4. At equal fallback score tiers, tie-break by path class (`code` before `test-like` before
   `docs/other`), then by file/position/symbol.

Scope controls for fallback rows:

- `--code-only`: restricts fallback matches to `.rs`, `.ts`, `.tsx`, `.py` paths.
- `--exclude-tests`: omits fallback matches in test-like paths (`tests/`, `/tests/`, `*_test.rs`).
- `--max-results <N>`: deterministic truncation after ranking (`0` yields empty results).

AST definition matches remain highest priority and are returned unchanged when present.

Example:

```bash
cargo run -- find run --repo .
cargo run -- find run --repo . --json
cargo run -- find run --repo . --code-only --exclude-tests --max-results 10 --json
```

### `refs <SYMBOL> --repo <PATH> [--json] [--code-only] [--exclude-tests] [--max-results <N>]`

Find likely references/usages.

Ranking strategy:

1. AST reference matches (`why_matched=ast_reference`, `confidence=ast_likely`, `score=0.95`).
2. Same text fallback sequence as `find`.
3. Same fallback path-class tie-break behavior as `find`.

Scope controls for fallback rows:

- `--code-only`: restricts fallback matches to `.rs`, `.ts`, `.tsx`, `.py` paths.
- `--exclude-tests`: omits fallback matches in test-like paths (`tests/`, `/tests/`, `*_test.rs`).
- `--max-results <N>`: deterministic truncation after ranking (`0` yields empty results).

AST reference matches remain highest priority and are returned unchanged when present.

Example:

```bash
cargo run -- refs run --repo .
cargo run -- refs run --repo . --json
cargo run -- refs run --repo . --code-only --exclude-tests --max-results 10 --json
```

### `impact <SYMBOL> --repo <PATH> [--json]`

Return one-hop incoming graph neighbors likely impacted by changing `SYMBOL`.

Relationship labels are normalized to:

- `called_by`
- `contained_by`
- `imported_by`
- `implemented_by`

Ranking notes:

- Semantic edge rows are deterministically calibrated by relationship/provenance.
- `called_by` rows from resolved call edges are ranked in a high-confidence band (for example
  `score ≈ 0.97` in Phase 8 fixture scenarios).
- Deterministic tie-breaks remain `file_path`, `line`, `column`, `symbol`, `relationship`.

Example:

```bash
cargo run -- impact run --repo .
cargo run -- impact run --repo . --json
```

### `context --task <TEXT> --repo <PATH> [--budget <N>] [--json] [--code-only] [--exclude-tests]`

Build a ranked, budgeted context bundle for an editing task.

Behavior:

- Extracts normalized task keywords (lowercased, deduped, stopword-filtered).
- Uses deterministic token-overlap relevance between task keywords and symbol tokens.
- Prioritizes direct symbol definition hits (typically `context_high`, score up to `0.98`).
- Adds one-hop graph neighbors (`context_medium`, score derived from direct match score).
- `--code-only` keeps only `.rs`, `.ts`, `.tsx`, `.py` paths.
- `--exclude-tests` removes test-like paths (`tests/`, `/tests/`, `*_test.rs`).
- Truncates to `max(1, budget / 200)` results.

Example:

```bash
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400 --json
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400 --code-only --exclude-tests --json
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

### `verify-plan --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--changed-line <path:start[:end]>] [--changed-symbol <symbol> ...] [--max-targeted <N>] [--json]`

Generate deterministic validation steps for changed files.

Behavior:

- Normalizes changed-file paths to repo-relative form.
- Deduplicates repeated changed-file inputs.
- Parses and normalizes repeatable `--changed-line` ranges (`path:start[:end]`).
- Applies repeatable `--changed-symbol` filters additively to changed-file symbol selection.
- Dampens high-frequency generic changed symbols (for example `Path`, `output`) for better signal.
- Suggests runnable targeted commands only (`cargo test --test <name>` for direct `tests/<file>.rs`
  targets).
- Caps symbol-derived targeted rows to `8` by default.
- `--max-targeted 0` suppresses symbol-derived targeted rows, while still preserving changed
  runnable test targets and the required full-suite gate.
- Always appends a full-suite safety gate: `cargo test`.

Example:

```bash
cargo run -- verify-plan --changed-file src/query/mod.rs --repo .
cargo run -- verify-plan --changed-file src/query/mod.rs --changed-file ./src/query/mod.rs --repo . --json
cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
```

### `diff-impact --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--max-distance <N>] [--exclude-tests|--include-tests] [--include-imports] [--changed-line <path:start[:end]>] [--changed-symbol <symbol> ...] [--exclude-changed] [--max-results <N>] [--json]`

Generate deterministic changed-file impact results.

Behavior:

- Normalizes and deduplicates changed-file paths.
- Emits changed symbols (`distance = 0`, `relationship = changed_symbol`).
- Excludes `kind=import` from changed-symbol seeds unless `--include-imports` is set.
- Applies `--changed-line` filters to changed-symbol seeds for matching files only.
- Applies repeatable `--changed-symbol` filters to changed-symbol seeds.
- Emits bounded multi-hop incoming neighbors (`called_by`, `contained_by`, `imported_by`,
  `implemented_by`) up to `--max-distance`.
- Uses module-aware TypeScript/Python call resolution so namespace/member and module-alias attribute
  calls resolve to the intended module under duplicate symbol names.
- Uses cycle-safe, deterministic dedupe to prevent duplicate growth and changed-symbol echo rows at
  non-zero distances.
- `--exclude-changed` removes changed-symbol (`distance=0`) rows from final output while traversal
  still uses those seeds.
- `--max-results <N>` truncates results deterministically after ranking.
- Emits test targets (`result_kind = test_target`) when available; default behavior remains
  `include_tests = true`.
- `--exclude-tests` suppresses test-target rows and flips schema-3 `include_tests` to `false`.
- `--include-tests` keeps explicit default behavior and conflicts with `--exclude-tests`.
- Semantic impacted-symbol rows use deterministic calibrated scoring (for example `call_resolution`
  `called_by` rows in Phase 8 fixtures score `0.97`) and rank ahead of fallback test-target rows.
- Terminal output is row-oriented: one deterministic `impacted_symbol ...` or `test_target ...` line
  per result with confidence/provenance/score fields.

`--changed-line` parsing rules:

- Format: `path:start[:end]`
- `start`/`end` are 1-based positive line numbers.
- `end` defaults to `start` when omitted.
- Invalid specs return an actionable error that includes the malformed token and expected format.

Examples:

```bash
cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
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

Corrupt or invalid index DB errors include a hint with the index path and recovery action
(`delete file, rerun index`).
