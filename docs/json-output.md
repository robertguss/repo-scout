# JSON Output Contracts

`repo-scout` supports JSON output for query commands.

- `find --json`
- `refs --json`
- `impact --json`
- `context --json`
- `tests-for --json`
- `verify-plan --json`

`index` and `status` are terminal-only today.

## Schema Versions

- `find`, `refs`: `schema_version = 1`
- `impact`, `context`, `tests-for`, `verify-plan`: `schema_version = 2`

## `find --json` and `refs --json` (Schema 1)

Top-level shape:

```json
{
  "schema_version": 1,
  "command": "find",
  "query": "run",
  "results": [
    {
      "file_path": "src/main.rs",
      "line": 50,
      "column": 4,
      "symbol": "run",
      "why_matched": "ast_definition",
      "confidence": "ast_exact",
      "score": 1.0
    }
  ]
}
```

Top-level fields:

- `schema_version` (`number`)
- `command` (`"find" | "refs"`)
- `query` (`string`)
- `results` (`array<QueryMatch>`)

`QueryMatch` fields:

- `file_path` (`string`, repo-relative)
- `line` (`number`, 1-based)
- `column` (`number`, 1-based)
- `symbol` (`string`)
- `why_matched` (`string`)
- `confidence` (`string`)
- `score` (`number`)

Observed `why_matched` vocabulary:

- `ast_definition`
- `ast_reference`
- `exact_symbol_name`
- `text_substring_match`

Observed `confidence` vocabulary:

- `ast_exact`
- `ast_likely`
- `text_fallback`

## `impact --json` (Schema 2)

```json
{
  "schema_version": 2,
  "command": "impact",
  "query": "run",
  "results": [
    {
      "symbol": "main",
      "kind": "function",
      "file_path": "src/main.rs",
      "line": 28,
      "column": 4,
      "distance": 1,
      "relationship": "called_by",
      "confidence": "graph_likely",
      "score": 0.95
    }
  ]
}
```

Per-result fields:

- `symbol` (`string`)
- `kind` (`string`)
- `file_path` (`string`)
- `line` (`number`)
- `column` (`number`)
- `distance` (`number`, currently `1`)
- `relationship` (`called_by | contained_by | imported_by | implemented_by | <edge_kind>`)
- `confidence` (`graph_likely`)
- `score` (`number`)

## `context --json` (Schema 2)

```json
{
  "schema_version": 2,
  "command": "context",
  "task": "update run and verify refs behavior",
  "budget": 400,
  "results": [
    {
      "file_path": "src/main.rs",
      "start_line": 50,
      "end_line": 50,
      "symbol": "run",
      "kind": "function",
      "why_included": "direct definition match for task keyword 'run'",
      "confidence": "context_high",
      "score": 0.95
    }
  ]
}
```

Per-result fields:

- `file_path` (`string`)
- `start_line` (`number`)
- `end_line` (`number`)
- `symbol` (`string`)
- `kind` (`string`)
- `why_included` (`string`)
- `confidence` (`context_high | context_medium`)
- `score` (`number`)

## `tests-for --json` (Schema 2)

```json
{
  "schema_version": 2,
  "command": "tests-for",
  "query": "compute_plan",
  "results": [
    {
      "target": "tests/plan_test.rs",
      "target_kind": "integration_test_file",
      "why_included": "direct symbol match for 'compute_plan' in test file",
      "confidence": "graph_likely",
      "score": 0.9
    }
  ]
}
```

Per-result fields:

- `target` (`string`)
- `target_kind` (`string`)
- `why_included` (`string`)
- `confidence` (`graph_likely | context_medium`)
- `score` (`number`)

## `verify-plan --json` (Schema 2)

```json
{
  "schema_version": 2,
  "command": "verify-plan",
  "changed_files": ["src/query/mod.rs"],
  "results": [
    {
      "step": "cargo test --test milestone8_graph",
      "scope": "targeted",
      "why_included": "targeted test references changed symbol 'Path'",
      "confidence": "graph_likely",
      "score": 0.9
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

Top-level fields:

- `schema_version` (`2`)
- `command` (`"verify-plan"`)
- `changed_files` (`array<string>`; normalized + deduplicated)
- `results` (`array<VerificationStep>`)

`VerificationStep` fields:

- `step` (`string`)
- `scope` (`"targeted" | "full_suite"`)
- `why_included` (`string`)
- `confidence` (`string`)
- `score` (`number`)

## Determinism Expectations

For the same indexed state and same inputs:

- field names stay stable,
- key ordering is stable,
- result ordering is deterministic,
- file paths are repository-relative.
