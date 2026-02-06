# Dogfood Log

This log captures real usage of `repo-scout` while building `repo-scout`.
Each entry should describe what task we attempted, what worked, what failed, and what follow-up action we took (especially new failing tests and fixes).

## Entry Template

Copy this template for each new task:

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
  - docs/plan update:
- Status: `open` | `fixed` | `deferred`

## Entries

- Date: `2026-02-06`
- Task: Verify index freshness when files are removed.
- Commands run:
  - `cargo run -- index --repo <tmp-repo>`
  - `cargo run -- find alpha --repo <tmp-repo>`
  - delete `a.txt`
  - `cargo run -- index --repo <tmp-repo>`
  - `cargo run -- find alpha --repo <tmp-repo>`
- What helped:
  - Query output was deterministic and easy to compare before/after deletion.
- What failed or felt weak:
  - `find` still returned a deleted file (`a.txt`) after reindex.
- Action taken:
  - failing test added: planned for Phase 2 Milestone 6 lifecycle correctness.
  - fix commit: pending.
  - docs/plan update: added lifecycle-pruning milestone in `agents/repo-scout-agent-first-phase2-execplan.md`.
- Status: `open`

- Date: `2026-02-06`
- Task: Ensure planning process itself is dogfooded and strict-TDD by default.
- Commands run:
  - review and update planning docs
- What helped:
  - Existing ExecPlan structure made it straightforward to add per-slice TDD gates.
- What failed or felt weak:
  - Base template (`agents/PLANS.md`) did not explicitly enforce strict red-green-refactor per feature slice.
- Action taken:
  - failing test added: not applicable (process/documentation change).
  - fix commit: pending.
  - docs/plan update: updated `agents/PLANS.md` with strict TDD requirements and updated phase-2 plan accordingly.
- Status: `fixed`

- Date: `2026-02-06`
- Task: Milestone 6 lifecycle correctness and schema migration guardrails.
- Commands run:
  - `just dogfood-pre launch`
  - `just tdd-red milestone6_delete_prunes_rows`
  - `just tdd-green milestone6_delete_prunes_rows`
  - `just tdd-refactor`
  - `just dogfood-pre rename_symbol`
  - `just tdd-red milestone6_rename_prunes_old_path`
  - `just tdd-red milestone6_lifecycle_counts_are_deterministic`
  - `just tdd-red milestone6_schema_v1_upgrades_to_v2_without_data_loss`
  - `just tdd-green milestone6_schema_v1_upgrades_to_v2_without_data_loss`
  - `just tdd-refactor`
  - `just dogfood-post deletable_symbol`
- What helped:
  - Existing `just` workflows made per-slice red/green/refactor and dogfood loops fast to repeat.
  - A single stale-path pruning path in `src/indexer/mod.rs` fixed delete and rename lifecycle defects.
- What failed or felt weak:
  - Slice-level red for rename/deterministic-count checks was already green once delete pruning landed, so the primitive had broader coverage than initially expected.
  - Schema version bump required touching old milestone assertions for `index/status` output.
- Action taken:
  - failing test added: `tests/milestone6_lifecycle.rs` and `tests/milestone6_schema_migration.rs`.
  - fix commit: pending.
  - docs/plan update: updated `agents/repo-scout-agent-first-phase2-execplan.md` Progress/Decision Log/Outcomes.
- Status: `fixed`

- Date: `2026-02-06`
- Task: Milestone 7 richer Rust symbol extraction and metadata persistence.
- Commands run:
  - `just dogfood-pre Launcher`
  - `just tdd-red milestone7_struct_enum_trait_defs`
  - `just tdd-green milestone7_struct_enum_trait_defs`
  - `just tdd-refactor`
  - `just dogfood-pre run`
  - `just tdd-red milestone7_impl_method_container`
  - `just tdd-green milestone7_impl_method_container`
  - `just tdd-refactor`
  - `just dogfood-pre LocalLauncher`
  - `just tdd-red milestone7_module_alias_const_use`
  - `just tdd-green milestone7_module_alias_const_use`
  - `just dogfood-pre start_engine`
  - `just tdd-red milestone7_spans_and_signatures_persist`
  - `just tdd-green milestone7_spans_and_signatures_persist`
  - `just tdd-refactor`
  - `just dogfood-post start_engine`
- What helped:
  - Existing Tree-sitter integration allowed additive symbol-kind extraction with small parser changes.
  - `symbols_v2` table made it straightforward to persist richer metadata without changing `find` output schema.
- What failed or felt weak:
  - Import extraction caused valid duplicate AST hits for `Launcher`, so count-based assertions needed to be relaxed.
- Action taken:
  - failing test added: `tests/milestone7_rust_symbols.rs`.
  - fix commit: pending.
  - docs/plan update: updated `README.md`, `docs/cli-reference.md`, `docs/architecture.md`, and phase-2 ExecPlan living sections.
- Status: `fixed`
