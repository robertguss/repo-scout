# Agent-First CLI Specification for `repo-scout`

Date: 2026-02-13
Status: Draft proposal
Audience: maintainers building `repo-scout` for coding agents

## Purpose

This specification defines how `repo-scout` should evolve from a human-oriented CLI with optional
machine output into an agent-first analysis interface with deterministic contracts.

After implementing this spec, coding agents should be able to trust `repo-scout` as a primary
planning and verification engine for indexing, querying, and refactoring workflows without custom
per-command heuristics.

## Objective

The objective is to make `repo-scout` maximally useful for autonomous coding agents by improving:

1. machine contract consistency,
2. correctness and precision of diagnostics,
3. orchestration efficiency for multi-step workflows,
4. explainability and confidence metadata,
5. operational safety (index freshness, explicit failure modes).

## Non-Goals

1. replacing the CLI with a GUI,
2. removing human-readable output entirely,
3. introducing schema migrations as a prerequisite,
4. changing command names unless strictly necessary.

## Guiding Principles

1. Machine contracts are primary; text UX is secondary.
2. Deterministic output and explicit semantics beat convenience shortcuts.
3. Conservative defaults are required for refactoring recommendations.
4. Empty results must never hide missing or stale analysis state.
5. Every actionable recommendation must include rationale and confidence.

## Current Gaps (Observed)

1. JSON support is inconsistent across commands.
2. Schema/versioning contracts are command-local and hard to discover.
3. Missing/stale index states can appear as empty successful results.
4. Filter/flag capabilities are uneven across related commands.
5. Refactor diagnostics have limited confidence/provenance metadata.
6. Agents must issue many subprocess calls to compose common workflows.

## Agent-First Contract Model

`repo-scout` should be treated as an analysis API exposed over CLI transport. All commands should
support a standard response envelope, stable error semantics, and explicit feature/version
introspection.

### Standard Success Envelope

All JSON responses should conform to this top-level shape:

    {
      "schema": "repo-scout/<command>@vN",
      "command": "<command>",
      "ok": true,
      "meta": {
        "repo": ".",
        "index": {
          "schema_version": 4,
          "indexed_at": "2026-02-13T12:34:56Z",
          "head_sha": "...",
          "stale": false
        },
        "filters": {
          "scope": "production",
          "lang": null,
          "file": null,
          "exclude_glob": []
        },
        "duration_ms": 17,
        "trace_id": "..."
      },
      "data": { ... }
    }

### Standard Error Envelope

All JSON errors should conform to this shape:

    {
      "schema": "repo-scout/error@v1",
      "command": "find",
      "ok": false,
      "error": {
        "code": "INDEX_STALE",
        "message": "Index is stale relative to repository HEAD",
        "details": {
          "repo_head": "...",
          "index_head": "...",
          "suggested_fix": "repo-scout index --repo ."
        }
      },
      "meta": {
        "trace_id": "...",
        "duration_ms": 5
      }
    }

## Global CLI Contract

### Required Global Flags

All commands should support:

1. `--repo <path>`
2. `--format json|jsonl|text` (default should be `json` in agent mode)
3. `--trace-id <id>`
4. `--timeout-ms <u32>`
5. `--quiet` (suppress non-essential text)
6. `--strict` (fail on warnings/partial data)

### Optional Global Flags

1. `--output <path>` for writing machine output,
2. `--no-color` for text mode,
3. `--profile conservative|balanced|aggressive` for refactor/diagnostic behavior.

### Exit Code Taxonomy

Use process exit codes deterministically:

1. `0`: success
2. `2`: usage or argument validation error
3. `3`: index missing or stale (when strict freshness required)
4. `4`: internal/query/parsing failure
5. `5`: partial data returned but strict mode requested

## Index Lifecycle and Freshness Semantics

### Required Behaviors

1. Query commands must distinguish `NO_RESULTS` from `INDEX_MISSING` and `INDEX_STALE`.
2. Add `--require-index-fresh` to read commands.
3. Add `--auto-index` to opportunistically index before query.
4. Add `status --json` fields: `indexed_at`, `head_sha`, `stale`, `file_count`, parser warnings.

### New Commands

1. `index verify --repo <repo> --json`
2. `index prune --repo <repo> --json` (optional optimization)
3. `schema --json` for command/schema introspection.

## Filter and Scope Unification

A single filter model should apply to all analysis commands where meaningful:

1. `--scope all|production|tests`
2. `--lang <lang>`
3. `--file <path-or-glob>`
4. `--exclude-glob <glob>` (repeatable)
5. `--include-fixtures`
6. `--exclude-tests`
7. `--code-only`

Commands currently missing parity should adopt relevant subsets (especially `coupling`, `suggest`,
`boundary`, `rename-check`, `move-check`, `split-check`, `test-gaps`).

## Command-Level Spec Changes

## Discovery and Lookup Commands

### `find` and `refs`

Additions:

1. `--min-confidence <float>`
2. `--symbol-id <id>` for disambiguated lookup
3. `--require-index-fresh`

Data additions per row:

1. `symbol_id`
2. `container`
3. `confidence`
4. `why_matched`
5. `provenance` (AST/text/import-hint)

### `resolve` (new)

Purpose: resolve potentially ambiguous symbols into canonical `symbol_id` candidates.

Inputs:

1. symbol string,
2. optional `--lang`, `--file`, `--scope`.

Outputs:

1. ranked candidates with `symbol_id`, location, signature summary,
2. ambiguity metadata and recommended target.

## Graph and Impact Commands

### `impact`, `callers`, `callees`, `related`, `call-path`, `deps`

Additions:

1. filter parity,
2. `--max-results`,
3. deterministic cursor-based pagination (`--cursor`, `--page-size`) for larger graphs,
4. edge-level provenance and confidence fields.

## Refactoring Diagnostics Commands

### `dead`

Required model:

1. default conservative mode with high precision,
2. aggressive mode only by explicit opt-in,
3. confidence tier (`high|medium|low`) and rationale per item,
4. roots considered listed in metadata.

Required flags:

1. `--mode conservative|aggressive`
2. `--min-confidence <tier-or-score>`
3. `--explain` (include audit trail)

### `test-gaps`

Required output semantics:

1. explicit state: `covered`, `uncovered`, `unknown`,
2. reasons for unknowns,
3. evidence links to test targets and symbols,
4. risk scoring made explainable.

### `boundary`

Required contract fix:

1. `--public-only` must only include public symbols,
2. internal symbols excluded in both text and JSON for this mode,
3. include per-symbol external reference evidence.

### `coupling`

Required default behavior:

1. production-first filtering by default,
2. fixture/test coupling excluded unless explicitly requested,
3. optional directional coupling metrics and normalized scores.

### `suggest`

Required improvements:

1. score decomposition fields (`size`, `fan_in`, `test_gap`, `coupling`, `confidence`),
2. risk profile controls,
3. suppression reasons when candidates are filtered out.

## Refactoring Preflight Commands

### `extract-check`, `move-check`, `rename-check`, `split-check`

Required additions:

1. filter parity,
2. confidence/risk metadata,
3. downstream impact grouping by category,
4. deterministic machine-readable warning codes.

`rename-check` must separate:

1. semantic references (graph/AST based),
2. lexical occurrences (string/text only),
3. unknown/ambiguous matches.

## Workflow-Oriented Commands

### `query` (new batch command)

Purpose: execute many requests in one process with uniform contracts.

Input:

1. JSONL or JSON array request payload.

Output:

1. JSONL response objects with per-request status,
2. stable request IDs and trace IDs,
3. optional fail-fast behavior.

### `refactor-plan` (new orchestration command)

Purpose: generate a complete, conservative refactoring plan for a target.

Pipeline (internally composed):

1. boundary and dependency extraction,
2. dead and test-gap diagnostics,
3. preflight checks,
4. safe steps and verification checklist generation.

Output:

1. ranked action list,
2. risk levels and confidence,
3. commands to run for validation,
4. explicit blockers and unknowns.

### `serve-mcp` (new optional daemon mode)

Purpose: expose `repo-scout` as a low-latency MCP tool server for agents.

Requirements:

1. same schemas as CLI JSON contracts,
2. command parity,
3. request/response tracing,
4. concurrency-safe read operations.

## Human Output Policy

Text mode remains available, but machine contracts are normative.

Rules:

1. text mode may summarize,
2. JSON/JSONL mode must be complete and stable,
3. all docs and tests should assert JSON contracts first.

## Schema Registry and Versioning

### Required Registry

`schema --json` should expose:

1. command name,
2. schema ID,
3. current version,
4. deprecated fields,
5. planned removals.

### Versioning Policy

1. additive fields: minor version bump,
2. renamed/removed fields: major version bump,
3. at least one compatibility window where old and new fields coexist.

## Determinism and Performance Requirements

1. All JSON arrays must have documented sort order.
2. Repeated identical calls against identical index state must return byte-stable JSON when
   timestamps are excluded.
3. Batch mode throughput should beat equivalent subprocess loops by at least 3x on local dogfood
   workloads.
4. Added explainability fields should not materially regress baseline command latency; if they do,
   gated profiles are required.

## Security and Safety Requirements

1. No command should imply destructive edits.
2. Refactor recommendations must surface risk/confidence before action steps.
3. Unsafe or low-confidence recommendations must be explicitly labeled.
4. All path-like inputs should be normalized and validated consistently.

## Deprecation and Migration Plan

### Phase A: Contract Foundation

1. Add global envelope and error model.
2. Add `status --json` parity and index freshness semantics.
3. Add schema registry command.

### Phase B: Precision and Diagnostics Hardening

1. Implement `dead` confidence modes and rationale.
2. Fix `boundary --public-only` contract.
3. Harden `test-gaps` state semantics.
4. Improve `rename-check` semantic vs lexical split.
5. Add `coupling` noise controls.

### Phase C: Agent Throughput and Orchestration

1. Implement `query` batch mode.
2. Implement `resolve` command and `symbol_id` references.
3. Implement `refactor-plan` orchestration output.

### Phase D: Optional Agent-Native Runtime

1. Implement `serve-mcp`.
2. Validate parity with CLI contracts.

## Acceptance Criteria

This spec is considered implemented when:

1. every command used by agents supports deterministic JSON contracts,
2. missing/stale index states are explicit and machine-distinguishable,
3. refactoring diagnostics include confidence and rationale,
4. batch mode and orchestration commands reduce agent command count materially,
5. integration tests assert schema contracts and deterministic outputs across the upgraded surface.

## Example Agent Flows After Implementation

### Reliable symbol rename planning

1. `resolve run --repo . --format json`
2. `rename-check --symbol-id <id> --to execute --repo . --format json --profile conservative`
3. `test-gaps <id> --repo . --format json`
4. `safe-steps <id> --action rename --to execute --repo . --format json`
5. `verify-refactor --before <sha> --after <sha> --repo . --format json --strict`

### Repository-wide refactor triage in one call

1. `refactor-plan src/main.rs --repo . --format json --profile conservative`

### Multi-query agent loop with reduced overhead

1. `query --repo . --format jsonl --input requests.jsonl`

## Open Questions

1. Should `--format json` become default for all commands or only when `CI=true` / explicit profile
   is set?
2. Should `symbol_id` be globally stable across reindex or stable only within one index snapshot?
3. Should `serve-mcp` ship in-core or as a companion binary first?

## Implementation Notes for Maintainers

1. Prioritize contract consistency and explicit failure semantics before adding new commands.
2. Keep conservative defaults for all refactoring recommendations.
3. Preserve backward compatibility with current command names and key fields during migration
   windows.
4. Enforce JSON contract tests per command as part of milestone gating.
