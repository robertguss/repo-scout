# JSON Output Contracts

`repo-scout` supports deterministic JSON output for all query commands.

Available today:
- `find --json`
- `refs --json`
- `impact --json`
- `context --json`
- `tests-for --json`
- `verify-plan --json`
- `diff-impact --json`
- `explain --json`

`index` and `status` are terminal-only today.

## Schema Versions

- `find`, `refs`: `schema_version = 1`
- `impact`, `context`, `tests-for`, `verify-plan`: `schema_version = 2`
- `diff-impact`, `explain`: `schema_version = 3` (implemented, contract frozen)

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

Schema 1 remains unchanged when using Phase 4/6 controls (`--code-only`, `--exclude-tests`,
`--max-results`).
These options only affect result selection/ranking order:
- scope flags filter text fallback rows only,
- fallback ties prefer code paths over test/docs at equal fallback score tiers,
- `--max-results` applies deterministic truncation after ranking,
while AST-priority rows and JSON envelope shape stay stable.

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
      "score": 0.97
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

Phase 7 keeps schema 2 unchanged and applies deterministic semantic score calibration by
relationship/provenance so stronger semantic rows (for example resolved `called_by`) stay in a
high-confidence ranking band.

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
      "why_included": "direct definition token-overlap relevance for [run]",
      "confidence": "context_high",
      "score": 0.98
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

Phase 5/6 keeps schema 2 stable and upgrades matching/ranking only: context rows now use
token-overlap rationale text (for example `token-overlap relevance`) and deterministic
definition-first scoring, with optional additive scope controls (`--code-only`, `--exclude-tests`)
that filter rows without changing the schema envelope.

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
- `target_kind` (`integration_test_file | support_test_file`)
- `why_included` (`string`)
- `confidence` (`graph_likely | context_medium`)
- `score` (`number`)

Default output excludes support paths and emits runnable integration targets first.
When `tests-for --include-support` is used, support rows are restored additively as
`target_kind = "support_test_file"`.

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

Phase 5/6 keeps schema 2 stable and adds precision controls through CLI options:
`--max-targeted` bounds symbol-derived targeted rows (default cap `8`, `0` means none),
while changed runnable test targets and the `cargo test` full-suite gate remain preserved.
Phase 6 adds additive changed-scope filters:
- repeatable `--changed-line path:start[:end]` limits symbol-derived targeted rows by span overlap,
- repeatable `--changed-symbol` limits symbol-derived targeted rows to named symbols.

## Schema 3 Contracts (Implemented, Frozen)

These contracts are intentionally additive and do not change schema 1 or schema 2 payloads.

## `diff-impact --json` (Schema 3)

```json
{
  "schema_version": 3,
  "command": "diff-impact",
  "changed_files": ["src/query/mod.rs"],
  "max_distance": 2,
  "include_tests": true,
  "results": [
    {
      "result_kind": "impacted_symbol",
      "symbol": "impact_matches",
      "qualified_symbol": "rust:src/query/mod.rs::impact_matches",
      "kind": "function",
      "language": "rust",
      "file_path": "src/query/mod.rs",
      "line": 120,
      "column": 8,
      "distance": 0,
      "relationship": "changed_symbol",
      "why_included": "symbol defined in changed file",
      "confidence": "graph_exact",
      "provenance": "ast_definition",
      "score": 1.0
    },
    {
      "result_kind": "test_target",
      "target": "tests/milestone10_validation.rs",
      "target_kind": "integration_test_file",
      "language": "rust",
      "why_included": "references impacted symbol 'impact_matches'",
      "confidence": "graph_likely",
      "provenance": "text_fallback",
      "score": 0.84
    }
  ]
}
```

Top-level fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schema_version` | `number` | yes | Always `3`. |
| `command` | `string` | yes | Always `"diff-impact"`. |
| `changed_files` | `array<string>` | yes | Repo-relative, normalized, sorted, deduplicated. |
| `max_distance` | `number` | yes | Echoes resolved traversal distance. |
| `include_tests` | `boolean` | yes | Echoes resolved test-target behavior (currently `true` by default). |
| `results` | `array<DiffImpactResult>` | yes | Deterministically ordered (see rules below). |

Phase 4/5/6 option effects (schema unchanged):

- `--include-imports` changes changed-symbol seed selection by allowing `kind=import` at
  `distance=0`.
- `--changed-line` limits changed-symbol seeds to symbol spans overlapping the specified line
  ranges.
- `--max-distance` now drives true bounded multi-hop inbound traversal for `distance >= 1`.
- Traversal suppresses changed-symbol re-entry at `distance > 0` and uses deterministic dedupe for
  cycle safety.
- repeatable `--changed-symbol` narrows changed-symbol seeds additively.
- `--exclude-changed` removes `relationship = changed_symbol` rows from final output while keeping
  traversal rooted at those seeds.
- `--max-results` applies deterministic post-sort truncation.
- `--include-tests` is currently a compatibility flag; schema 3 continues to report
  `include_tests = true` by default.
- Phase 7 calibrates semantic impacted-symbol row scores deterministically by
  relationship/provenance/distance (for example resolved `called_by` rows score `0.97` at
  `distance = 1` in the Phase 7 benchmark fixture).
- Neither option requires new mandatory top-level fields in schema 3.

`DiffImpactResult` union discriminator:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `result_kind` | `string` | yes | `"impacted_symbol"` or `"test_target"`. |

`DiffImpactResult` when `result_kind = "impacted_symbol"`:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `symbol` | `string` | yes | Unqualified symbol text. |
| `qualified_symbol` | `string` | yes | Stable qualified symbol ID. |
| `kind` | `string` | yes | Symbol kind enum. |
| `language` | `string` | yes | Language enum. |
| `file_path` | `string` | yes | Repo-relative path to symbol definition. |
| `line` | `number` | yes | 1-based start line. |
| `column` | `number` | yes | 1-based start column. |
| `distance` | `number` | yes | Graph distance from changed symbol (`0` means changed symbol itself). |
| `relationship` | `string` | yes | Relationship enum. |
| `why_included` | `string` | yes | Human-readable deterministic rationale. |
| `confidence` | `string` | yes | Confidence enum. |
| `provenance` | `string` | yes | Provenance enum. |
| `score` | `number` | yes | Ranking score. |

`DiffImpactResult` when `result_kind = "test_target"`:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `target` | `string` | yes | Repo-relative test file path. |
| `target_kind` | `string` | yes | Currently `"integration_test_file"`. |
| `language` | `string` | yes | Language enum for the target file. |
| `why_included` | `string` | yes | Human-readable deterministic rationale. |
| `confidence` | `string` | yes | Confidence enum. |
| `provenance` | `string` | yes | Provenance enum. |
| `score` | `number` | yes | Ranking score. |

Deterministic ordering rules for `results`:

1. Sort by `score` descending.
2. Tie-break by `result_kind` (`impacted_symbol` before `test_target`).
3. For `impacted_symbol`, tie-break by `file_path`, `line`, `column`, `qualified_symbol`.
4. For `test_target`, tie-break by `target_kind`, then `target`.

## `explain --json` (Schema 3)

```json
{
  "schema_version": 3,
  "command": "explain",
  "query": "impact_matches",
  "include_snippets": false,
  "results": [
    {
      "symbol": "impact_matches",
      "qualified_symbol": "rust:src/query/mod.rs::impact_matches",
      "kind": "function",
      "language": "rust",
      "file_path": "src/query/mod.rs",
      "start_line": 120,
      "start_column": 8,
      "end_line": 210,
      "end_column": 2,
      "signature": "pub fn impact_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<ImpactMatch>>",
      "inbound": {
        "called_by": 2,
        "imported_by": 0,
        "implemented_by": 0,
        "contained_by": 1
      },
      "outbound": {
        "calls": 4,
        "imports": 0,
        "implements": 0,
        "contains": 0
      },
      "why_included": "exact symbol definition match",
      "confidence": "graph_exact",
      "provenance": "ast_definition",
      "score": 1.0
    }
  ]
}
```

Top-level fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schema_version` | `number` | yes | Always `3`. |
| `command` | `string` | yes | Always `"explain"`. |
| `query` | `string` | yes | Input symbol/query string. |
| `include_snippets` | `boolean` | yes | Echoes resolved snippet behavior. |
| `results` | `array<ExplainMatch>` | yes | Deterministically ordered. |

`ExplainMatch` fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `symbol` | `string` | yes | Unqualified symbol text. |
| `qualified_symbol` | `string` | yes | Stable qualified symbol ID. |
| `kind` | `string` | yes | Symbol kind enum. |
| `language` | `string` | yes | Language enum. |
| `file_path` | `string` | yes | Repo-relative path. |
| `start_line` | `number` | yes | 1-based start line. |
| `start_column` | `number` | yes | 1-based start column. |
| `end_line` | `number` | yes | 1-based end line. |
| `end_column` | `number` | yes | 1-based end column. |
| `signature` | `string` | no | Present when extractor provides one. |
| `inbound` | `object` | yes | Relationship counters (see below). |
| `outbound` | `object` | yes | Relationship counters (see below). |
| `why_included` | `string` | yes | Human-readable deterministic rationale. |
| `confidence` | `string` | yes | Confidence enum. |
| `provenance` | `string` | yes | Provenance enum. |
| `score` | `number` | yes | Ranking score. |
| `snippet` | `string` | no | Present only when `include_snippets = true` and snippet extraction succeeds. |

`inbound` fields (all required numbers):

- `called_by`
- `imported_by`
- `implemented_by`
- `contained_by`

`outbound` fields (all required numbers):

- `calls`
- `imports`
- `implements`
- `contains`

Deterministic ordering rules for `results`:

1. Sort by `score` descending.
2. Tie-break by `file_path`, `start_line`, `start_column`, `qualified_symbol`.

## Schema 3 Enumerations (Frozen)

`language` values:

- `rust`
- `typescript`
- `python`
- `unknown`

`kind` values:

- `function`
- `method`
- `class`
- `interface`
- `trait`
- `enum`
- `module`
- `type_alias`
- `const`
- `variable`
- `import`

`relationship` values:

- `changed_symbol`
- `called_by`
- `contained_by`
- `imported_by`
- `implemented_by`

`confidence` values:

- `graph_exact`
- `graph_likely`
- `context_high`
- `context_medium`
- `context_low`

`provenance` values:

- `ast_definition`
- `ast_reference`
- `import_resolution`
- `call_resolution`
- `text_fallback`

## Determinism Expectations

For the same indexed state and same inputs:

- field names stay stable,
- key ordering is stable,
- result ordering is deterministic,
- file paths are repository-relative.
