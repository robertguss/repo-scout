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

- Date: `2026-02-09`
- Task: Phase 10 Milestones 51/52 Go `find` MVP (adapter wiring + deterministic JSON behavior).
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find test_command_for_target --repo . --json`
  - `cargo run -- refs test_command_for_target --repo . --json`
  - `cargo test milestone50_go_find_definitions_are_ast_backed -- --nocapture`
  - `cargo test milestone50_go_find_persists_language_metadata -- --nocapture`
  - `cargo test milestone50_go_find_json_is_deterministic -- --nocapture`
  - `cargo test milestone50_go_find_scope_flags_do_not_regress_existing_languages -- --nocapture`
  - `cargo test milestone50_go_find -- --nocapture`
  - `cargo run -- index --repo tests/fixtures/phase10/go_find`
  - `cargo run -- find SayHello --repo tests/fixtures/phase10/go_find --json`
  - `cargo run -- find Greeter --repo tests/fixtures/phase10/go_find --json`
  - `cargo test`
- What helped:
  - Keeping Go scope definition-only allowed minimal adapter integration with no schema changes.
  - Fixture-backed tests locked AST-first `find` behavior and `symbols_v2.language = "go"` data
    persistence.
- What failed or felt weak:
  - `refs` for Go symbols remains text fallback in this phase by design (Go AST references deferred).
- Action taken:
  - failing test added:
    - `tests/milestone50_go_find.rs`
  - fix commit:
    - pending (Phase 10 branch in progress)
  - docs update:
    - `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `docs/performance-baseline.md`, `agents/plans/repo-scout-phase10-execplan.md`.
- Status: `fixed`

- Date: `2026-02-09`
- Task: Phase 10 Milestones 49/50 Rust hardening (`refs` dedupe + test-target/scope classifier).
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find test_command_for_target --repo . --json`
  - `cargo run -- refs test_command_for_target --repo . --json`
  - `cargo test milestone49_refs_deduplicates_ast_rows -- --nocapture`
  - `cargo test milestone49_verify_plan_targets_remain_deterministic -- --nocapture`
  - `cargo test milestone49_scope_filtering_preserves_contract -- --nocapture`
  - `cargo test`
- What helped:
  - A minimal chained-call fixture (`helper().is_some()`) reproduced duplicate AST reference rows
    reliably for strict Red evidence.
  - Narrowing runnable-target synthesis to direct `tests/<file>.rs` removed invalid cargo commands
    for non-Rust test files.
- What failed or felt weak:
  - Test-like filtering was too Rust-centric before this slice and leaked common TS/Python test
    paths under `--exclude-tests`.
- Action taken:
  - failing test added:
    - `tests/milestone49_rust_hardening.rs`
  - fix commit:
    - pending (Phase 10 branch in progress)
  - docs update:
    - `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `agents/plans/repo-scout-phase10-execplan.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 8 Milestone 41 docs/evidence/performance refresh plus final verification.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json`
  - `cargo run -- explain diff_impact_for_changed_files --repo . --json`
  - `cargo run -- index --repo tests/fixtures/phase8/semantic_precision`
  - `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo fmt`
- What helped:
  - Updating docs directly from executed command output avoided drift in `include_tests` and
    terminal `diff-impact` behavior descriptions.
  - Re-running the semantic fixture pack confirmed Milestone 37 closure still holds after Milestone
    39/40 changes.
- What failed or felt weak:
  - Terminal `diff-impact` output for large files can be long; `--exclude-tests` is useful to trim
    noisy test-target rows during quick triage.
- Action taken:
  - failing test added:
    - no new behavior tests (documentation/evidence and final verification milestone).
  - fix commit:
    - `Implement Milestone 41 Phase 8 docs and verification refresh`
  - docs update:
    - `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `docs/performance-baseline.md`, `docs/dogfood-log.md`, `agents/repo-scout-phase8-execplan.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 8 Milestone 40 actionable deterministic `diff-impact` terminal rows.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`
  - `cargo test milestone40_diff_impact_terminal_lists_impacted_symbol_rows -- --nocapture`
  - `cargo test milestone40_diff_impact_terminal_lists_test_target_rows_conditionally -- --nocapture`
  - `cargo test milestone40_diff_impact_terminal_output_is_deterministic -- --nocapture`
  - `cargo test`
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
- What helped:
  - Reusing the existing deterministic `DiffImpactMatch` sort order made row-level rendering
    straightforward and stable.
  - Explicit `result_kind` row prefixes (`impacted_symbol`, `test_target`) made terminal output
    machine- and human-scannable without changing schema 3 JSON.
- What failed or felt weak:
  - Pre-change terminal output only reported summary counts, which hid actionable details for fast
    CLI triage.
- Action taken:
  - failing test added:
    - `tests/milestone40_diff_impact_terminal_output.rs`
  - fix commit:
    - `Implement Milestone 40 deterministic diff-impact terminal rows via TDD`
  - docs update:
    - `agents/repo-scout-phase8-execplan.md`, `docs/dogfood-log.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 8 Milestone 39 explicit `diff-impact` test-target toggle.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`
  - `cargo test milestone39_diff_impact_exclude_tests_omits_test_targets -- --nocapture`
  - `cargo test milestone39_diff_impact_default_and_include_tests_keep_test_targets -- --nocapture`
  - `cargo test milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit -- --nocapture`
  - `cargo test`
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json`
- What helped:
  - Additive CLI plumbing (`--exclude-tests` with clap conflict handling) enabled symbol-only mode
    without any schema changes.
  - JSON echo field `include_tests` made behavior explicit for automation consumers.
- What failed or felt weak:
  - Before the change, `--include-tests` produced the same output as default behavior and there was
    no way to suppress test-target rows.
- Action taken:
  - failing test added:
    - `tests/milestone39_diff_impact_test_toggle.rs`
  - fix commit:
    - `Implement Milestone 39 diff-impact test-toggle opt-out via TDD`
  - docs update:
    - `agents/repo-scout-phase8-execplan.md`, `docs/dogfood-log.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 8 Milestone 38 strict clippy gate recovery.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`
  - `cargo clippy --test harness_smoke -- -D warnings`
  - `cargo clippy --bin repo-scout -- -D warnings`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
- What helped:
  - Targeting lint failures by slice (`harness_smoke` then `bin`) kept edits tightly scoped.
  - Preserving recursive helper signatures with explicit `#[allow(clippy::too_many_arguments)]`
    prevented risky refactors while still meeting strict lint gates.
- What failed or felt weak:
  - Clippy warning classes evolved (`double_ended_iterator_last` to `filter_next`) as code changed,
    so fixes needed a second pass (`rfind`) to settle.
- Action taken:
  - failing test added:
    - no new behavior tests; milestone enforced static-analysis gates and full-suite regression
      pass.
  - fix commit:
    - `Implement Milestone 38 strict clippy quality-gate cleanup via TDD`
  - docs update:
    - `agents/repo-scout-phase8-execplan.md`, `docs/dogfood-log.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 8 Milestone 37 semantic closure for direct alias-import call paths.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`
  - `cargo test milestone37_typescript_namespace_alias_diff_impact_recalls_caller -- --nocapture`
  - `cargo test milestone37_python_module_alias_diff_impact_recalls_caller -- --nocapture`
  - `cargo test milestone37_semantic_precision_deterministic_ordering -- --nocapture`
  - `cargo test`
  - `cargo run -- index --repo tests/fixtures/phase8/semantic_precision`
  - `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json`
- What helped:
  - New alias-call hint maps in TypeScript/Python adapters resolved direct alias-import calls to
    qualified duplicate-name callees (`helperA()` and `helper_a()`).
  - Keeping both direct and local-import call edges preserved existing `impact <import_alias>`
    behavior while fixing `diff-impact` distance-1 caller recall.
- What failed or felt weak:
  - TypeScript fixture still includes an expected `imported_by` row for the local import symbol
    (`helperA`) in addition to direct caller rows, which is correct but can add output noise.
- Action taken:
  - failing test added:
    - `tests/milestone37_semantic_precision.rs`
  - fix commit:
    - `Implement Milestone 37 semantic alias-call closure via TDD`
  - docs update:
    - `agents/repo-scout-phase8-execplan.md`, `docs/dogfood-log.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 7 Milestones 32-36 cross-language semantic precision and quality benchmark guardrails.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find resolve_symbol_id_in_tx --repo . --json`
  - `cargo run -- refs resolve_symbol_id_in_tx --repo . --json`
  - `cargo test milestone32_typescript_namespace_alias_call_contract -- --nocapture`
  - `cargo test milestone32_python_module_alias_call_contract -- --nocapture`
  - `cargo test milestone32_schema_contracts_stay_stable -- --nocapture`
  - `cargo test milestone33_typescript_namespace_alias_resolves_changed_callee -- --nocapture`
  - `cargo test milestone33_typescript_member_call_prefers_import_context -- --nocapture`
  - `cargo test milestone33_typescript_semantics_preserve_existing_m15_behavior -- --nocapture`
  - `cargo test milestone34_python_module_alias_resolves_changed_callee -- --nocapture`
  - `cargo test milestone34_python_attribute_call_prefers_import_context -- --nocapture`
  - `cargo test milestone34_python_semantics_preserve_existing_m16_behavior -- --nocapture`
  - `cargo test milestone35_diff_impact_semantic_confidence_ranking -- --nocapture`
  - `cargo test milestone35_impact_semantic_rows_rank_deterministically -- --nocapture`
  - `cargo test milestone35_fixture_quality_benchmark_is_stable -- --nocapture`
  - `cargo run -- index --repo tests/fixtures/phase7/semantic_precision`
  - `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json`
  - `cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase7/semantic_precision --json`
  - `cargo run -- impact helper --repo tests/fixtures/phase7/semantic_precision --json`
  - `cargo run -- diff-impact --changed-file src/indexer/languages/typescript.rs --repo . --json`
  - `cargo run -- diff-impact --changed-file src/indexer/languages/python.rs --repo . --json`
  - `cargo run -- refs helper --repo . --code-only --exclude-tests --max-results 10 --json`
  - `cargo test`
  - `cargo fmt`
- Transcript artifact (from
  `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json`):

```json
{
  "changed_files": ["src/util_a.ts"],
  "results": [
    {
      "symbol": "helper",
      "file_path": "src/util_a.ts",
      "relationship": "changed_symbol",
      "distance": 0
    },
    {
      "symbol": "run",
      "file_path": "src/app.ts",
      "relationship": "called_by",
      "distance": 1,
      "provenance": "call_resolution",
      "score": 0.97
    }
  ]
}
```

- What helped:
  - Module-aware alias hints in TypeScript/Python adapters eliminated duplicate-name callee
    ambiguity for namespace/member and module-alias attribute calls.
  - Query-time semantic calibration made `impact`/`diff-impact` ranking deterministic with
    high-confidence caller rows (`score: 0.97`) above fallback rows.
  - Shared Phase 7 fixture corpus made behavior-check packs reproducible across milestones.
- What failed or felt weak:
  - Fixture index state can go stale across extractor-code changes when file hashes are unchanged;
    verification requires at least one fixture-content refresh or fresh fixture copy.
- Action taken:
  - failing test added:
    - `tests/milestone32_semantic_contracts.rs`
    - `tests/milestone33_typescript_semantics.rs`
    - `tests/milestone34_python_semantics.rs`
    - `tests/milestone35_quality_benchmark.rs`
  - fix commit:
    - `Add Milestone 32 semantic contract tests via TDD`
    - `Implement Milestone 33 TypeScript semantic resolution via TDD`
    - `Implement Milestone 34 Python semantic resolution via TDD`
    - `Implement Milestone 35 semantic ranking calibration via TDD`
  - docs update:
    - `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `docs/dogfood-log.md`, `docs/performance-baseline.md`, `agents/repo-scout-phase7-execplan.md`.
- Status: `fixed`

- Date: `2026-02-08`
- Task: Phase 6 documentation/coverage audit against current implementation.
- Commands run:
  - `cargo run -- index --repo .`
  - `cargo run -- find explain_symbol --repo . --json`
  - `cargo run -- refs explain_symbol --repo . --json`
  - `rustup run stable cargo llvm-cov --workspace --all-targets --summary-only`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json`
- What helped:
  - Coverage output gave concrete line/function/region percentages to validate ">=85%" expectations.
  - Direct command checks exposed docs drift for `diff-impact` test-target wording and schema enum
    notes.
- What failed or felt weak:
  - `--include-tests` is currently a compatibility flag; behavior remains default-on in practice.
- Action taken:
  - failing test added:
    - unit tests in `src/store/mod.rs` for store bootstrap/corruption hint helpers.
    - unit tests in `src/main.rs` for changed-line parsing/normalization edge cases.
  - fix commit:
    - not yet committed (working tree update).
  - docs update:
    - `README.md`, `docs/architecture.md`, `docs/cli-reference.md`, `docs/json-output.md`,
      `docs/performance-baseline.md`.
- Status: `fixed`

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
  - `refs helper --max-results 10` remains test-heavy in this repository because exact fallback hits
    for that token are concentrated under `tests/`.
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
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`, `agents/repo-scout-phase5-execplan.md`.
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
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`, `agents/repo-scout-phase5-execplan.md`.
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
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`.
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
  - docs update: `README.md`, `docs/cli-reference.md`, `docs/json-output.md`,
    `docs/architecture.md`.
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
  - fix commit: SymbolKey-aware resolver plus adapter disambiguation hints; prioritize repo-scout in
    test harness.
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
  - failing test added:
    `tests/milestone12_diff_impact.rs::milestone12_diff_impact_changed_files_normalization`
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
  - Python adapter achieved `find`/`refs`/`impact`/`diff-impact`/`explain` coverage without
    query-layer branching.
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
  - failing test added:
    `tests/milestone10_validation.rs::milestone10_verify_plan_skips_non_runnable_test_modules`
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
