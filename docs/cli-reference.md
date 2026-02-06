# CLI Reference

All examples below assume invocation via Cargo:

```bash
cargo run -- <command> ...
```

If installed as a binary, replace `cargo run --` with your binary invocation.

## Commands

### `index --repo <PATH>`

Build or update the repository index in `<PATH>/.repo-scout/index.db`.

Example:

```bash
cargo run -- index --repo .
```

Example output:

```text
index_path: /absolute/path/to/repo/.repo-scout/index.db
schema_version: 2
indexed_files: 12
skipped_files: 48
```

### `status --repo <PATH>`

Show index location and schema version.

Example:

```bash
cargo run -- status --repo .
```

### `find <SYMBOL> --repo <PATH> [--json]`

Find likely definitions or matches for `SYMBOL`.

Behavior:

- Prefers Rust AST definitions if available (functions, methods, types, modules, consts, and imports).
- Falls back to ranked text matching (exact token first, substring second).

Example:

```bash
cargo run -- find launch --repo .
```

### `refs <SYMBOL> --repo <PATH> [--json]`

Find likely references for `SYMBOL`.

Behavior:

- Prefers Rust AST references if available.
- Falls back to ranked text matching.

Example:

```bash
cargo run -- refs launch --repo .
```

## Output Labels

`why_matched` values currently used:

- `ast_definition`
- `ast_reference`
- `exact_symbol_name`
- `text_substring_match`

`confidence` values currently used:

- `ast_exact`
- `ast_likely`
- `text_fallback`

## Exit Behavior

- Success paths return exit code `0`.
- Errors return non-zero and print a message to stderr.
- Corrupt index errors include a recovery hint that points to the index path and suggests deleting it before re-running `index`.
