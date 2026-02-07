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

### `find <SYMBOL> --repo <PATH> [--json]`

Find likely symbol definitions.

Ranking strategy:

1. AST definition matches (`why_matched=ast_definition`, `confidence=ast_exact`, `score=1.0`).
2. Text exact token fallback (`exact_symbol_name`, `text_fallback`, `0.8`).
3. Text substring fallback (`text_substring_match`, `text_fallback`, `0.4`).

Example:

```bash
cargo run -- find run --repo .
cargo run -- find run --repo . --json
```

### `refs <SYMBOL> --repo <PATH> [--json]`

Find likely references/usages.

Ranking strategy:

1. AST reference matches (`why_matched=ast_reference`, `confidence=ast_likely`, `score=0.95`).
2. Same text fallback sequence as `find`.

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

- Extracts deduplicated lowercase keywords from task text.
- Prioritizes direct symbol definition hits (`confidence=context_high`, `score=0.95`).
- Adds one-hop graph neighbors (`context_medium`, `0.7`).
- Truncates to `max(1, budget / 200)` results.

Example:

```bash
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400
cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400 --json
```

### `tests-for <SYMBOL> --repo <PATH> [--json]`

Return test targets likely relevant to `SYMBOL`.

Current target discovery:

- file path under `tests/`
- file path containing `/tests/`
- file name matching `*_test.rs`

Output rows include:

- `target`
- `target_kind` (currently `integration_test_file`)
- `why_included`
- `confidence`
- `score`

Example:

```bash
cargo run -- tests-for run --repo .
cargo run -- tests-for run --repo . --json
```

### `verify-plan --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--json]`

Generate deterministic validation steps for changed files.

Behavior:

- Normalizes changed-file paths to repo-relative form.
- Deduplicates repeated changed-file inputs.
- Suggests runnable targeted commands only (`cargo test --test <name>` for direct `tests/<file>.rs` targets).
- Always appends a full-suite safety gate: `cargo test`.

Example:

```bash
cargo run -- verify-plan --changed-file src/query/mod.rs --repo .
cargo run -- verify-plan --changed-file src/query/mod.rs --changed-file ./src/query/mod.rs --repo . --json
```

### `diff-impact --changed-file <PATH> [--changed-file <PATH> ...] --repo <PATH> [--max-distance <N>] [--include-tests] [--json]`

Generate deterministic changed-file impact results.

Behavior:

- Normalizes and deduplicates changed-file paths.
- Emits changed symbols (`distance = 0`, `relationship = changed_symbol`).
- Emits one-hop incoming neighbors (`called_by`, `contained_by`, `imported_by`, `implemented_by`)
  when `max_distance >= 1`.
- Optionally emits test targets (`result_kind = test_target`).

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
