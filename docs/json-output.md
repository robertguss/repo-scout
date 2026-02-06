# JSON Output

`find` and `refs` support `--json` today. Phase 2 introduces additional JSON
commands, and this document freezes the planned contracts before implementation.

## Current Contract (Schema Version 1)

Example:

```bash
cargo run -- find orbit --repo . --json
```

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

## Phase 2 Planned Contract (Frozen Pre-Implementation)

Status: This section is a planning contract and does not imply features already
exist. Implementation must match these shapes or explicitly revise this doc.

Planned schema version: `2`

### `impact --json`

Example payload shape:

```json
{
  "schema_version": 2,
  "command": "impact",
  "query": "launch",
  "results": [
    {
      "symbol": "start_engine",
      "kind": "function",
      "file_path": "src/runtime.rs",
      "line": 42,
      "column": 5,
      "distance": 1,
      "relationship": "called_by",
      "confidence": "graph_likely",
      "score": 0.91
    }
  ]
}
```

### `context --json`

Example payload shape:

```json
{
  "schema_version": 2,
  "command": "context",
  "task": "modify launch flow and update call sites",
  "budget": 1200,
  "results": [
    {
      "file_path": "src/runtime.rs",
      "start_line": 30,
      "end_line": 70,
      "symbol": "launch",
      "kind": "function",
      "why_included": "direct definition match for task keyword 'launch'",
      "confidence": "context_high",
      "score": 0.95
    }
  ]
}
```

### `tests-for --json`

Example payload shape:

```json
{
  "schema_version": 2,
  "command": "tests-for",
  "query": "launch",
  "results": [
    {
      "target": "tests/launch_flow.rs",
      "target_kind": "integration_test_file",
      "why_included": "references launch in nearby module",
      "confidence": "graph_likely",
      "score": 0.83
    }
  ]
}
```

### `verify-plan --json`

Example payload shape:

```json
{
  "schema_version": 2,
  "command": "verify-plan",
  "changed_files": ["src/query/mod.rs"],
  "results": [
    {
      "step": "cargo test milestone9_ -- --nocapture",
      "scope": "targeted",
      "why_included": "changed file participates in impact/context query routing",
      "confidence": "context_medium",
      "score": 0.86
    },
    {
      "step": "cargo test",
      "scope": "full_suite",
      "why_included": "required safety gate after refactor",
      "confidence": "context_high",
      "score": 1.0
    }
  ]
}
```

## Determinism Guarantees

For identical index and query state, JSON output should be deterministic:

- stable field names and ordering,
- deterministic SQL ordering and tie-break rules,
- repository-relative paths,
- finite, documented vocabulary for `why_*`, `relationship`, and `confidence`.

Determinism is required for scripting, regression tests, and coding-agent workflows.
