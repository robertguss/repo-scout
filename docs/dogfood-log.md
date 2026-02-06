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
