# JSON Output

`find` and `refs` support `--json`.

Example:

```bash
cargo run -- find orbit --repo . --json
```

## Top-Level Schema

Current schema version: `1`

```json
{
  "schema_version": 1,
  "command": "find",
  "query": "orbit",
  "results": [
    {
      "file_path": "docs/rank.txt",
      "line": 1,
      "column": 1,
      "symbol": "orbit",
      "why_matched": "exact_symbol_name",
      "confidence": "text_fallback",
      "score": 0.8
    }
  ]
}
```

## Field Definitions

Top-level fields:

- `schema_version` (`number`): schema contract version.
- `command` (`string`): `find` or `refs`.
- `query` (`string`): symbol argument passed by the user.
- `results` (`array`): ordered match list.

Per-result fields:

- `file_path` (`string`): repository-relative file path.
- `line` (`number`): 1-based line.
- `column` (`number`): 1-based column.
- `symbol` (`string`): matched symbol/token text.
- `why_matched` (`string`): match provenance label.
- `confidence` (`string`): confidence tier.
- `score` (`number`): ranking score (higher is better).

## Determinism Guarantees

The project aims for deterministic JSON output for identical index/query state:

- stable field shapes,
- deterministic SQL ordering,
- repository-relative paths.

This is important for tooling, scripted checks, and coding-agent workflows.
