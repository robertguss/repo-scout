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

- Date: `2026-02-08`
- Task: Phase 6 Milestones 27-30 change-scope precision and output-focus controls.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find verify_plan_for_changed_files --repo . --json`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --json`
  - `cargo test milestone27_context_exclude_tests_omits_test_paths -- --nocapture`
  - `cargo test milestone27_context_code_only_restricts_to_code_extensions -- --nocapture`
  - `cargo test milestone27_context_scope_flags_preserve_deterministic_json -- --nocapture`
  - `cargo test milestone28_verify_plan_changed_line_limits_targeted_symbol_set -- --nocapture`
  - `cargo test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations -- --nocapture`
  - `cargo test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate -- --nocapture`
  - `cargo test milestone29_diff_impact_changed_symbol_filters_seed_rows -- --nocapture`
  - `cargo test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows -- --nocapture`
  - `cargo test milestone29_diff_impact_max_results_caps_deterministically -- --nocapture`
  - `cargo test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests -- --nocapture`
  - `cargo test milestone30_find_and_refs_max_results_cap_deterministically -- --nocapture`
  - `cargo test milestone30_query_caps_compose_with_code_only_and_exclude_tests -- --nocapture`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json`
  - `cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json`
  - `cargo run -- refs helper --repo . --max-results 10 --json`
  - `cargo test`
- What helped:
  - Additive scope filters (`context`, `verify-plan`, `diff-impact`) cut broad payload noise in
    large files without schema changes.
  - `find`/`refs --max-results` plus fallback code-first tie-breaks improved deterministic scan
    quality in fallback-heavy queries.
  - `diff-impact --exclude-changed --max-results` produced focused impacted/test outputs while
    preserving traversal semantics.
- What failed or felt weak:
  - `refs helper --max-results 10` remains test-heavy in this repository because exact fallback
    hits for that token are concentrated under `tests/`.
  - Slice 30C regression guard was already satisfied on first run; no new production change was
    needed for cap/scope composition once max-result plumbing landed.
- Action taken:
  - failing test added:
    - `tests/milestone27_context_scope.rs`
    - `tests/milestone28_verify_plan_scope.rs`
    - `tests/milestone29_diff_impact_scope.rs`
    - `tests/milestone30_query_focus.rs`
  - fix commit:
    - `Implement Milestone 27 context scope controls via TDD`
    - `Implement Milestone 28 verify-plan scope controls via TDD`
    - `Implement Milestone 29 diff-impact scope controls via TDD`
    - `Implement Milestone 30 query fallback focus and deterministic caps via TDD`
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`, `docs/dogfood-log.md`, `docs/performance-baseline.md`,
    `agents/repo-scout-phase6-execplan.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Phase 5 Milestones 22-24 recommendation quality (`tests-for`, `verify-plan`, `context`).
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find verify_plan_for_changed_files --repo . --json`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --json`
  - `cargo test milestone22_tests_for_excludes_support_paths_by_default -- --nocapture`
  - `cargo test milestone22_tests_for_prefers_runnable_targets -- --nocapture`
  - `cargo test milestone22_tests_for_include_support_restores_paths -- --nocapture`
  - `cargo test milestone23_verify_plan_downranks_generic_changed_symbols -- --nocapture`
  - `cargo test milestone23_verify_plan_applies_targeted_cap_deterministically -- --nocapture`
  - `cargo test milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate -- --nocapture`
  - `cargo test milestone24_context_matches_relevant_symbols_for_paraphrased_task -- --nocapture`
  - `cargo test milestone24_context_prioritizes_definitions_over_incidental_tokens -- --nocapture`
  - `cargo test milestone24_context_json_is_stable_with_relevance_scoring -- --nocapture`
  - `cargo run -- tests-for Path --repo . --json`
  - `cargo run -- tests-for Path --repo . --include-support --json`
  - `cargo run -- verify-plan --changed-file src/main.rs --repo . --json`
  - `cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json`
- What helped:
  - Runnable-first target classification plus explicit `--include-support` eliminated noisy default
    support-path recommendations.
  - Generic-symbol damping and deterministic capping reduced verify-plan step explosion.
  - Token-overlap relevance improved context recall for paraphrased task text without schema churn.
- What failed or felt weak:
  - Initial strict TDD sequencing briefly failed when future-slice tests were introduced too early.
- Action taken:
  - failing test added:
    - `tests/milestone22_recommendation_quality.rs`
    - `tests/milestone23_verify_plan_precision.rs`
    - `tests/milestone24_context_relevance.rs`
  - fix commit:
    - `Implement Milestone 22 tests-for recommendation quality via TDD`
    - `Implement Milestone 23 verify-plan precision controls via TDD`
    - `Implement Milestone 24 context relevance scoring via TDD`
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
    `agents/repo-scout-phase5-execplan.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Phase 5 Milestone 25 true multi-hop `diff-impact` traversal fidelity.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find verify_plan_for_changed_files --repo . --json`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --json`
  - `cargo test milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors -- --nocapture`
  - `cargo test milestone25_diff_impact_respects_max_distance_bound -- --nocapture`
  - `cargo test milestone25_diff_impact_handles_cycles_without_duplicate_growth -- --nocapture`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json`
  - `cargo test`
- What helped:
  - Inbound BFS frontier traversal over `symbol_edges_v2` made `--max-distance` semantics true for
    distance 2+ without changing schema envelopes.
- What failed or felt weak:
  - Cycle traversal initially re-emitted changed symbols at non-zero distance.
- Action taken:
  - failing test added: `tests/milestone25_diff_impact_multihop.rs`
  - fix commit: `Implement Milestone 25 multi-hop diff-impact traversal via TDD`
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
    `agents/repo-scout-phase5-execplan.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Phase 5 Milestone 26 docs and transcript refresh.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- tests-for Path --repo . --json`
  - `cargo run -- tests-for Path --repo . --include-support --json`
  - `cargo run -- verify-plan --changed-file src/main.rs --repo . --json`
  - `cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json`
  - `cargo test`
- What helped:
  - Re-running the exact dogfood set after docs edits confirmed command/output descriptions still
    match real behavior.
- What failed or felt weak:
  - Large `diff-impact --json` payloads are hard to scan manually without focused filtering.
- Action taken:
  - failing test added: none (documentation/evidence pass).
  - fix commit: docs and plan refresh commit for Milestone 26.
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`, `docs/dogfood-log.md`, `docs/performance-baseline.md`,
    `agents/repo-scout-phase5-execplan.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 21 scoped fallback controls for `find`/`refs`.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find verify_plan_for_changed_files --repo . --json`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --json`
  - `cargo test milestone21_refs_code_only_omits_docs_text_fallback -- --nocapture`
  - `cargo test milestone21_refs_exclude_tests_omits_test_paths -- --nocapture`
  - `cargo test milestone21_find_scope_flags_keep_ast_priority_and_determinism -- --nocapture`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --code-only --exclude-tests`
- What helped:
  - Applying scope flags only to text fallback kept AST-priority behavior stable and deterministic.
- What failed or felt weak:
  - Initial CLI surface rejected `--code-only`/`--exclude-tests`.
- Action taken:
  - failing test added: `tests/milestone21_query_scope.rs`
  - fix commit: add command-specific args and fallback scope filtering in query module.
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestone 20 `diff-impact` import/line-range precision controls.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo test milestone20_diff_impact_excludes_import_seeds_by_default -- --nocapture`
  - `cargo test milestone20_diff_impact_include_imports_restores_import_rows -- --nocapture`
  - `cargo test milestone20_diff_impact_changed_line_limits_seed_symbols -- --nocapture`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --changed-line src/query/mod.rs:132:220 --repo .`
- What helped:
  - Existing symbol span metadata in `symbols_v2` made overlap filtering straightforward.
- What failed or felt weak:
  - CLI lacked `--include-imports` and `--changed-line` support prior to implementation.
- Action taken:
  - failing test added: `tests/milestone20_diff_impact_precision.rs`
  - fix commit: add diff-impact option parsing plus changed-symbol seed filtering by kind/range.
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`.
- Status: `fixed`

- Date: `2026-02-07`
- Task: Milestones 18-19 precision contract lock and resolver hardening.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find run --repo . --json`
  - `cargo run -- refs run --repo . --json`
  - `cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture`
  - `cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture`
  - `cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture`
- What helped:
  - Scoped/qualified symbol keys made duplicate-name call resolution deterministic.
- What failed or felt weak:
  - Test harness initially preferred external `codex-5-3` binary over local `repo-scout`.
- Action taken:
  - failing test added: `tests/milestone18_precision_graph.rs`
  - fix commit: SymbolKey-aware resolver plus adapter disambiguation hints; prioritize repo-scout in test harness.
  - docs update: architecture precision notes and Phase 4 plan artifacts.
- Status: `fixed`

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
