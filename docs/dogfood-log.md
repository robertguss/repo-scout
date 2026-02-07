# Dogfood Log

This log captures real `repo-scout` usage while building `repo-scout`.

## Entry Template

- Date: `YYYY-MM-DD`
- Task:
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find <symbol> --repo . --json`
  - `cargo run -- refs <symbol> --repo . --json`
- What helped:
- What failed or felt weak:
- Action taken:
  - failing test added:
  - fix commit:
  - docs update:
- Status: `open` | `fixed` | `deferred`

## Entries

- Date: `2026-02-06`
- Task: Milestone 6 lifecycle + schema migration correctness.
- Commands run:
  - `just dogfood-pre launch`
  - `just tdd-red milestone6_delete_prunes_rows`
  - `just tdd-green milestone6_delete_prunes_rows`
  - `just tdd-refactor`
  - `just dogfood-post deletable_symbol`
- What helped:
  - Incremental hashing + pruning logic was straightforward to validate with fixture repos.
- What failed or felt weak:
  - Deleted paths originally remained queryable after reindex.
- Action taken:
  - failing test added: `tests/milestone6_lifecycle.rs`
  - fix commit: lifecycle pruning in indexer path replacement flow.
  - docs update: schema/lifecycle notes in `docs/architecture.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 12 `diff-impact` command rollout and deterministic ranking checks.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json`
- What helped:
  - Existing `symbols_v2` + `symbol_edges_v2` relationships made changed-file impact expansion
    straightforward.
- What failed or felt weak:
  - Absolute path aliases (`/var` vs `/private/var`) briefly duplicated changed files.
- Action taken:
  - failing test added: `tests/milestone12_diff_impact.rs::milestone12_diff_impact_changed_files_normalization`
  - fix commit: canonicalize changed-file paths before dedupe and ranking.
  - docs update: `docs/cli-reference.md` + `docs/json-output.md` schema 3 command section.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 13 `explain` command dossier and JSON determinism checks.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- explain impact_matches --repo .`
  - `cargo run -- explain impact_matches --repo . --json`
  - `cargo run -- explain impact_matches --repo . --include-snippets --json`
- What helped:
  - Reusing symbol-edge counters gave deterministic inbound/outbound relationship summaries.
- What failed or felt weak:
  - Identifier-only spans produced low-value snippets.
- Action taken:
  - failing test added: `tests/milestone13_explain.rs::milestone13_explain_json_determinism`
  - fix commit: persist full-node Rust spans so snippets include meaningful code context.
  - docs update: `docs/json-output.md` explain schema 3 contract notes.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 15 TypeScript adapter MVP dogfood.
- Commands run:
  - `cargo test milestone15_typescript_definitions -- --nocapture`
  - `cargo test milestone15_typescript_references_and_calls -- --nocapture`
  - `cargo test milestone15_typescript_edges_and_queries -- --nocapture`
  - `cargo run -- index --repo .`
  - `cargo run -- find TypeScriptLanguageAdapter --repo .`
  - `cargo run -- refs TypeScriptLanguageAdapter --repo .`
- What helped:
  - Adapter boundary kept query/output logic unchanged while adding `.ts/.tsx` extraction.
- What failed or felt weak:
  - Cross-file import/implements edges dropped when targets were indexed later.
- Action taken:
  - failing test added: `tests/milestone15_typescript.rs::milestone15_typescript_edges_and_queries`
  - fix commit: deferred edge-resolution pass after full indexing plus deterministic import parsing.
  - docs update: `docs/architecture.md` adapter-boundary and cross-file edge notes.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 16 Python adapter MVP dogfood.
- Commands run:
  - `cargo test milestone16_python_definitions -- --nocapture`
  - `cargo test milestone16_python_references_calls_imports -- --nocapture`
  - `cargo test milestone16_python_edges_and_queries -- --nocapture`
  - `cargo run -- index --repo .`
  - `cargo run -- find PythonLanguageAdapter --repo .`
  - `cargo run -- refs PythonLanguageAdapter --repo .`
- What helped:
  - Python adapter achieved `find`/`refs`/`impact`/`diff-impact`/`explain` coverage without query-layer
    branching.
- What failed or felt weak:
  - Import-driven references were initially text-only for `refs`.
- Action taken:
  - failing test added: `tests/milestone16_python.rs::milestone16_python_edges_and_queries`
  - fix commit: emit import-binding `ast_references` alongside `imports` edges.
  - docs update: `README.md`, `docs/cli-reference.md`, and `docs/json-output.md`.
- Status: `fixed`

- Date: `2026-02-06`
- Task: Milestone 8 graph model correctness.
- Commands run:
  - `just dogfood-pre run`
  - `just tdd-red milestone8_call_and_contains_edges`
  - `just tdd-green milestone8_call_and_contains_edges`
  - `just tdd-refactor`
  - `just dogfood-post run`
- What helped:
  - Deterministic edge upserts made repeated indexing stable.
- What failed or felt weak:
  - Early symbol-ID reuse strategy hit uniqueness conflicts.
- Action taken:
  - failing test added: `tests/milestone8_graph.rs`
  - fix commit: stable ID reuse + deterministic fallback generation.
  - docs update: graph architecture and edge semantics.
- Status: `fixed`

- Date: `2026-02-06`
- Task: Milestone 9 + 10 agent-native query and validation commands.
- Commands run:
  - `just tdd-red milestone9_impact_json_schema`
  - `just tdd-green milestone9_impact_json_schema`
  - `just tdd-red milestone9_context_json_schema`
  - `just tdd-green milestone9_context_json_schema`
  - `just tdd-red milestone10_verify_plan_changed_files`
  - `just tdd-green milestone10_verify_plan_changed_files`
  - `just dogfood-post compute_plan`
- What helped:
  - Existing symbols/edges tables enabled small, focused query additions.
- What failed or felt weak:
  - Recommendation list briefly suggested non-runnable nested test modules.
- Action taken:
  - failing test added: `tests/milestone10_validation.rs::milestone10_verify_plan_skips_non_runnable_test_modules`
  - fix commit: restrict runnable target extraction to direct `tests/<file>.rs` files.
  - docs update: `docs/cli-reference.md` and `docs/json-output.md`.
- Status: `fixed`

- Date: `2026-02-06`
- Task: Post-merge docs refresh for implemented Phase 2 behavior.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find main --repo . --json`
  - `cargo run -- refs main --repo . --json`
  - `cargo run -- impact run --repo . --json`
  - `cargo run -- context --task "update run and verify refs behavior" --repo . --budget 400 --json`
  - `cargo run -- tests-for run --repo . --json`
  - `cargo run -- verify-plan --changed-file src/query/mod.rs --repo . --json`
- What helped:
  - Deterministic JSON and terminal output made command-contract documentation straightforward.
- What failed or felt weak:
  - No functional defects found during this documentation pass.
- Action taken:
  - failing test added: none.
  - fix commit: none (docs/Justfile refresh only).
  - docs update: README + all user/contributor docs under `docs/`.
- Status: `fixed`
