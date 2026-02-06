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
- [ ] Milestone 12 feature implementation (`diff-impact` behavior in terminal + JSON with
      deterministic ranking and confidence/provenance).
- [ ] Milestone 13 feature implementation (`explain` behavior in terminal + JSON with symbol dossier
      output).
- [ ] Milestone 14 refactor: language adapter boundary extraction with Rust adapter migration and no
      behavior regression.
- [ ] Milestone 15 TypeScript adapter MVP (definitions, references, imports, calls, containers).
- [ ] Milestone 16 Python adapter MVP (definitions, references, imports, calls, containers).
- [ ] Milestone 17 documentation and dogfood transcript updates (`README.md`,
      `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `docs/dogfood-log.md`).

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

## Outcomes & Retrospective

Planning outcome at this stage: Phase 3 scope is explicitly sequenced around agent-loop value
(`diff-impact`, `explain`) with contract locking first, then adapter extraction, then
TypeScript/Python rollout. No implementation has been performed yet under this plan.

Milestone 11 outcome: `diff-impact` and `explain` commands now exist with schema v3 JSON envelopes
and contract tests in `tests/milestone11_contracts.rs`. `diff-impact` currently returns changed-file
symbol definitions (distance 0), and `explain` returns definition dossiers with optional snippets.
Milestone 12+ behavior (graph neighbors, test targets, richer relationship counts) remains pending.

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

    - red/green/refactor transcripts for all milestone12_* tests (pending).
    - red/green/refactor transcripts for all milestone13_* tests.
    - red/green/refactor transcripts for all milestone14_* tests.
    - red/green/refactor transcripts for all milestone15_* tests.
    - red/green/refactor transcripts for all milestone16_* tests.

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
