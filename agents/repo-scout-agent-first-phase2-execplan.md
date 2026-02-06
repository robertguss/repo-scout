# Build `repo-scout` Agent-First Graph and Task Query Platform (Phase 2)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `/Users/robertguss/Projects/experiments/repo-scout/agents/PLANS.md`, and this document must be maintained in accordance with that file.

This plan builds on `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-hybrid-rust-execplan.md`, which completed v0 (`index`, `status`, `find`, `refs`) and established deterministic JSON output plus baseline Rust AST extraction.

## Purpose / Big Picture

The next phase makes `repo-scout` meaningfully better for coding agents and humans by turning it into a reliable code-intelligence substrate rather than a pure symbol matcher. After this change, a user can ask for impact analysis, request a compact context bundle for an editing task, and get targeted validation guidance tied to changed symbols. The user can still run old commands, but now gets richer symbol metadata, relationship edges between symbols, and freshness guarantees that prevent stale results after files are renamed or deleted.

A user should be able to see this working through concrete commands: index a repository, query symbol definitions and references, ask for transitive impact for a symbol, and ask for a task context bundle that returns ranked snippets plus explicit reasons for inclusion. The output must remain deterministic and machine-stable so agent workflows can trust it.

## Progress

- [x] (2026-02-06 00:00Z) Reviewed current repository state, prior v0 ExecPlan, and `/Users/robertguss/Projects/experiments/repo-scout/agents/PLANS.md` requirements.
- [x] (2026-02-06 00:00Z) Drafted this Phase 2 agent-first ExecPlan with full milestone narrative, validation commands, and interface contracts.
- [x] (2026-02-06 00:00Z) Added strict per-feature TDD gates (red, green, refactor) and evidence requirements for each milestone.
- [x] (2026-02-06 01:10Z) Implemented Milestone 6 lifecycle correctness and schema migration guards (`tests/milestone6_lifecycle.rs`, `tests/milestone6_schema_migration.rs`, `src/indexer/mod.rs`, `src/store/schema.rs`) with full-suite green gate.
- [x] (2026-02-06 01:40Z) Implemented Milestone 7 richer Rust symbol extraction and metadata persistence (`tests/milestone7_rust_symbols.rs`, `src/indexer/rust_ast.rs`, `src/indexer/mod.rs`) with full-suite green gate.
- [x] (2026-02-06 02:15Z) Implemented Milestone 8 graph storage primitives with stable symbol IDs and edge persistence (`tests/milestone8_graph.rs`, `src/indexer/mod.rs`, `src/indexer/rust_ast.rs`) with full-suite green gate.
- [ ] Implement Milestone 9: agent-native commands (`impact`, `context`) with deterministic JSON contracts.
- [ ] Implement Milestone 10: validation intelligence (`tests-for`, `verify-plan`) and end-to-end acceptance hardening.
- [ ] Finalize documentation updates, revision notes, and retrospective once implementation completes.

## Surprises & Discoveries

- Observation: The current indexer leaves stale rows for deleted files.
  Evidence: Running `index`, deleting a file, then running `index` again still returns the deleted file in `find` results (`results: 1`, `a.txt:1:1 alpha ...`) while index summary reports `indexed_files: 0` and `skipped_files: 0`.

- Observation: The v0 query model is deterministic and stable, which is valuable and should be preserved as a non-negotiable contract for any new commands.
  Evidence: Existing milestone tests lock output ordering and schema behavior for `find` and `refs`.

- Observation: Tree-sitter parsing is already integrated for Rust and can be expanded incrementally without introducing new parser infrastructure.
  Evidence: `src/indexer/rust_ast.rs` already extracts `function_item` definitions and `call_expression` identifiers with source locations.

- Observation: One stale-row pruning primitive in `index_repository` covered both delete and rename lifecycle correctness; rename and deterministic-count tests were already green after slice 6A.
  Evidence: `just tdd-red milestone6_rename_prunes_old_path` and `just tdd-red milestone6_lifecycle_counts_are_deterministic` passed immediately after the delete-pruning implementation.

- Observation: Migrating store schema to v2 only impacted `index/status` schema reporting; `find`/`refs --json` remained schema-version 1 and behavior-compatible.
  Evidence: `tests/milestone1_cli.rs` required schema assertion updates to `2`, while `tests/milestone4_ranking_json.rs` and `tests/milestone5_e2e.rs` continued to assert JSON schema `1` successfully.

- Observation: Import extraction introduces legitimate multiple AST definitions for some symbols (for example a local struct plus a `use` import of the same name), so tests should assert non-zero AST results rather than hard-coding single-result counts.
  Evidence: `milestone7_struct_enum_trait_defs` initially expected `results: 1` for `Launcher` and failed once `use` extraction was added in slice 7C.

- Observation: Stable symbol IDs required explicit ID reuse plus deterministic allocation for newly introduced symbols in a changed file.
  Evidence: `milestone8_symbol_upsert_stable_ids` initially failed with `left: 9 right: 10`, and a naive ID-reuse pass hit `UNIQUE constraint failed: symbols_v2.symbol_id` until new symbols were assigned IDs above current max.

## Decision Log

- Decision: Prioritize trust and freshness correctness before adding richer intelligence features.
  Rationale: Agent quality drops sharply with stale index data; “not found” and “changed impact” must be trustworthy before adding advanced query surfaces.
  Date/Author: 2026-02-06 / Codex

- Decision: Keep v0 commands backward-compatible while introducing new agent-native commands.
  Rationale: Existing users and tests must remain valid; additive CLI growth is safer than command behavior replacement.
  Date/Author: 2026-02-06 / Codex

- Decision: Introduce a first-class symbol graph model in SQLite, with explicit edge types and stable symbol identifiers.
  Rationale: Agent tasks such as impact analysis and context assembly need graph traversals, not flat match lists.
  Date/Author: 2026-02-06 / Codex

- Decision: Keep confidence and provenance explicit in all new outputs and extend the existing convention.
  Rationale: Agents need calibrated certainty to avoid overconfident edits; users need transparent explanations.
  Date/Author: 2026-02-06 / Codex

- Decision: Add prototyping slices for high-ambiguity features (`context` ranking and `verify-plan` recommendations) before finalizing schema contracts.
  Rationale: These features involve heuristics; a prototype-first milestone reduces rework and improves measurable behavior.
  Date/Author: 2026-02-06 / Codex

- Decision: Enforce strict TDD at the feature-slice level, not only milestone level.
  Rationale: Milestone-level loops are too coarse and can allow untested feature additions; per-feature red-green-refactor keeps changes small, test-led, and auditable.
  Date/Author: 2026-02-06 / Codex

- Decision: Bump SQLite store schema metadata to v2 while preserving existing `find`/`refs` JSON schema version at v1.
  Rationale: Phase 2 needs additive graph tables and migration guards, but existing query JSON contracts are explicitly preserved for compatibility.
  Date/Author: 2026-02-06 / Codex

- Decision: Persist rich symbol metadata in `symbols_v2` while keeping `find` backed by `ast_definitions` for compatibility.
  Rationale: Phase 2 needs container/span/signature data for new graph/query commands, but preserving existing `find` behavior avoids destabilizing earlier milestone contracts.
  Date/Author: 2026-02-06 / Codex

- Decision: Build `imports` and `implements` edges from lightweight source-line relation hints during indexing, while deriving `calls` and `contains` from AST-extracted metadata.
  Rationale: This keeps Milestone 8 additive and testable without destabilizing `find`/`refs` data paths, while still producing deterministic edge coverage needed by Milestone 9 queries.
  Date/Author: 2026-02-06 / Codex

## Outcomes & Retrospective

Initial outcome at planning time: the repository has a solid v0 baseline and deterministic behavior, but no graph-level understanding, no task-shaped commands, and a correctness gap for deleted files. This plan addresses those gaps in an incremental path that preserves existing behavior and test contracts.

Target completion outcome: `repo-scout` becomes a dependable local intelligence tool for coding workflows with trustworthy incremental indexing, richer symbol coverage, graph-powered impact/context queries, and actionable validation suggestions. Remaining future work after this plan should be deeper semantic resolution (for example compiler-assisted type resolution), additional languages, and ranking quality iteration based on real repository telemetry.

Milestone 6 outcome (2026-02-06): lifecycle correctness is now enforced for file deletion and rename scenarios, and schema migration guards validate upgrade from v1 to v2 without losing query behavior. Remaining work is entirely Phase 2 intelligence expansion (Milestones 7-10).

Milestone 7 outcome (2026-02-06): Rust symbol extraction now covers struct/enum/trait/module/type-alias/const/import plus impl-method container metadata, and `symbols_v2` now persists container/span/signature summaries for downstream graph and agent-query milestones.

Milestone 8 outcome (2026-02-06): graph primitives now persist deterministic `calls`, `contains`, `imports`, and `implements` edges in `symbol_edges_v2`, and symbol IDs remain stable across same-identity reindex updates, enabling graph-backed agent query work in Milestone 9.

## Context and Orientation

`repo-scout` is a Rust CLI with command parsing in `/Users/robertguss/Projects/experiments/repo-scout/src/cli.rs`, top-level dispatch in `/Users/robertguss/Projects/experiments/repo-scout/src/main.rs`, indexing logic in `/Users/robertguss/Projects/experiments/repo-scout/src/indexer/`, querying in `/Users/robertguss/Projects/experiments/repo-scout/src/query/`, and SQLite schema/store handling in `/Users/robertguss/Projects/experiments/repo-scout/src/store/`. Integration tests live under `/Users/robertguss/Projects/experiments/repo-scout/tests/`.

This plan uses several terms that must be explicit. A “symbol” means a named program element such as a function, struct, enum, trait, method, module, or imported name. A “span” means a source range, including where the symbol starts and ends in line/column coordinates. An “edge” means a directional relationship between two symbols, such as “A calls B” or “file X imports symbol Y.” A “symbol graph” means symbols plus edges stored in queryable tables. “Impact analysis” means finding likely downstream symbols/files affected if one symbol changes. A “context bundle” means a capped, ranked set of snippets and metadata selected to help complete a coding task inside a limited token budget.

The current implementation already stores text occurrences and limited Rust AST entries. It does not store symbol identity beyond raw rows, has no edge graph, and does not expose impact/context commands. The plan adds these capabilities while preserving deterministic sorting and JSON stability.

## Strict TDD Contract

This plan is strict red-green-refactor for every feature slice. A “feature slice” means a smallest user-visible behavior unit, such as “deleted file rows are pruned” or “`impact --json` emits stable schema fields.” No production code for a slice may be written until a failing automated test exists for that exact slice. A green step means only enough code to pass that failing test. A refactor step means improving structure without changing behavior, while all tests remain green.

For each slice, the contributor must record three artifacts in this document: a red transcript that fails for the intended reason, a green transcript where that specific test passes, and a refactor transcript where the full suite passes. If a slice changes scope, update `Decision Log` and split the slice into two explicit items in `Progress`.

## Plan of Work

Milestone 6 fixes index lifecycle correctness through three feature slices executed in strict TDD order. Slice 6A proves stale rows are removed when a file is deleted. Slice 6B proves rows migrate correctly when a file is renamed. Slice 6C proves lifecycle counts and output remain deterministic and backward compatible. Add tests first in `/Users/robertguss/Projects/experiments/repo-scout/tests/milestone6_lifecycle.rs`, then implement reconciliation in `/Users/robertguss/Projects/experiments/repo-scout/src/indexer/mod.rs` with transactional pruning across `indexed_files`, `text_occurrences`, `ast_definitions`, and `ast_references`.

Milestone 7 expands Rust AST extraction through four slices. Slice 7A adds `struct`, `enum`, and `trait` definitions. Slice 7B adds `impl` and method extraction with container metadata. Slice 7C adds module, alias, constant, and import symbol extraction. Slice 7D adds span and signature metadata storage and output wiring. Tests live in `/Users/robertguss/Projects/experiments/repo-scout/tests/milestone7_rust_symbols.rs`; implementation lives in `/Users/robertguss/Projects/experiments/repo-scout/src/indexer/rust_ast.rs`, `/Users/robertguss/Projects/experiments/repo-scout/src/indexer/mod.rs`, and `/Users/robertguss/Projects/experiments/repo-scout/src/store/schema.rs`.

Milestone 8 introduces graph storage through three slices. Slice 8A creates canonical symbol table upserts with stable IDs. Slice 8B stores `calls` and `contains` edges. Slice 8C adds `imports` and `implements` edges and deterministic retrieval ordering. Tests live in `/Users/robertguss/Projects/experiments/repo-scout/tests/milestone8_graph.rs`; implementation lives in new graph modules under `/Users/robertguss/Projects/experiments/repo-scout/src/query/` or `/Users/robertguss/Projects/experiments/repo-scout/src/graph/` plus index-time wiring in `/Users/robertguss/Projects/experiments/repo-scout/src/indexer/mod.rs`.

Milestone 9 adds agent-native task queries through four slices. Slice 9A introduces CLI routing for `impact` and terminal output. Slice 9B adds `impact --json` deterministic schema. Slice 9C introduces `context` ranking with budget enforcement. Slice 9D adds `context --json` with explicit `why_included` reasons. Tests live in `/Users/robertguss/Projects/experiments/repo-scout/tests/milestone9_agent_queries.rs`; routing is in `/Users/robertguss/Projects/experiments/repo-scout/src/cli.rs` and `/Users/robertguss/Projects/experiments/repo-scout/src/main.rs`; query logic and ranking live in `/Users/robertguss/Projects/experiments/repo-scout/src/query/`.

Milestone 10 adds validation intelligence through four slices. Slice 10A introduces `tests-for` for direct symbol-to-test mapping. Slice 10B improves `tests-for` with confidence tiers and deduplication. Slice 10C introduces `verify-plan` for changed files. Slice 10D refines `verify-plan` into deterministic recommended command sets with rationale strings. Tests live in `/Users/robertguss/Projects/experiments/repo-scout/tests/milestone10_validation.rs`; implementation lives in `/Users/robertguss/Projects/experiments/repo-scout/src/query/`, `/Users/robertguss/Projects/experiments/repo-scout/src/output.rs`, and CLI wiring files.

Throughout all milestones, preserve current v0 command behavior and JSON schema compatibility for existing fields. If schema version increments are required, implement explicit migration steps in `/Users/robertguss/Projects/experiments/repo-scout/src/store/schema.rs` and maintain readable compatibility notes in docs.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/repo-scout`.

Use this exact loop for every feature slice. The red step must fail before any production edit is allowed for that slice.

Feature-slice loop template:

    cargo test <slice_test_name> -- --nocapture
    # confirm failure for expected reason and capture transcript
    cargo test <slice_test_name> -- --nocapture
    # after minimal implementation, confirm pass and capture transcript
    cargo test
    # refactor gate: full suite must pass with no behavior change

Milestone 6 strict slices:

    cargo test milestone6_delete_prunes_rows -- --nocapture
    cargo test milestone6_delete_prunes_rows -- --nocapture
    cargo test
    cargo test milestone6_rename_prunes_old_path -- --nocapture
    cargo test milestone6_rename_prunes_old_path -- --nocapture
    cargo test
    cargo test milestone6_lifecycle_counts_are_deterministic -- --nocapture
    cargo test milestone6_lifecycle_counts_are_deterministic -- --nocapture
    cargo test

Expected milestone-6 acceptance transcript after green:

    running 3 tests
    test milestone6_delete_prunes_rows ... ok
    test milestone6_rename_prunes_old_path ... ok
    test milestone6_lifecycle_counts_are_deterministic ... ok
    test result: ok. 3 passed; 0 failed

Milestone 7 strict slices:

    cargo test milestone7_struct_enum_trait_defs -- --nocapture
    cargo test milestone7_struct_enum_trait_defs -- --nocapture
    cargo test
    cargo test milestone7_impl_method_container -- --nocapture
    cargo test milestone7_impl_method_container -- --nocapture
    cargo test
    cargo test milestone7_module_alias_const_use -- --nocapture
    cargo test milestone7_module_alias_const_use -- --nocapture
    cargo test
    cargo test milestone7_spans_and_signatures_persist -- --nocapture
    cargo test milestone7_spans_and_signatures_persist -- --nocapture
    cargo test

Expected milestone-7 acceptance transcript after green:

    running 4 tests
    test milestone7_struct_enum_trait_defs ... ok
    test milestone7_impl_method_container ... ok
    test milestone7_module_alias_const_use ... ok
    test milestone7_spans_and_signatures_persist ... ok
    test result: ok. 4 passed; 0 failed

Milestone 8 strict slices:

    cargo test milestone8_symbol_upsert_stable_ids -- --nocapture
    cargo test milestone8_symbol_upsert_stable_ids -- --nocapture
    cargo test
    cargo test milestone8_call_and_contains_edges -- --nocapture
    cargo test milestone8_call_and_contains_edges -- --nocapture
    cargo test
    cargo test milestone8_imports_and_implements_edges -- --nocapture
    cargo test milestone8_imports_and_implements_edges -- --nocapture
    cargo test

Expected milestone-8 acceptance transcript after green:

    running 3 tests
    test milestone8_symbol_upsert_stable_ids ... ok
    test milestone8_call_and_contains_edges ... ok
    test milestone8_imports_and_implements_edges ... ok
    test result: ok. 3 passed; 0 failed

Milestone 9 strict slices:

    cargo test milestone9_impact_terminal -- --nocapture
    cargo test milestone9_impact_terminal -- --nocapture
    cargo test
    cargo test milestone9_impact_json_schema -- --nocapture
    cargo test milestone9_impact_json_schema -- --nocapture
    cargo test
    cargo test milestone9_context_budgeted_terminal -- --nocapture
    cargo test milestone9_context_budgeted_terminal -- --nocapture
    cargo test
    cargo test milestone9_context_json_schema -- --nocapture
    cargo test milestone9_context_json_schema -- --nocapture
    cargo test

Manual scenario checks after milestone 9:

    cargo run -- index --repo .
    cargo run -- impact launch --repo .
    cargo run -- impact launch --repo . --json
    cargo run -- context --task "modify launch flow and update call sites" --repo . --budget 1200
    cargo run -- context --task "modify launch flow and update call sites" --repo . --json --budget 1200

Milestone 10 strict slices:

    cargo test milestone10_tests_for_direct_matches -- --nocapture
    cargo test milestone10_tests_for_direct_matches -- --nocapture
    cargo test
    cargo test milestone10_tests_for_dedup_confidence -- --nocapture
    cargo test milestone10_tests_for_dedup_confidence -- --nocapture
    cargo test
    cargo test milestone10_verify_plan_changed_files -- --nocapture
    cargo test milestone10_verify_plan_changed_files -- --nocapture
    cargo test
    cargo test milestone10_verify_plan_deterministic_recommendations -- --nocapture
    cargo test milestone10_verify_plan_deterministic_recommendations -- --nocapture
    cargo test

Manual scenario checks after milestone 10:

    cargo run -- tests-for launch --repo .
    cargo run -- verify-plan --changed-file src/query/mod.rs --repo .
    cargo run -- verify-plan --changed-file src/query/mod.rs --repo . --json

Before finalizing, run formatting and full tests:

    cargo fmt
    cargo test

## Validation and Acceptance

Acceptance is behavior-first and must be observable from terminal output or deterministic JSON. After Milestone 6, deleting a file and rerunning `index` must remove all query hits from the deleted file. A reproducible check is: index a temporary fixture repository, query a symbol, delete the containing file, rerun index, and verify `results: 0` for that symbol.

After Milestone 7, `find` for symbols defined as struct names, trait names, method names, and module names must return AST-based results with explicit `why_matched` and `confidence` values, and those values must be stable across runs.

After Milestone 8, graph-backed internals must be demonstrable through milestone tests that confirm expected edges exist, and by an `impact` command in Milestone 9 that returns first-order downstream symbols for simple fixture call chains.

After Milestone 9, `impact` and `context` must support `--json` with stable top-level fields and deterministic ordering. Every row in `context` output must include an explanation string describing why the snippet was included (for example direct definition match, direct call neighbor, or shared file with changed symbol).

After Milestone 10, `tests-for` and `verify-plan` must recommend a non-empty, deterministic command/file list for fixture changes, and must avoid duplicates. End-to-end tests must show that recommendations include both nearest tests and broader safety checks when confidence is low.

Final acceptance requires evidence of strict TDD for each feature slice: at least one red transcript, one green transcript, and one refactor full-suite transcript per slice in the Artifacts section or linked commit messages. Final acceptance also requires `cargo test` full-suite pass, preservation of existing v0 behavior (`find`, `refs`, `--json`), and updated docs in `/Users/robertguss/Projects/experiments/repo-scout/README.md`, `/Users/robertguss/Projects/experiments/repo-scout/docs/cli-reference.md`, `/Users/robertguss/Projects/experiments/repo-scout/docs/json-output.md`, and `/Users/robertguss/Projects/experiments/repo-scout/docs/architecture.md`.

## Idempotence and Recovery

Indexing must remain idempotent. Repeated runs with unchanged files must preserve row counts and ordering and must not duplicate symbols or edges. File lifecycle reconciliation in Milestone 6 must be safe to run repeatedly and must converge to the same state.

If schema changes require migration, use additive migration steps keyed by schema version in `/Users/robertguss/Projects/experiments/repo-scout/src/store/schema.rs`. A failed migration should leave the previous schema intact when possible; when not possible, emit an actionable error that names the index file path and recovery steps.

The existing corruption recovery behavior must remain intact. If the index file is corrupted, the tool must still instruct the user to delete the file and rerun `index`.

## Artifacts and Notes

Initial stale-delete reproduction transcript that motivates Milestone 6:

    $ cargo run -- index --repo /tmp/sample-repo
    index_path: /tmp/sample-repo/.repo-scout/index.db
    schema_version: 1
    indexed_files: 1
    skipped_files: 0
    $ cargo run -- find alpha --repo /tmp/sample-repo
    command: find
    query: alpha
    results: 1
    a.txt:1:1 alpha [exact_symbol_name text_fallback]
    $ rm /tmp/sample-repo/a.txt
    $ cargo run -- index --repo /tmp/sample-repo
    indexed_files: 0
    skipped_files: 0
    $ cargo run -- find alpha --repo /tmp/sample-repo
    command: find
    query: alpha
    results: 1
    a.txt:1:1 alpha [exact_symbol_name text_fallback]

Target `impact --json` shape after Milestone 9:

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

Target `context --json` shape after Milestone 9:

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

Strict TDD artifact checklist for each milestone (to be filled during implementation):

    Milestone 6:
    - red: milestone6_delete_prunes_rows fails with stale hit still present.
    - green: milestone6_delete_prunes_rows passes.
    - refactor: cargo test passes.
    - red: milestone6_rename_prunes_old_path fails with stale old-path rows.
    - green: milestone6_rename_prunes_old_path passes.
    - refactor: cargo test passes.
    - red: milestone6_lifecycle_counts_are_deterministic fails with unstable counts/order.
    - green: milestone6_lifecycle_counts_are_deterministic passes.
    - refactor: cargo test passes.

    Milestone 7:
    - red/green/refactor for milestone7_struct_enum_trait_defs.
    - red/green/refactor for milestone7_impl_method_container.
    - red/green/refactor for milestone7_module_alias_const_use.
    - red/green/refactor for milestone7_spans_and_signatures_persist.

    Milestone 8:
    - red/green/refactor for milestone8_symbol_upsert_stable_ids.
    - red/green/refactor for milestone8_call_and_contains_edges.
    - red/green/refactor for milestone8_imports_and_implements_edges.

    Milestone 9:
    - red/green/refactor for milestone9_impact_terminal.
    - red/green/refactor for milestone9_impact_json_schema.
    - red/green/refactor for milestone9_context_budgeted_terminal.
    - red/green/refactor for milestone9_context_json_schema.

    Milestone 10:
    - red/green/refactor for milestone10_tests_for_direct_matches.
    - red/green/refactor for milestone10_tests_for_dedup_confidence.
    - red/green/refactor for milestone10_verify_plan_changed_files.
    - red/green/refactor for milestone10_verify_plan_deterministic_recommendations.

## Interfaces and Dependencies

Use existing dependencies already present in `/Users/robertguss/Projects/experiments/repo-scout/Cargo.toml`: `tree-sitter`, `tree-sitter-rust`, `rusqlite`, `serde`, and `clap`. Add no new runtime dependency unless needed for deterministic ranking or parsing quality; if added, justify it in the decision log.

In `/Users/robertguss/Projects/experiments/repo-scout/src/cli.rs`, add command variants and argument structs for:

    Impact(QueryArgs)
    Context(ContextArgs)
    TestsFor(QueryArgs)
    VerifyPlan(VerifyPlanArgs)

Define:

    pub struct ContextArgs {
        pub task: String,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
        #[arg(long, default_value_t = 1200)]
        pub budget: usize,
    }

    pub struct VerifyPlanArgs {
        #[arg(long = "changed-file")]
        pub changed_files: Vec<PathBuf>,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
    }

In `/Users/robertguss/Projects/experiments/repo-scout/src/query/mod.rs` (or split submodules), define and expose:

    pub fn impact_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<ImpactMatch>>;
    pub fn context_matches(
        db_path: &Path,
        task: &str,
        budget: usize,
    ) -> anyhow::Result<Vec<ContextMatch>>;
    pub fn tests_for_symbol(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<TestTarget>>;
    pub fn verify_plan_for_changes(
        db_path: &Path,
        changed_files: &[PathBuf],
    ) -> anyhow::Result<Vec<VerificationStep>>;

In `/Users/robertguss/Projects/experiments/repo-scout/src/store/schema.rs`, introduce schema version 2 with additive tables:

    symbols_v2(
      symbol_id INTEGER PRIMARY KEY,
      file_path TEXT NOT NULL,
      symbol TEXT NOT NULL,
      kind TEXT NOT NULL,
      container TEXT,
      start_line INTEGER NOT NULL,
      start_column INTEGER NOT NULL,
      end_line INTEGER NOT NULL,
      end_column INTEGER NOT NULL,
      signature TEXT
    )

    symbol_edges_v2(
      edge_id INTEGER PRIMARY KEY,
      from_symbol_id INTEGER NOT NULL,
      to_symbol_id INTEGER NOT NULL,
      edge_kind TEXT NOT NULL,
      confidence REAL NOT NULL
    )

Create indexes on symbol text, file path, and `(from_symbol_id, edge_kind)` plus `(to_symbol_id, edge_kind)`.

In `/Users/robertguss/Projects/experiments/repo-scout/src/output.rs`, preserve existing output functions and add:

    pub fn print_impact(symbol: &str, matches: &[ImpactMatch]);
    pub fn print_impact_json(symbol: &str, matches: &[ImpactMatch]) -> anyhow::Result<()>;
    pub fn print_context(task: &str, budget: usize, matches: &[ContextMatch]);
    pub fn print_context_json(task: &str, budget: usize, matches: &[ContextMatch]) -> anyhow::Result<()>;
    pub fn print_tests_for(symbol: &str, targets: &[TestTarget]);
    pub fn print_verify_plan(changed_files: &[PathBuf], steps: &[VerificationStep]);

Confidence and provenance vocabulary must remain explicit and finite. Extend existing labels with:

    confidence: graph_exact, graph_likely, context_high, context_medium, context_low
    edge_kind: calls, called_by, defines, contains, imports, implements, tests

## Revision Note

2026-02-06: Created this new Phase 2 ExecPlan to define an agent-first roadmap after v0 completion. Chose a trust-first sequence (freshness before intelligence) and specified concrete milestones, commands, validation behavior, and interfaces so a novice can implement end-to-end from this single file.
2026-02-06: Revised the plan to enforce strict per-feature red-green-refactor workflow, added explicit feature slices per milestone, and added mandatory TDD evidence requirements so implementation mirrors the rigor of the prior v0 plan.
2026-02-06: Updated living sections after Milestone 6 implementation with lifecycle/migration outcomes, schema-version compatibility decisions, and TDD evidence notes for restartability.
2026-02-06: Updated living sections after Milestone 7 implementation with richer symbol extraction outcomes, metadata persistence decisions, and slice-level test evidence notes.
2026-02-06: Updated living sections after Milestone 8 implementation with stable symbol-ID strategy, edge-construction decisions, and graph slice evidence notes.
