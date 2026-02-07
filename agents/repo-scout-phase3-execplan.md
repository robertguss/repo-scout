# Build `repo-scout` Phase 3 Agent Loop Commands and Multi-Language Contracts

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan builds on `agents/repo-scout-agent-first-phase2-execplan.md`, which delivered schema v2
graph storage plus `impact`, `context`, `tests-for`, and `verify-plan`.

## Purpose / Big Picture

Phase 3 is focused on improving the core coding-agent loop before broad language rollout. After this
change, a user can pass changed files and receive deterministic impact/test guidance
(`diff-impact`), inspect a symbol quickly (`explain`), and trust that TypeScript and Python support
will plug into stable language-neutral interfaces instead of one-off parser logic. The user-visible
result is faster “what changed, what breaks, what should I run” navigation with the same
deterministic output guarantees established in v0 and Phase 2.

This plan intentionally starts with contract locking and strict TDD slices so implementation can
proceed quickly without schema churn. The goal is to avoid spending effort on language adapters
before agent-facing commands and data contracts are stable.

## Progress

- [x] (2026-02-06 16:29Z) Re-read `agents/PLANS.md`, `agents/repo-scout-hybrid-rust-execplan.md`,
      and `agents/repo-scout-agent-first-phase2-execplan.md` to align Phase 3 with repository
      standards.
- [x] (2026-02-06 16:29Z) Ran dogfooding baseline commands (`cargo run -- index --repo .`,
      `cargo run -- find launch --repo . --json`, `cargo run -- refs launch --repo . --json`) to
      confirm current command and schema behavior before planning.
- [x] (2026-02-06 16:29Z) Authored this Phase 3 ExecPlan as planning-only work (no production code
      changes outside plan docs).
- [x] (2026-02-06 16:46Z) Milestone 11 slice 11A complete: red/green/refactor for
      `milestone11_diff_impact_cli_contract` plus mandatory pre/post dogfooding commands.
- [x] (2026-02-06 16:46Z) Milestone 11 slice 11B complete: red/green/refactor for
      `milestone11_diff_impact_json_contract` and minimal schema v3 payload production with
      `result_kind`, `qualified_symbol`, and `provenance`.
- [x] (2026-02-06 16:46Z) Milestone 11 slice 11C complete: red/green/refactor for
      `milestone11_explain_cli_contract` and new `explain` command wiring.
- [x] (2026-02-06 16:46Z) Milestone 11 slice 11D complete: red/green/refactor for
      `milestone11_explain_json_contract` including optional snippet extraction behavior.
- [x] (2026-02-06 16:46Z) Milestone 11 contract-freeze implementation complete
      (`diff-impact` and `explain` CLI surfaces, schema v3 JSON contracts, and integration tests).
- [x] (2026-02-06 16:57Z) Milestone 12 slice 12A complete: red/green/refactor for
      `milestone12_diff_impact_changed_files_normalization` plus normalization fix for canonical
      absolute paths and terminal changed-file listing.
- [x] (2026-02-06 16:57Z) Milestone 12 slice 12B complete: red/green/refactor for
      `milestone12_diff_impact_graph_neighbors` with direct inbound graph neighbor expansion
      (`distance = 1`, relationship mapping, provenance mapping).
- [x] (2026-02-06 16:57Z) Milestone 12 slice 12C complete: red/green/refactor for
      `milestone12_diff_impact_includes_tests` with `test_target` emission when tests are enabled.
- [x] (2026-02-06 16:57Z) Milestone 12 slice 12D complete: red/green/refactor for
      `milestone12_diff_impact_deterministic_ordering` including `max_distance` enforcement,
      deterministic ordering tie-breaks, and vocabulary assertions.
- [x] (2026-02-06 16:57Z) Milestone 12 feature implementation complete (`diff-impact` behavior in
      terminal + JSON with deterministic ranking and confidence/provenance).
- [x] (2026-02-06 17:03Z) Milestone 13 slice 13A complete: red/green/refactor for
      `milestone13_explain_definition_summary` with terminal dossier signature output.
- [x] (2026-02-06 17:03Z) Milestone 13 slice 13B complete: red/green/refactor for
      `milestone13_explain_relationship_summary` with inbound/outbound edge-count summaries.
- [x] (2026-02-06 17:03Z) Milestone 13 slice 13C complete: red/green/refactor for
      `milestone13_explain_json_determinism` and full-span snippets under
      `--include-snippets`.
- [x] (2026-02-06 17:03Z) Milestone 13 feature implementation complete (`explain` behavior in
      terminal + JSON with symbol dossier output).
- [x] (2026-02-06 17:18Z) Milestone 14 slice 14A complete: red/green/refactor for
      `milestone14_language_adapter_trait_migration`, replacing direct indexer parser dependency
      assertions with adapter-boundary checks.
- [x] (2026-02-06 17:18Z) Milestone 14 slice 14B complete: red/green/refactor for
      `milestone14_rust_behavior_unchanged_through_adapter` by restoring `ast_references`
      persistence through adapter-provided reference extraction.
- [x] (2026-02-06 17:18Z) Milestone 14 slice 14C complete: red/green/refactor for
      `milestone14_schema_language_metadata_migration` with additive schema v3 migration for
      `language`, `qualified_symbol`, `provenance`, and new indices.
- [x] (2026-02-06 17:18Z) Milestone 14 feature implementation complete (adapter extraction boundary
      stabilized, Rust behavior preserved, and schema migration upgraded to v3 metadata contracts).
- [x] (2026-02-07 00:11Z) Milestone 15 slice 15A complete: red/green/refactor for
      `milestone15_typescript_definitions` with TypeScript adapter definition extraction
      (function/class/interface/enum/type_alias/method + language metadata).
- [x] (2026-02-07 00:11Z) Milestone 15 slice 15B complete: red/green/refactor for
      `milestone15_typescript_references_and_calls` with arrow/function-expression caller
      resolution and call edge persistence.
- [x] (2026-02-07 00:11Z) Milestone 15 slice 15C complete: red/green/refactor for
      `milestone15_typescript_edges_and_queries` with import/implements/contains edges flowing
      through `impact`, `diff-impact`, and `explain`.
- [x] (2026-02-07 00:11Z) Milestone 15 feature implementation complete (TypeScript adapter MVP with
      deterministic query behavior and milestone-level manual `diff-impact`/`explain` checks).
- [x] (2026-02-07 00:19Z) Milestone 16 slice 16A complete: red/green/refactor for
      `milestone16_python_definitions` with Python adapter extraction for
      class/function/method/constant definitions and `contains` edges.
- [x] (2026-02-07 00:19Z) Milestone 16 slice 16B complete: red/green/refactor for
      `milestone16_python_references_calls_imports` with Python `ast_references`, `calls`, and
      `imports` graph extraction.
- [x] (2026-02-07 00:19Z) Milestone 16 slice 16C complete: red/green/refactor for
      `milestone16_python_edges_and_queries` with deterministic `find`/`refs`/`diff-impact` and
      `explain` behavior on Python fixtures.
- [x] (2026-02-07 00:19Z) Milestone 16 feature implementation complete (Python adapter MVP wired
      behind language adapters with deterministic query behavior and milestone-level manual
      `diff-impact`/`explain` checks).
- [x] (2026-02-07 00:22Z) Milestone 17 complete: documentation and dogfood transcript updates
      landed in `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
      `docs/architecture.md`, and `docs/dogfood-log.md` with Phase 3 command/adaptation behavior.

## Surprises & Discoveries

- Observation: `refs launch --json` currently returns many fallback text hits from documentation and
  test code, which can drown direct code-navigation tasks. Evidence: Baseline dogfood output
  returned large result sets dominated by `README.md` and `tests/*` occurrences.

- Observation: Existing command families already use independent schema versions (`find`/`refs` v1,
  newer commands v2), which enables additive command-specific schema evolution. Evidence:
  `docs/json-output.md` and live command output confirm this split.

- Observation: Current graph model is sufficient for first-order changed-file impact without
  introducing semantic compiler integrations. Evidence: `symbols_v2` and `symbol_edges_v2` already
  support deterministic edge queries for `impact` and `verify-plan`.

- Observation: `explain --json --include-snippets` can only load source content by deriving the repo
  root from the SQLite path (`<repo>/.repo-scout/index.db`). Evidence: snippet extraction failed
  until `db_path.parent().parent()` was used to locate the file root.

- Observation: Starting `diff-impact` contract tests with an empty result set made it impossible to
  validate required union fields (`result_kind`, `provenance`) in JSON tests. Evidence:
  `milestone11_diff_impact_json_contract` red run failed with `assertion failed: !results.is_empty()`.

- Observation: Absolute changed-file paths under macOS `/var/...` did not deduplicate against repo
  canonical paths under `/private/var/...`. Evidence:
  `milestone12_diff_impact_changed_files_normalization` red run showed `changed_files: 2` for the
  same file before canonicalizing candidate paths.

- Observation: `clap` boolean flags in this CLI shape do not accept explicit `true` literals (for
  example `--include-tests true`). Evidence: red run produced `error: unexpected argument 'true'
  found`, so tests use the default-enabled behavior for this flag.

- Observation: Snippet quality for `explain --include-snippets` depended on stored definition spans;
  identifier-only spans produced one-token snippets with no body context. Evidence:
  `milestone13_explain_json_determinism` red run failed on `assertion failed:
  snippet.contains("leaf();")` until Rust AST spans used full node end positions.

- Observation: Removing direct Rust parser calls from `indexer/mod.rs` without an adapter reference
  channel immediately regressed `refs` behavior to non-AST fallbacks. Evidence:
  `milestone14_rust_behavior_unchanged_through_adapter` red run failed on
  `assertion failed: refs_out.contains("[ast_reference ast_likely]")` until
  `ExtractionUnit.references` was added and persisted.

- Observation: Creating new schema-v3 indexes before adding missing columns on migrated v2 tables
  causes bootstrap failure. Evidence: `milestone14_schema_language_metadata_migration` green attempt
  failed with `no such column: language` during `CREATE INDEX ... symbols_v2(language, symbol)` and
  required moving metadata index creation after additive column migration.

- Observation: Cross-file TypeScript edges were dropped when the target symbol appeared in a later
  file because edge resolution happened inside per-file transactions only. Evidence:
  `milestone15_typescript_edges_and_queries` red run and SQLite inspection showed
  `run|callHelper|calls` but missing `callHelper|helper|imports` until a deferred edge pass was
  added after indexing all files.

- Observation: Tree-sitter TypeScript import field mapping is not stable enough across specifier
  forms (`{ x }` vs `{ x as y }`) for direct field-name extraction in this codebase. Evidence:
  alias import rows were defined but import edges did not resolve for `callHelper` until import
  bindings were parsed from import statement text.

- Observation: Python `refs` for imported symbols fell back to text-only matches until import
  statements contributed `ast_references` for imported names. Evidence:
  `milestone16_python_edges_and_queries` red run failed on
  `assertion failed: refs_results.iter().any(|item| item["why_matched"] == "ast_reference")`.

- Observation: For Python MVP, parsing import bindings from statement text is more stable across
  `import` and `from ... import ... as ...` forms than depending on tree-sitter field names.
  Evidence: import symbol/edge extraction became deterministic after line-text parsing with
  `last_identifier` normalization.

- Observation: Docs still described schema 3 commands as planned after implementation, which
  conflicted with current CLI behavior. Evidence: `docs/json-output.md` had planned-only language
  for `diff-impact`/`explain` until Milestone 17 refresh.

## Decision Log

- Decision: Sequence Phase 3 as command-first (`diff-impact`, `explain`) before TypeScript/Python
  extraction. Rationale: Agent usefulness depends first on actionability and deterministic command
  contracts, then on language coverage. Date/Author: 2026-02-06 / Codex

- Decision: Introduce a new schema family version (`schema_version = 3`) for new Phase 3 JSON
  commands while preserving existing v1/v2 contracts unchanged. Rationale: This avoids breaking
  existing automation consumers and keeps migration additive. Date/Author: 2026-02-06 / Codex

- Decision: Add a language-adapter interface milestone before adding TypeScript/Python parsers.
  Rationale: A normalized adapter boundary prevents parser-specific logic from leaking into
  query/ranking/output layers. Date/Author: 2026-02-06 / Codex

- Decision: Keep confidence and provenance vocabularies explicit and finite, and require
  deterministic ordering in all new outputs. Rationale: Agent automation depends on stable contracts
  and transparent uncertainty signals. Date/Author: 2026-02-06 / Codex

- Decision: Seed Milestone 11 `diff-impact` contract support with changed-symbol records directly
  from `symbols_v2` instead of leaving `results` empty. Rationale: frozen JSON contract tests must
  verify required per-result fields before Milestone 12 neighbor/test-target expansion.
  Date/Author: 2026-02-06 / Codex

- Decision: Implement `--include-snippets` for `explain` by reading indexed files from the repo path
  inferred via the index database location. Rationale: this preserves the existing query function
  signature and satisfies schema v3 optional snippet behavior without adding new CLI plumbing.
  Date/Author: 2026-02-06 / Codex

- Decision: Canonicalize absolute changed-file candidates before prefix stripping to normalize
  `/var` and `/private/var` path aliases. Rationale: deduplication must be stable across path alias
  forms to satisfy deterministic `changed_files` contracts. Date/Author: 2026-02-06 / Codex

- Decision: Gate neighbor traversal on `max_distance` and implement full diff-impact tie-break
  ordering for `test_target` rows. Rationale: this aligns runtime behavior with schema v3 ordering
  rules and avoids nondeterministic JSON from map iteration order. Date/Author: 2026-02-06 / Codex

- Decision: Compute explain relationship summaries directly from `symbol_edges_v2` grouped by
  `edge_kind` and project into fixed inbound/outbound counters. Rationale: deterministic counts are
  required for agent-readable dossiers and avoid additional graph traversal complexity.
  Date/Author: 2026-02-06 / Codex

- Decision: Persist Rust definition `end_line`/`end_column` from full AST node spans, not name-node
  spans. Rationale: snippet extraction should include meaningful code context (for example function
  body lines), not only identifier text. Date/Author: 2026-02-06 / Codex

- Decision: Extend the language adapter extraction payload with explicit references
  (`ExtractionUnit.references`) so `ast_references` can be persisted through the adapter boundary
  instead of direct parser calls in `indexer/mod.rs`. Rationale: Milestone 14 requires parser
  abstraction without regressing existing `refs` behavior. Date/Author: 2026-02-06 / Codex

- Decision: Upgrade store schema metadata to `SCHEMA_VERSION = 3` and perform additive migration for
  `symbols_v2.language`, `symbols_v2.qualified_symbol`, and `symbol_edges_v2.provenance` with
  deterministic backfill. Rationale: adapter contracts in Milestone 14 depend on persisted language
  identifiers and stable qualified/provenance values while keeping v1/v2 query behavior intact.
  Date/Author: 2026-02-06 / Codex

- Decision: Add a deferred edge-resolution pass after all files are indexed and push unresolved
  cross-file edges into that pass. Rationale: language adapters now emit cross-file relations
  (notably TypeScript imports/implements) that cannot always resolve during a single file
  transaction. Date/Author: 2026-02-07 / Codex

- Decision: Parse TypeScript import bindings from import statement text for named imports in the MVP
  path, then emit deterministic `import` symbols plus `imports` edges. Rationale: this is a small,
  reliable implementation for milestone scope without blocking on grammar-field variance across
  import syntactic forms. Date/Author: 2026-02-07 / Codex

- Decision: Classify Python `function_definition` nodes inside classes as `method` symbols and emit
  `contains` edges from class to method. Rationale: this preserves language-neutral relationship
  behavior for `impact`, `diff-impact`, and `explain` while keeping Python extraction minimal.
  Date/Author: 2026-02-07 / Codex

- Decision: Emit Python import bindings as both `import` symbols/`imports` edges and
  `ast_references` for the imported symbol name. Rationale: `refs` should stay AST-prioritized on
  import-driven usage and remain deterministic across repeated runs. Date/Author: 2026-02-07 / Codex

- Decision: Keep schema 3 payload contracts unchanged while rewording docs from “planned” to
  “implemented, frozen.” Rationale: this preserves the original contract freeze and avoids consumer
  churn while aligning docs with shipped behavior. Date/Author: 2026-02-07 / Codex

## Outcomes & Retrospective

Planning outcome at this stage: Phase 3 scope is explicitly sequenced around agent-loop value
(`diff-impact`, `explain`) with contract locking first, then adapter extraction, then
TypeScript/Python rollout. No implementation has been performed yet under this plan.

Milestone 11 outcome: `diff-impact` and `explain` commands now exist with schema v3 JSON envelopes
and contract tests in `tests/milestone11_contracts.rs`. `diff-impact` currently returns changed-file
symbol definitions (distance 0), and `explain` returns definition dossiers with optional snippets.
Milestone 12+ behavior (graph neighbors, test targets, richer relationship counts) remains pending.

Milestone 12 outcome: `diff-impact` now normalizes changed-file inputs deterministically, emits
distance-1 graph neighbors, optionally includes test targets, honors `max_distance`, and sorts mixed
result kinds deterministically under schema v3. Remaining scope is `explain` relationship depth and
language-adapter extraction/migration milestones.

Milestone 13 outcome: `explain` now provides terminal/JSON dossiers with definition signatures,
relationship counters (`inbound`/`outbound`), deterministic serialization, and multi-line snippets
when requested. Remaining work is language-adapter extraction/migration and TypeScript/Python
indexing rollout.

Milestone 14 outcome: indexing now consumes Rust extraction through the language-adapter boundary,
including adapter-provided references so legacy `find`/`refs` behavior remains intact. Store schema
is upgraded additively to version 3 with persisted `language`, `qualified_symbol`, and
`provenance` metadata plus migration-safe index creation/backfill. Remaining work is TypeScript and
Python adapter rollout plus documentation updates.

Milestone 15 outcome: TypeScript indexing now extracts definitions, references, calls, import edges,
implements edges, and contains edges through the adapter boundary. `impact`, `diff-impact`, and
`explain` produce deterministic TypeScript-labeled outputs for fixture repositories, and milestone15
manual dogfood checks for `diff-impact`/`explain` completed. Remaining work is Python adapter rollout
and documentation finalization.

Milestone 16 outcome: Python indexing now extracts definitions (functions/classes/methods/constants),
`ast_references`, `calls`, `imports`, and `contains` edges through the adapter boundary.
`find`, `refs`, `impact`, `diff-impact`, and `explain` produce deterministic Python-labeled output
on fixture repositories, and milestone16 manual dogfood checks for `diff-impact`/`explain`
completed. Remaining work is documentation finalization.

Milestone 17 outcome: user-facing docs now describe the implemented Phase 3 surface (schema version
3 commands, language-adapter architecture, and updated dogfood transcripts for new commands and
TypeScript/Python adapters). This closes Phase 3 plan scope with restartable documentation state.

Target completion outcome: `repo-scout` provides deterministic changed-file impact analysis and
symbol dossier commands, plus a language-neutral extraction pipeline that supports Rust, TypeScript,
and Python with shared output semantics.

Expected residual work after this plan: deeper semantic resolution (type-aware calls/implements),
ranking quality metrics based on real dogfooding telemetry, and possible LSP-backed confidence
upgrades.

## Context and Orientation

`repo-scout` command parsing lives in `src/cli.rs`, command dispatch in `src/main.rs`, indexing in
`src/indexer/`, storage schema in `src/store/schema.rs`, query behavior in `src/query/mod.rs`, and
output serialization in `src/output.rs`. Integration tests are under `tests/` and should remain
milestone-oriented.

A “language adapter” in this plan means a module that receives file content and returns normalized
symbol/edge extraction records that match repository-wide contracts. “Normalized” means
command/query layers do not care whether records came from Rust, TypeScript, or Python. A
“changed-file impact” command means a command that starts from one or more changed files, resolves
changed symbols from those files, traverses graph edges, and returns ranked impacted
symbols/files/tests with explicit rationale and confidence.

Phase 2 introduced graph tables and v2 commands. Phase 3 must preserve all existing command behavior
while adding new command surfaces and additive schema/migration changes.

## Strict TDD Contract

Strict red-green-refactor applies to every feature slice in this plan. A feature slice is the
smallest user-visible behavior unit, such as “`diff-impact --json` includes deterministic
`changed_files` ordering” or “TypeScript `import` creates `imports` edges.” No production code for a
slice is allowed before the red step fails for that exact behavior.

Each slice must record three artifacts in this file (or linked commit notes): a red failure
transcript, a green pass transcript for the targeted test, and a refactor transcript with full suite
pass. If scope changes while implementing a slice, split the slice in `Progress` and document the
decision in `Decision Log`.

Dogfooding rule for every slice must follow `AGENTS.md` exactly. Before implementing the slice, run
`cargo run -- index --repo .`, `cargo run -- find <target_symbol> --repo . --json`, and
`cargo run -- refs <target_symbol> --repo . --json` with a symbol relevant to that slice. After
implementing the slice, run `cargo run -- index --repo .`,
`cargo run -- find <target_symbol> --repo .`, `cargo run -- refs <target_symbol> --repo .`, and
`cargo test`.

## Plan of Work

Milestone 11 locks contracts before behavior changes. The work starts by adding failing integration
tests for new command parsing and JSON envelope stability for `diff-impact` and `explain`, including
schema version, deterministic top-level fields, and required per-result fields. Green implementation
for this milestone is only CLI/output envelope plumbing, not full ranking logic.

Milestone 12 implements `diff-impact` behavior in four slices. Slice 12A proves changed files are
normalized, sorted, deduplicated, and echoed deterministically in terminal and JSON outputs. Slice
12B proves changed files map to changed symbols and direct graph neighbors. Slice 12C proves
impacted test targets are included when `--include-tests` is enabled. Slice 12D proves ranking and
tie-break ordering are deterministic and confidence/provenance fields stay within the finite
vocabulary.

Milestone 13 implements `explain` in three slices. Slice 13A proves direct symbol dossier output
(definition summary and file location). Slice 13B proves inbound/outbound relationship summaries
(calls, called_by, imports, implemented_by, contains, contained_by). Slice 13C proves deterministic
JSON serialization with language metadata and confidence/provenance strings.

Milestone 14 extracts language-adapter interfaces while preserving behavior. Slice 14A introduces
normalized adapter traits and data structures with Rust adapter migrated behind the interface. Slice
14B proves no behavior regression for existing Rust milestones. Slice 14C adds migration-safe schema
support for language metadata needed by adapters.

Milestone 15 adds TypeScript adapter MVP with strict additive behavior. Slice 15A proves definition
extraction for functions/classes/interfaces/enums/type aliases. Slice 15B proves references/calls
extraction. Slice 15C proves import/implements/contains edges and deterministic output through
existing and new commands.

Milestone 16 adds Python adapter MVP in strict slices. Slice 16A proves definitions for
functions/classes/methods/constants. Slice 16B proves calls and imports extraction. Slice 16C proves
deterministic command behavior for `find`, `refs`, `diff-impact`, and `explain` on Python fixtures.

Milestone 17 completes docs and dogfood notes with command examples, JSON schema documentation,
architecture updates for adapter boundaries, and at least one transcript per new command and
language adapter.

## Concrete Steps

Run all commands from `.`.

For every feature slice, execute strict TDD in this order and capture transcripts:

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure first
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimal implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 11 command set:

    cargo test milestone11_diff_impact_cli_contract -- --nocapture
    cargo test milestone11_diff_impact_cli_contract -- --nocapture
    cargo test
    cargo test milestone11_diff_impact_json_contract -- --nocapture
    cargo test milestone11_diff_impact_json_contract -- --nocapture
    cargo test
    cargo test milestone11_explain_cli_contract -- --nocapture
    cargo test milestone11_explain_cli_contract -- --nocapture
    cargo test
    cargo test milestone11_explain_json_contract -- --nocapture
    cargo test milestone11_explain_json_contract -- --nocapture
    cargo test

Milestone 12 command set:

    cargo test milestone12_diff_impact_changed_files_normalization -- --nocapture
    cargo test milestone12_diff_impact_changed_files_normalization -- --nocapture
    cargo test
    cargo test milestone12_diff_impact_graph_neighbors -- --nocapture
    cargo test milestone12_diff_impact_graph_neighbors -- --nocapture
    cargo test
    cargo test milestone12_diff_impact_includes_tests -- --nocapture
    cargo test milestone12_diff_impact_includes_tests -- --nocapture
    cargo test
    cargo test milestone12_diff_impact_deterministic_ordering -- --nocapture
    cargo test milestone12_diff_impact_deterministic_ordering -- --nocapture
    cargo test

Milestone 13 command set:

    cargo test milestone13_explain_definition_summary -- --nocapture
    cargo test milestone13_explain_definition_summary -- --nocapture
    cargo test
    cargo test milestone13_explain_relationship_summary -- --nocapture
    cargo test milestone13_explain_relationship_summary -- --nocapture
    cargo test
    cargo test milestone13_explain_json_determinism -- --nocapture
    cargo test milestone13_explain_json_determinism -- --nocapture
    cargo test

Milestone 14 command set:

    cargo test milestone14_language_adapter_trait_migration -- --nocapture
    cargo test milestone14_language_adapter_trait_migration -- --nocapture
    cargo test
    cargo test milestone14_rust_behavior_unchanged_through_adapter -- --nocapture
    cargo test milestone14_rust_behavior_unchanged_through_adapter -- --nocapture
    cargo test
    cargo test milestone14_schema_language_metadata_migration -- --nocapture
    cargo test milestone14_schema_language_metadata_migration -- --nocapture
    cargo test

Milestone 15 command set:

    cargo test milestone15_typescript_definitions -- --nocapture
    cargo test milestone15_typescript_definitions -- --nocapture
    cargo test
    cargo test milestone15_typescript_references_and_calls -- --nocapture
    cargo test milestone15_typescript_references_and_calls -- --nocapture
    cargo test
    cargo test milestone15_typescript_edges_and_queries -- --nocapture
    cargo test milestone15_typescript_edges_and_queries -- --nocapture
    cargo test

Milestone 16 command set:

    cargo test milestone16_python_definitions -- --nocapture
    cargo test milestone16_python_definitions -- --nocapture
    cargo test
    cargo test milestone16_python_references_calls_imports -- --nocapture
    cargo test milestone16_python_references_calls_imports -- --nocapture
    cargo test
    cargo test milestone16_python_edges_and_queries -- --nocapture
    cargo test milestone16_python_edges_and_queries -- --nocapture
    cargo test

Manual command checks after Milestones 12, 13, 15, and 16:

    cargo run -- index --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
    cargo run -- explain impact_matches --repo .
    cargo run -- explain impact_matches --repo . --json

Before finalizing:

    cargo fmt
    cargo test

## Validation and Acceptance

Acceptance is behavior-first. For `diff-impact`, given one or more changed files, output must
deterministically list impacted symbols and recommended test targets with explicit rationale and
confidence labels. Running the same command twice on unchanged index state must yield
byte-equivalent JSON.

For `explain`, given a symbol, output must include deterministic definition details and graph
relationship summaries. JSON output must expose language metadata and relationship counts in stable
field order.

For TypeScript and Python milestones, fixture-based tests must show that `find`, `refs`,
`diff-impact`, and `explain` all return non-empty, deterministic, language-labeled results where
appropriate, without regressing existing Rust behavior.

Final acceptance requires strict TDD evidence per slice (red, green, refactor), full-suite pass, and
updated docs in `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
and `docs/dogfood-log.md`.

## Idempotence and Recovery

Indexing must remain idempotent for all languages. Reindexing unchanged repositories must not
duplicate symbols or edges. Changed-file normalization for `diff-impact` must be deterministic and
safe to rerun.

Schema evolution must be additive. Existing v1/v2 JSON consumers must remain unaffected. If schema
migration to v3 fails, errors must include index path and actionable recovery guidance without
silently dropping prior data.

## Artifacts and Notes

Contract target for `diff-impact --json` in Phase 3:

    {
      "schema_version": 3,
      "command": "diff-impact",
      "changed_files": ["src/query/mod.rs"],
      "results": [
        {
          "symbol": "impact_matches",
          "qualified_symbol": "crate::query::impact_matches",
          "kind": "function",
          "language": "rust",
          "file_path": "src/query/mod.rs",
          "line": 120,
          "column": 8,
          "distance": 0,
          "relationship": "changed_symbol",
          "why_included": "symbol defined in changed file",
          "confidence": "graph_exact",
          "score": 1.0
        }
      ]
    }

Contract target for `explain --json` in Phase 3:

    {
      "schema_version": 3,
      "command": "explain",
      "query": "impact_matches",
      "results": [
        {
          "symbol": "impact_matches",
          "qualified_symbol": "crate::query::impact_matches",
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
          "score": 1.0
        }
      ]
    }

Strict TDD artifact checklist to fill during implementation:

    - milestone11_diff_impact_cli_contract
      red: `cargo test milestone11_diff_impact_cli_contract -- --nocapture` failed with
      `unrecognized subcommand 'diff-impact'`.
      green: `cargo test milestone11_diff_impact_cli_contract -- --nocapture` passed after
      adding `DiffImpact` CLI wiring.
      refactor: `cargo test` passed full suite.

    - milestone11_diff_impact_json_contract
      red: `cargo test milestone11_diff_impact_json_contract -- --nocapture` failed with
      `assertion failed: !results.is_empty()`.
      green: `cargo test milestone11_diff_impact_json_contract -- --nocapture` passed after
      changed-symbol result emission in `diff_impact_for_changed_files`.
      refactor: `cargo test` passed full suite.

    - milestone11_explain_cli_contract
      red: `cargo test milestone11_explain_cli_contract -- --nocapture` failed with
      `unrecognized subcommand 'explain'`.
      green: `cargo test milestone11_explain_cli_contract -- --nocapture` passed after
      adding `Explain` command dispatch and terminal output.
      refactor: `cargo test` passed full suite.

    - milestone11_explain_json_contract
      red: `cargo test milestone11_explain_json_contract -- --nocapture` failed with
      `assertion failed: snippet.contains("compute_plan")`.
      green: `cargo test milestone11_explain_json_contract -- --nocapture` passed after
      snippet extraction from repository sources.
      refactor: `cargo test` passed full suite.

    - dogfood pre/post evidence captured for every milestone11 slice with required commands:
      `cargo run -- index --repo .`,
      `cargo run -- find <target_symbol> --repo . --json`,
      `cargo run -- refs <target_symbol> --repo . --json`,
      and post-implementation `cargo run -- find <target_symbol> --repo .`,
      `cargo run -- refs <target_symbol> --repo .`, `cargo test`.

    - milestone12_diff_impact_changed_files_normalization
      red: `cargo test milestone12_diff_impact_changed_files_normalization -- --nocapture`
      failed with `assertion failed: terminal_out.contains("changed_files: 1")`.
      green: `cargo test milestone12_diff_impact_changed_files_normalization -- --nocapture`
      passed after changed-file canonicalization and terminal path output additions.
      refactor: `cargo test` passed full suite.

    - milestone12_diff_impact_graph_neighbors
      red: `cargo test milestone12_diff_impact_graph_neighbors -- --nocapture`
      failed with missing `watcher` `called_by` distance-1 result.
      green: `cargo test milestone12_diff_impact_graph_neighbors -- --nocapture`
      passed after incoming-edge neighbor expansion.
      refactor: `cargo test` passed full suite.

    - milestone12_diff_impact_includes_tests
      red: `cargo test milestone12_diff_impact_includes_tests -- --nocapture`
      failed with missing `result_kind = "test_target"` row.
      green: `cargo test milestone12_diff_impact_includes_tests -- --nocapture`
      passed after test-target enrichment for impacted symbols.
      refactor: `cargo test` passed full suite.

    - milestone12_diff_impact_deterministic_ordering
      red: `cargo test milestone12_diff_impact_deterministic_ordering -- --nocapture`
      failed because `--max-distance 0` still returned distance-1 results.
      green: `cargo test milestone12_diff_impact_deterministic_ordering -- --nocapture`
      passed after `max_distance` gating and deterministic tie-break updates.
      refactor: `cargo test` passed full suite.

    - dogfood pre/post evidence captured for every milestone12 slice with required commands and
      milestone-level manual checks (`diff-impact`/`explain` terminal + JSON runs).

    - milestone13_explain_definition_summary
      red: `cargo test milestone13_explain_definition_summary -- --nocapture`
      failed on missing terminal signature output.
      green: `cargo test milestone13_explain_definition_summary -- --nocapture`
      passed after adding per-result `signature:` lines in `print_explain`.
      refactor: `cargo test` passed full suite.

    - milestone13_explain_relationship_summary
      red: `cargo test milestone13_explain_relationship_summary -- --nocapture`
      failed with missing `inbound: called_by=1` text.
      green: `cargo test milestone13_explain_relationship_summary -- --nocapture`
      passed after grouped inbound/outbound edge counting for explain results.
      refactor: `cargo test` passed full suite.

    - milestone13_explain_json_determinism
      red: `cargo test milestone13_explain_json_determinism -- --nocapture`
      failed on `snippet.contains("leaf();")`.
      green: `cargo test milestone13_explain_json_determinism -- --nocapture`
      passed after Rust symbol spans switched to full AST node ranges.
      refactor: `cargo test` passed full suite.

    - dogfood pre/post evidence captured for every milestone13 slice with required commands and
      milestone-level manual checks (`diff-impact`/`explain` terminal + JSON runs).
    - milestone14_language_adapter_trait_migration
      red: `cargo test milestone14_language_adapter_trait_migration -- --nocapture`
      failed on `indexing should route language extraction through adapters`.
      green: `cargo test milestone14_language_adapter_trait_migration -- --nocapture`
      passed after indexer extraction moved behind adapter-only entrypoints.
      refactor: `cargo test` passed full suite.

    - milestone14_rust_behavior_unchanged_through_adapter
      red: `cargo test milestone14_rust_behavior_unchanged_through_adapter -- --nocapture`
      failed on missing `[ast_reference ast_likely]` after adapter migration.
      green: `cargo test milestone14_rust_behavior_unchanged_through_adapter -- --nocapture`
      passed after `ExtractionUnit.references` persisted `ast_references`.
      refactor: `cargo test` passed full suite.

    - milestone14_schema_language_metadata_migration
      red: `cargo test milestone14_schema_language_metadata_migration -- --nocapture`
      failed on `schema_version: 3` expectation and missing metadata columns.
      green: `cargo test milestone14_schema_language_metadata_migration -- --nocapture`
      passed after additive schema v3 migration and metadata backfill/index creation.
      refactor: `cargo test` passed full suite.

    - milestone15_typescript_definitions
      red: `cargo test milestone15_typescript_definitions -- --nocapture`
      failed on missing TypeScript AST definition match for `helper`.
      green: `cargo test milestone15_typescript_definitions -- --nocapture`
      passed after adding `TypeScriptLanguageAdapter` definition extraction.
      refactor: `cargo test` passed full suite.

    - milestone15_typescript_references_and_calls
      red: `cargo test milestone15_typescript_references_and_calls -- --nocapture`
      failed with missing `called_by` impact evidence for callable variable wrappers.
      green: `cargo test milestone15_typescript_references_and_calls -- --nocapture`
      passed after arrow/function-expression caller resolution and variable symbol extraction.
      refactor: `cargo test` passed full suite.

    - milestone15_typescript_edges_and_queries
      red: `cargo test milestone15_typescript_edges_and_queries -- --nocapture`
      failed with missing `imported_by`/`implemented_by` graph evidence.
      green: `cargo test milestone15_typescript_edges_and_queries -- --nocapture`
      passed after import-binding edge emission plus deferred cross-file edge resolution.
      refactor: `cargo test` passed full suite.

    - dogfood pre/post evidence captured for every milestone14 and milestone15 slice with required
      commands; milestone15 manual checks (`diff-impact` and `explain` terminal + JSON) completed.
    - milestone16_python_definitions
      red: `cargo test milestone16_python_definitions -- --nocapture`
      failed on missing Python AST definition evidence (`assertion failed:
      find_out.contains("[ast_definition ast_exact]")`).
      green: `cargo test milestone16_python_definitions -- --nocapture`
      passed after adding `PythonLanguageAdapter` definitions (class/function/method/constant) and
      adapter registration in `extract_with_adapter`.
      refactor: `cargo test` passed full suite.

    - milestone16_python_references_calls_imports
      red: `cargo test milestone16_python_references_calls_imports -- --nocapture`
      failed on missing `[ast_reference ast_likely]` for Python call sites.
      green: `cargo test milestone16_python_references_calls_imports -- --nocapture`
      passed after Python call/import parsing emitted `ast_references`, `calls`, and `imports`.
      refactor: `cargo test` passed full suite.

    - milestone16_python_edges_and_queries
      red: `cargo test milestone16_python_edges_and_queries -- --nocapture`
      failed because `refs helper --json` lacked `ast_reference` rows for import-driven usage.
      green: `cargo test milestone16_python_edges_and_queries -- --nocapture`
      passed after import bindings also emitted `ast_references` for imported symbols.
      refactor: `cargo test` passed full suite.

    - dogfood pre/post evidence captured for every milestone16 slice with required commands;
      milestone16 manual checks (`diff-impact` and `explain` terminal + JSON) completed.

## Interfaces and Dependencies

Phase 3 keeps existing dependencies (`tree-sitter`, `tree-sitter-rust`, `rusqlite`, `serde`, `clap`)
and may add `tree-sitter-typescript` and `tree-sitter-python` when language milestones start. Any
new dependency must be justified in `Decision Log` with deterministic-output implications.

In `src/cli.rs`, add:

    DiffImpact(DiffImpactArgs)
    Explain(ExplainArgs)

Define:

    pub struct DiffImpactArgs {
        #[arg(long = "changed-file", required = true)]
        pub changed_files: Vec<String>,
        #[arg(long, default_value_t = 2)]
        pub max_distance: u32,
        #[arg(long, default_value_t = true)]
        pub include_tests: bool,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
    }

    pub struct ExplainArgs {
        pub symbol: String,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
        #[arg(long, default_value_t = false)]
        pub include_snippets: bool,
    }

In `src/query/mod.rs`, add:

    pub fn diff_impact_for_changed_files(
        db_path: &Path,
        changed_files: &[String],
        max_distance: u32,
        include_tests: bool,
    ) -> anyhow::Result<Vec<DiffImpactMatch>>;

    pub fn explain_symbol(
        db_path: &Path,
        symbol: &str,
        include_snippets: bool,
    ) -> anyhow::Result<Vec<ExplainMatch>>;

Normalized language adapter contract (new module under `src/indexer/languages/`):

    pub trait LanguageAdapter {
        fn language_id(&self) -> &'static str;
        fn file_extensions(&self) -> &'static [&'static str];
        fn extract(&self, file_path: &str, source: &str) -> anyhow::Result<ExtractionUnit>;
    }

    pub struct ExtractionUnit {
        pub symbols: Vec<ExtractedSymbol>,
        pub edges: Vec<ExtractedEdge>,
    }

    pub struct ExtractedSymbol {
        pub symbol: String,
        pub qualified_symbol: Option<String>,
        pub kind: String,
        pub language: String,
        pub container: Option<String>,
        pub start_line: u32,
        pub start_column: u32,
        pub end_line: u32,
        pub end_column: u32,
        pub signature: Option<String>,
    }

    pub struct ExtractedEdge {
        pub from_symbol_key: SymbolKey,
        pub to_symbol_key: SymbolKey,
        pub edge_kind: String,
        pub confidence: f64,
        pub provenance: String,
    }

`src/store/schema.rs` migration target (schema v3) must be additive:

- Add `language TEXT NOT NULL DEFAULT 'unknown'` and `qualified_symbol TEXT` to `symbols_v2`.
- Add `provenance TEXT NOT NULL DEFAULT 'ast'` to `symbol_edges_v2`.
- Add indices on `(language, symbol)` and `qualified_symbol`.

`src/output.rs` additions:

    pub fn print_diff_impact(...)
    pub fn print_diff_impact_json(...) -> anyhow::Result<()>
    pub fn print_explain(...)
    pub fn print_explain_json(...) -> anyhow::Result<()>

Vocabulary contracts for Phase 3:

- `language`: `rust | typescript | python | unknown`
- `relationship`: `changed_symbol | called_by | contained_by | imported_by | implemented_by | tests`
- `confidence`: `graph_exact | graph_likely | context_high | context_medium | context_low`
- `provenance`:
  `ast_definition | ast_reference | import_resolution | call_resolution | text_fallback`

## Revision Note

2026-02-06: Created this Phase 3 planning-only ExecPlan to lock command-first sequencing
(`diff-impact`, `explain`) and define TypeScript/Python-compatible interfaces before implementation.
Chosen approach emphasizes deterministic contracts, strict per-slice TDD, additive schema evolution,
and backward compatibility with existing v1/v2 outputs.

2026-02-06: Updated the living plan after Milestone 11 implementation to record completed slices,
strict TDD red/green/refactor evidence, dogfooding transcripts, and decisions to emit changed-symbol
diff-impact rows and explain snippets in schema v3 outputs.

2026-02-06: Updated the living plan after Milestone 12 implementation to capture changed-file
normalization fixes, graph-neighbor/test-target expansion, `max_distance` enforcement, deterministic
ordering rules, and full red/green/refactor transcript evidence for all milestone12 slices.

2026-02-06: Updated the living plan after Milestone 13 implementation to record explain dossier
improvements (signature output, inbound/outbound summaries, deterministic JSON snippets) and the
strict TDD/dogfooding evidence for all milestone13 slices.

2026-02-07: Updated the living plan after Milestone 14 and Milestone 15 implementation to capture
adapter-boundary migration outcomes, schema v3 metadata migration evidence, TypeScript adapter MVP
behavior, cross-file deferred edge resolution decisions, and strict TDD/dogfooding transcripts for
all milestone14/15 slices.

2026-02-07: Updated the living plan after Milestone 16 implementation to capture Python adapter MVP
outcomes (definitions/references/calls/imports/contains), deterministic command behavior across
`find`/`refs`/`diff-impact`/`explain`, and strict red/green/refactor plus dogfooding evidence for
all milestone16 slices.

2026-02-07: Updated the living plan after Milestone 17 documentation finalization to record schema
3 command doc alignment, architecture/dogfood transcript refreshes, and completed Phase 3 closure
state.
