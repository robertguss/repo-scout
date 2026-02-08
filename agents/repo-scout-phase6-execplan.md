# Build `repo-scout` Phase 6 Change-Scope Precision and Output Focus Controls

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance
with that file.

This plan builds on `agents/repo-scout-phase5-execplan.md`, which delivered recommendation-quality
improvements (`tests-for`, `verify-plan`, `context`) and true bounded multi-hop traversal for
`diff-impact`.

## Purpose / Big Picture

Phase 6 focuses on change-scoped precision and output focus for real editing loops. After this
change, users and agents should be able to narrow `context`, `verify-plan`, and `diff-impact`
outputs to the exact touched regions/symbols of a large file, and they should be able to cap and
prioritize noisy fallback-heavy `find`/`refs` results deterministically.

User-visible outcome: fewer broad payloads when working in large files, less fallback clutter from
non-code paths, and faster conversion from “what changed” to “what do I run or inspect next”.

## Progress

- [x] (2026-02-07 23:32Z) Re-read `agents/PLANS.md`, `agents/repo-scout-phase5-execplan.md`,
      `README.md`, `docs/architecture.md`, `docs/cli-reference.md`, and `docs/json-output.md`
      to align Phase 6 scope with current contracts.
- [x] (2026-02-07 23:32Z) Ran Phase 6 planning baseline dogfood commands:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 23:32Z) Captured baseline noise/focus evidence for fallback-heavy `refs`,
      context relevance distribution, and `diff-impact` changed-symbol flooding.
- [x] (2026-02-07 23:32Z) Authored this Phase 6 ExecPlan as planning-only work.
- [x] (2026-02-07 23:38Z) Ran required pre-milestone dogfood baseline for Milestone 27.
- [x] (2026-02-07 23:55Z) Completed Milestone 27 strict TDD slices for context scope controls
      (`--exclude-tests`, `--code-only`, deterministic combined scope behavior).
- [x] (2026-02-07 23:58Z) Ran Milestone 27 post-dogfood checks; `verify-plan --changed-line`,
      `verify-plan --changed-symbol`, `diff-impact --changed-symbol`, `diff-impact --exclude-changed`,
      `diff-impact --max-results`, and `refs --max-results` correctly fail as unsupported prior to
      Milestones 28–30.
- [x] (2026-02-07 23:59Z) Ran required pre-milestone dogfood baseline for Milestone 28.
- [x] (2026-02-08 00:03Z) Completed Milestone 28 strict TDD slices for verify-plan change-scope
      controls (`--changed-line`, repeatable `--changed-symbol`, scope safety preservation).
- [x] (2026-02-08 00:04Z) Ran Milestone 28 post-dogfood checks; verify-plan scoped dogfood command
      now succeeds while future Milestone 29/30 flags remain expected clap failures.
- [x] (2026-02-08 00:05Z) Ran required pre-milestone dogfood baseline for Milestone 29.
- [x] (2026-02-08 00:08Z) Completed Milestone 29 strict TDD slices for diff-impact focused output
      controls (`--changed-symbol`, `--exclude-changed`, deterministic `--max-results` cap).
- [x] (2026-02-08 00:09Z) Ran Milestone 29 post-dogfood checks; diff-impact focused command now
      succeeds while future Milestone 30 `refs --max-results` still fails as expected.
- [x] (2026-02-08 00:10Z) Ran required pre-milestone dogfood baseline for Milestone 30.
- [x] (2026-02-08 00:13Z) Completed Milestone 30 strict TDD slices for fallback relevance/limit
      controls in find/refs (`code-first` fallback path-class tie-breaks, `--max-results`, and
      cap/scope composition with AST-priority preserved).
- [x] (2026-02-08 00:13Z) Ran Milestone 30 post-dogfood checks; verify-plan + diff-impact scoped
      commands and `refs --max-results` all succeed with full test suite green.
- [x] (2026-02-08 00:14Z) Ran required pre-milestone dogfood baseline for Milestone 31.
- [x] (2026-02-08 00:15Z) Updated docs and evidence artifacts for Phase 6:
      `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`,
      `docs/dogfood-log.md`, `docs/performance-baseline.md`.
- [x] (2026-02-08 00:16Z) Re-ran required post-milestone dogfood checks after docs refresh,
      executed `cargo fmt`, and passed final `cargo test`.

## Surprises & Discoveries

- Observation: fallback-heavy `refs` queries still return broad test/docs payloads by default when
  AST matches are unavailable.
  Evidence: `cargo run --quiet -- refs helper --repo . --json | jq ...` reported `total: 80`,
  `tests: 65`, `docs: 13`.

- Observation: context retrieval quality improved in Phase 5 but still skews toward test files for
  verification-heavy task wording.
  Evidence: `cargo run --quiet -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json | jq ...`
  reported `total: 6`, `tests: 4`.

- Observation: `diff-impact` can still produce mostly distance-0 changed-symbol rows for large
  changed files when no seed narrowing is provided.
  Evidence: `cargo run --quiet -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json | jq ...`
  reported `total: 57`, `d0: 56`, `d1: 0`, `d2: 0`, `d3: 0`.

- Observation: post-milestone dogfood command packs include future-phase flags that are expected to
  fail before their milestone is implemented.
  Evidence: Milestone 27 post-checks reported clap errors for unsupported flags:
  `verify-plan --changed-line`, `verify-plan --changed-symbol`, `diff-impact --changed-symbol`,
  `diff-impact --exclude-changed`, `diff-impact --max-results`, and `refs --max-results`.

- Observation: context dedupe previously collapsed same-path/same-line/same-symbol rows across
  different symbol kinds, reducing determinism under tied scores.
  Evidence: red transcript for
  `milestone27_context_scope_flags_preserve_deterministic_json` only returned `kind=function`
  until dedupe/sort keys were made kind-aware.

- Observation: `verify-plan` scoped safety behavior from Phase 5 remained intact under new
  line/symbol scope controls (changed test target + full-suite gate survived restrictive filters).
  Evidence: `milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate`
  passed immediately on first run and again on the green re-run after scope feature work landed.

- Observation: once Milestone 29 landed, the previously failing dogfood command
  `diff-impact --changed-symbol ... --exclude-changed --max-results ...` became fully actionable
  and produced focused neighbor/test payloads without distance-0 seed rows.
  Evidence: Milestone 29 post-check produced schema 3 JSON with test targets only for
  `verify_plan_for_changed_files` when `--exclude-changed` was set.

- Observation: Milestone 30 cap/scope composition behavior (`--code-only --exclude-tests
  --max-results`) was already satisfied on first run once `--max-results` landed for find/refs.
  Evidence: `milestone30_query_caps_compose_with_code_only_and_exclude_tests` passed on its first
  execution and again on the required re-run.

- Observation: `refs helper --max-results 10` now succeeds deterministically but still skews
  test-heavy for this repository token because non-test code exact matches are sparse.
  Evidence: Milestone 30 post-dogfood output returned schema 1 JSON with 10 deterministic rows,
  all from `tests/...` exact fallback matches.

- Observation: the post-refresh Milestone 31 dogfood rerun produced the same scoped command
  behavior and schema envelopes as Milestone 30, confirming documentation-only edits did not drift
  command semantics.
  Evidence: final command pack (`context`, `verify-plan`, `diff-impact`, `refs --max-results`,
  `cargo test`) completed successfully with unchanged schema versions (1/2/3).

## Decision Log

- Decision: prioritize option-driven narrowing controls over adding new command families.
  Rationale: current command surface is broad enough; the gap is precision and scanability under
  realistic changed-file and fallback-heavy conditions.
  Date/Author: 2026-02-07 / Codex

- Decision: keep schema 1/2/3 envelopes stable through Phase 6 and implement behavior changes via
  deterministic filtering/ranking/capping options.
  Rationale: existing automation depends on current schemas; this phase addresses relevance without
  forcing contract migration.
  Date/Author: 2026-02-07 / Codex

- Decision: align new changed-scope controls between `verify-plan` and `diff-impact` using shared
  changed-line parsing and additive changed-symbol filters.
  Rationale: users already think in changed file + line/symbol scope; consistent semantics reduce
  command friction and learning cost.
  Date/Author: 2026-02-07 / Codex

- Decision: preserve Phase 5 safety guarantees (`cargo test` gate and changed runnable test target
  preservation) regardless of new narrowing options.
  Rationale: precision controls must not remove core safety behavior from validation workflows.
  Date/Author: 2026-02-07 / Codex

- Decision: run the post-milestone dogfood command set exactly as written at every milestone, and
  treat unsupported-flag failures as expected until each owning milestone lands.
  Rationale: this preserves consistent dogfood evidence while showing feature activation progress
  from milestone to milestone.
  Date/Author: 2026-02-07 / Codex

- Decision: make context dedupe/sort kind-aware (`file_path/start_line/symbol/kind`) so combined
  scope flags preserve deterministic output when equal-score symbol rows differ only by kind.
  Rationale: without kind-aware keys/tie-breaks, valid rows can be dropped or left with unstable
  ordering in same-symbol/same-location scenarios.
  Date/Author: 2026-02-07 / Codex

- Decision: keep the Phase 5 changed-test/full-suite safety semantics unchanged while adding
  `verify-plan --changed-line` and `--changed-symbol` filters, and treat slice 28C as a strict
  regression guard.
  Rationale: scope narrowing must not regress safety guarantees; preserving existing behavior is
  the intended outcome for this slice.
  Date/Author: 2026-02-08 / Codex

- Decision: apply `diff-impact --exclude-changed` as an output-stage filter after seed collection
  and traversal, and apply `--max-results` as post-sort truncation.
  Rationale: this preserves traversal correctness and deterministic ordering semantics while making
  focused output controls additive and easy to reason about.
  Date/Author: 2026-02-08 / Codex

- Decision: apply `find`/`refs --max-results` as a handler-stage truncation after query-layer
  scope filtering and AST/fallback selection.
  Rationale: this keeps AST-priority behavior unchanged while ensuring deterministic caps for both
  AST and fallback result sets.
  Date/Author: 2026-02-08 / Codex

- Decision: implement code-first fallback tie-breaks only inside shared text fallback ranking and
  leave AST ordering untouched.
  Rationale: Phase 6 requires fallback focus improvements without changing established AST-first
  semantics or schema 1 output contracts.
  Date/Author: 2026-02-08 / Codex

## Outcomes & Retrospective

Planning outcome: Phase 6 scope is constrained to high-impact precision controls for existing
commands, with no schema version churn and no new command families.

Expected completion outcome: large-file workflows become actionable with explicit changed-line and
changed-symbol targeting, `diff-impact` output can be focused on true impacted neighbors, and
fallback-heavy `find`/`refs` queries can be capped and ranked toward code-first relevance.

Expected residual work after this plan: deeper type-aware semantics for cross-language call/import
resolution and broader benchmark corpora for recommendation-quality scoring.

Milestone 27 outcome (2026-02-07): `context` now supports `--exclude-tests` and `--code-only`,
and combined scoped JSON output is deterministic with kind-aware dedupe/sort behavior. Full-suite
tests stayed green through each slice refactor gate.

Milestone 28 outcome (2026-02-08): `verify-plan` now supports additive `--changed-line` and
repeatable `--changed-symbol` scope controls with deterministic normalization, while preserving
changed runnable test targets and the mandatory `cargo test` gate.

Milestone 29 outcome (2026-02-08): `diff-impact` now supports repeatable `--changed-symbol`,
`--exclude-changed`, and deterministic `--max-results` truncation while keeping schema 3 envelope
shape and traversal semantics stable.

Milestone 30 outcome (2026-02-08): fallback-heavy `find`/`refs` now rank code paths ahead of
test/docs paths at equal fallback score tiers, both commands support deterministic
`--max-results`, and cap behavior composes with existing scope flags without changing AST-priority.

Milestone 31 outcome (2026-02-08): all user-facing docs and evidence artifacts now reflect Phase 6
controls (`context` scope flags, `verify-plan` change-scope filters, `diff-impact` focused output
controls, and `find`/`refs` deterministic caps), with post-refresh dogfood and full-suite tests
confirming contract alignment.

## Context and Orientation

`repo-scout` command parsing is in `src/cli.rs`, command dispatch and changed-line parsing are in
`src/main.rs`, query logic is in `src/query/mod.rs`, and output envelopes are in `src/output.rs`.
Milestone-oriented integration tests live under `tests/`.

Terms used in this plan:

- A “changed-symbol filter” means explicit user-provided symbol names that constrain seed symbols
  selected from changed files.
- A “distance-0 row” in `diff-impact` means `relationship = changed_symbol`; these are seed rows,
  not traversed inbound neighbors.
- A “fallback row” means text-only `find`/`refs` results (`confidence = text_fallback`) returned
  when AST hits are absent.
- A “path class” means deterministic grouping of paths for ranking (`code`, `test-like`,
  `docs/other`).

Current hot spots:

- `src/query/mod.rs::context_matches` has no scope filters analogous to `find`/`refs`.
- `src/query/mod.rs::verify_plan_for_changed_files` supports `--max-targeted` but cannot narrow by
  changed line range or explicit changed symbols.
- `src/query/mod.rs::diff_impact_for_changed_files` supports changed-line filtering but cannot
  explicitly filter seeds by symbol name or omit distance-0 rows from final output.
- `src/query/mod.rs::ranked_text_matches`/`text_exact_matches`/`text_substring_matches` do not yet
  support deterministic result caps or path-class tie-break ranking improvements.
- `src/cli.rs` currently lacks Phase 6 flags for context, verify-plan, diff-impact, and find/refs
  result caps.

## Strict TDD Contract

Phase 6 enforces strict per-slice red-green-refactor. No production code is allowed for a feature
slice before a failing test exists for that exact slice.

A feature slice in this plan is one user-visible behavior, such as “`verify-plan --changed-line`
limits symbol-derived targeted recommendations” or “`diff-impact --exclude-changed` suppresses
`distance = 0` rows while traversal still occurs from those seeds.”

For every slice, record:

- red transcript: failing integration test command,
- green transcript: the same test command passing after minimal implementation,
- refactor transcript: full-suite `cargo test` pass.

Record evidence in this plan and in `docs/dogfood-log.md`.

## Plan of Work

### Milestone 27: Context scope controls for focused bundles

Milestone goal: make `context` narrowing symmetric with other query surfaces so users can opt out
of test/docs noise when gathering editing context.

Feature slice 27A adds `context --exclude-tests`. Add test
`tests/milestone27_context_scope.rs::milestone27_context_exclude_tests_omits_test_paths` asserting
no test-like paths are returned when the flag is set.

Feature slice 27B adds `context --code-only`. Add test
`tests/milestone27_context_scope.rs::milestone27_context_code_only_restricts_to_code_extensions`
asserting results are limited to `.rs`, `.ts`, `.tsx`, `.py` paths.

Feature slice 27C locks deterministic composition of both scope flags. Add test
`tests/milestone27_context_scope.rs::milestone27_context_scope_flags_preserve_deterministic_json`
that repeats identical scoped queries and asserts byte-identical JSON output.

Implementation orientation: update `src/cli.rs::ContextArgs`, `src/main.rs::run_context`, and
`src/query/mod.rs::context_matches` with additive scope options, reusing existing path classifiers
(`is_code_file_path`, `is_test_like_path`) to avoid duplicated policy logic.

### Milestone 28: Verify-plan change-scope targeting

Milestone goal: narrow targeted verification recommendations to actual touched regions/symbols in
large changed files while preserving Phase 5 safety guarantees.

Feature slice 28A adds `verify-plan --changed-line <path:start[:end]>` semantics aligned to
`diff-impact`. Add test
`tests/milestone28_verify_plan_scope.rs::milestone28_verify_plan_changed_line_limits_targeted_symbol_set`
asserting symbol-derived targeted steps are generated only from changed symbols overlapping supplied
line ranges.

Feature slice 28B adds `verify-plan --changed-symbol <symbol>` (repeatable) filter semantics. Add
test
`tests/milestone28_verify_plan_scope.rs::milestone28_verify_plan_changed_symbol_filters_targeted_recommendations`
asserting only selected changed symbols contribute targeted recommendations.

Feature slice 28C preserves critical safety behavior under combined scope narrowing. Add test
`tests/milestone28_verify_plan_scope.rs::milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate`
asserting changed runnable test targets and final `cargo test` are retained even with restrictive
line/symbol filters.

Implementation orientation: extend `src/cli.rs::VerifyPlanArgs`, parse and normalize changed-line
specs in `src/main.rs` using existing parser logic, and evolve
`src/query/mod.rs::verify_plan_for_changed_files` to accept additive scope options without changing
schema 2 output shape.

### Milestone 29: Diff-impact focused output controls

Milestone goal: reduce `diff-impact` scan burden by letting users focus on selected seeds and hide
seed rows when they only want impacted neighbors.

Feature slice 29A adds `diff-impact --changed-symbol <symbol>` (repeatable). Add test
`tests/milestone29_diff_impact_scope.rs::milestone29_diff_impact_changed_symbol_filters_seed_rows`
asserting distance-0 rows and traversal roots are limited to selected changed symbols.

Feature slice 29B adds `diff-impact --exclude-changed`. Add test
`tests/milestone29_diff_impact_scope.rs::milestone29_diff_impact_exclude_changed_hides_distance_zero_rows`
asserting no `relationship = changed_symbol` rows are emitted while inbound neighbors from those
seeds still appear when present.

Feature slice 29C adds `diff-impact --max-results <N>` deterministic truncation. Add test
`tests/milestone29_diff_impact_scope.rs::milestone29_diff_impact_max_results_caps_deterministically`
asserting stable sorted truncation boundaries and repeatable JSON output.

Implementation orientation: extend `src/cli.rs::DiffImpactArgs`, `src/main.rs::run_diff_impact`,
and `src/query/mod.rs::diff_impact_for_changed_files` with additive filters and cap controls while
preserving schema 3 payload shape and deterministic ordering semantics.

### Milestone 30: Find/refs fallback relevance and result caps

Milestone goal: make fallback-heavy `find`/`refs` outputs more actionable by prioritizing code
paths and allowing deterministic truncation.

Feature slice 30A adds path-class tie-break ranking for fallback rows. Add test
`tests/milestone30_query_focus.rs::milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests`
asserting code-path fallback hits rank ahead of docs/support paths at equal score tiers.

Feature slice 30B adds `find --max-results <N>` and `refs --max-results <N>`. Add test
`tests/milestone30_query_focus.rs::milestone30_find_and_refs_max_results_cap_deterministically`
asserting deterministic truncation and stable ordering across repeated runs.

Feature slice 30C locks compatibility with existing scope flags. Add test
`tests/milestone30_query_focus.rs::milestone30_query_caps_compose_with_code_only_and_exclude_tests`
asserting cap behavior is applied after scope filtering and AST-priority behavior remains unchanged.

Implementation orientation: extend `src/cli.rs::{FindArgs, RefsArgs}`, `src/main.rs::{run_find,
run_refs}`, and fallback ranking in `src/query/mod.rs` so AST-first behavior remains intact and
schema 1 envelopes stay stable.

### Milestone 31: Documentation, dogfood evidence, and baseline refresh

Milestone goal: align command docs and recorded evidence with Phase 6 behavior and new controls.

Update:

- `README.md`
- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`
- `docs/dogfood-log.md`
- `docs/performance-baseline.md`

Document new context, verify-plan, diff-impact, and find/refs options with deterministic behavior
notes and examples showing recommended large-file workflows.

## Concrete Steps

Run all commands from repository root:

    cd /Users/robertguss/Projects/experiments/repo-scout

Pre-milestone dogfood baseline (before each milestone):

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json

Milestone 27 strict TDD loop:

    cargo test milestone27_context_exclude_tests_omits_test_paths -- --nocapture
    cargo test milestone27_context_exclude_tests_omits_test_paths -- --nocapture
    cargo test

    cargo test milestone27_context_code_only_restricts_to_code_extensions -- --nocapture
    cargo test milestone27_context_code_only_restricts_to_code_extensions -- --nocapture
    cargo test

    cargo test milestone27_context_scope_flags_preserve_deterministic_json -- --nocapture
    cargo test milestone27_context_scope_flags_preserve_deterministic_json -- --nocapture
    cargo test

Milestone 28 strict TDD loop:

    cargo test milestone28_verify_plan_changed_line_limits_targeted_symbol_set -- --nocapture
    cargo test milestone28_verify_plan_changed_line_limits_targeted_symbol_set -- --nocapture
    cargo test

    cargo test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations -- --nocapture
    cargo test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations -- --nocapture
    cargo test

    cargo test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate -- --nocapture
    cargo test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate -- --nocapture
    cargo test

Milestone 29 strict TDD loop:

    cargo test milestone29_diff_impact_changed_symbol_filters_seed_rows -- --nocapture
    cargo test milestone29_diff_impact_changed_symbol_filters_seed_rows -- --nocapture
    cargo test

    cargo test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows -- --nocapture
    cargo test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows -- --nocapture
    cargo test

    cargo test milestone29_diff_impact_max_results_caps_deterministically -- --nocapture
    cargo test milestone29_diff_impact_max_results_caps_deterministically -- --nocapture
    cargo test

Milestone 30 strict TDD loop:

    cargo test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests -- --nocapture
    cargo test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests -- --nocapture
    cargo test

    cargo test milestone30_find_and_refs_max_results_cap_deterministically -- --nocapture
    cargo test milestone30_find_and_refs_max_results_cap_deterministically -- --nocapture
    cargo test

    cargo test milestone30_query_caps_compose_with_code_only_and_exclude_tests -- --nocapture
    cargo test milestone30_query_caps_compose_with_code_only_and_exclude_tests -- --nocapture
    cargo test

Post-milestone dogfood checks:

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo test

Before finalizing:

    cargo fmt
    cargo test

## Validation and Acceptance

Acceptance is behavior-first and repository-observable.

For `context`:

- `--exclude-tests` and `--code-only` reduce bundle noise deterministically,
- combined scope flags remain deterministic in JSON output.

For `verify-plan`:

- line-range and changed-symbol filters narrow symbol-derived targeted steps,
- changed runnable test targets and `cargo test` full-suite gate are always preserved.

For `diff-impact`:

- changed-symbol filters narrow seed selection deterministically,
- `--exclude-changed` suppresses distance-0 rows without breaking inbound traversal,
- `--max-results` caps output deterministically after ranking.

For `find`/`refs`:

- fallback ranking prioritizes code paths over docs/support at equal score tiers,
- `--max-results` truncates deterministically and composes with scope flags,
- AST-priority behavior remains unchanged.

Final acceptance requires strict red/green/refactor evidence for every slice, full-suite pass, and
updated docs/dogfood logs.

## Idempotence and Recovery

Indexing/idempotence guarantees remain unchanged in Phase 6.

All new controls are additive command options. Omitting them must preserve existing default
behavior unless explicitly covered by a Phase 6 acceptance change. Recovery from malformed input
must remain actionable and deterministic, matching current error style (for example malformed
`--changed-line` tokens include expected format guidance).

Result caps and filters must be pure post-query transformations with deterministic ordering so
repeated runs on the same index and inputs remain byte-stable in JSON mode.

## Artifacts and Notes

Baseline evidence captured before implementation:

    cargo run --quiet -- refs helper --repo . --json | jq \
      '{total:(.results|length), tests:(.results|map(select(.file_path|test("(^tests/|/tests/|_test\\.rs$)")))|length), docs:(.results|map(select(.file_path|test("^docs/|^agents/|^README\\.md$")))|length), code:(.results|map(select(.file_path|test("\\.(rs|ts|tsx|py)$")))|length)}'

    {
      "total": 80,
      "tests": 65,
      "docs": 13,
      "code": 65
    }

    cargo run --quiet -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json | jq \
      '{total:(.results|length), tests:(.results|map(select(.file_path|test("^tests/")))|length), top:(.results|.[0:6]|map({file_path,symbol,score,why_included}))}'

    {
      "total": 6,
      "tests": 4
    }

    cargo run --quiet -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json | jq \
      '{total:(.results|length), impacted:(.results|map(select(.result_kind=="impacted_symbol"))|length), tests:(.results|map(select(.result_kind=="test_target"))|length), d0:(.results|map(select(.distance==0))|length), d1:(.results|map(select(.distance==1))|length), d2:(.results|map(select(.distance==2))|length), d3:(.results|map(select(.distance==3))|length)}'

    {
      "total": 57,
      "impacted": 56,
      "tests": 1,
      "d0": 56,
      "d1": 0,
      "d2": 0,
      "d3": 0
    }

Add strict TDD transcripts per slice in this section as milestones execute.

Milestone 27 strict TDD evidence:

    # 27A red
    cargo test milestone27_context_exclude_tests_omits_test_paths -- --nocapture
    ...
    error: unexpected argument '--exclude-tests' found

    # 27A green
    cargo test milestone27_context_exclude_tests_omits_test_paths -- --nocapture
    ...
    test milestone27_context_exclude_tests_omits_test_paths ... ok

    # 27A refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 27B red
    cargo test milestone27_context_code_only_restricts_to_code_extensions -- --nocapture
    ...
    error: unexpected argument '--code-only' found

    # 27B green
    cargo test milestone27_context_code_only_restricts_to_code_extensions -- --nocapture
    ...
    test milestone27_context_code_only_restricts_to_code_extensions ... ok

    # 27B refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 27C red
    cargo test milestone27_context_scope_flags_preserve_deterministic_json -- --nocapture
    ...
    assertion `left == right` failed
    left: ["function"]
    right: ["function", "type_alias"]

    # 27C green
    cargo test milestone27_context_scope_flags_preserve_deterministic_json -- --nocapture
    ...
    test milestone27_context_scope_flags_preserve_deterministic_json ... ok

    # 27C refactor
    cargo test
    ...
    test result: ok. (full suite)

Milestone 27 post-dogfood evidence:

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo test

    # expected at Milestone 27:
    # - context scoped commands succeed
    # - verify-plan/diff-impact/refs new-flag commands fail with "unexpected argument"
    # - full cargo test suite passes

Milestone 28 strict TDD evidence:

    # 28A red
    cargo test milestone28_verify_plan_changed_line_limits_targeted_symbol_set -- --nocapture
    ...
    error: unexpected argument '--changed-line' found

    # 28A green
    cargo test milestone28_verify_plan_changed_line_limits_targeted_symbol_set -- --nocapture
    ...
    test milestone28_verify_plan_changed_line_limits_targeted_symbol_set ... ok

    # 28A refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 28B red
    cargo test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations -- --nocapture
    ...
    error: unexpected argument '--changed-symbol' found

    # 28B green
    cargo test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations -- --nocapture
    ...
    test milestone28_verify_plan_changed_symbol_filters_targeted_recommendations ... ok

    # 28B refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 28C red check (regression guard already satisfied)
    cargo test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate -- --nocapture
    ...
    test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate ... ok

    # 28C green re-run
    cargo test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate -- --nocapture
    ...
    test milestone28_verify_plan_scope_filters_preserve_changed_test_and_full_suite_gate ... ok

    # 28C refactor
    cargo test
    ...
    test result: ok. (full suite)

Milestone 28 post-dogfood evidence:

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo test

    # expected at Milestone 28:
    # - verify-plan scoped command now succeeds
    # - diff-impact and refs future flags still fail with "unexpected argument"
    # - full cargo test suite passes

Milestone 29 strict TDD evidence:

    # 29A red
    cargo test milestone29_diff_impact_changed_symbol_filters_seed_rows -- --nocapture
    ...
    error: unexpected argument '--changed-symbol' found

    # 29A green
    cargo test milestone29_diff_impact_changed_symbol_filters_seed_rows -- --nocapture
    ...
    test milestone29_diff_impact_changed_symbol_filters_seed_rows ... ok

    # 29A refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 29B red
    cargo test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows -- --nocapture
    ...
    error: unexpected argument '--exclude-changed' found

    # 29B green
    cargo test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows -- --nocapture
    ...
    test milestone29_diff_impact_exclude_changed_hides_distance_zero_rows ... ok

    # 29B refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 29C red
    cargo test milestone29_diff_impact_max_results_caps_deterministically -- --nocapture
    ...
    error: unexpected argument '--max-results' found

    # 29C green
    cargo test milestone29_diff_impact_max_results_caps_deterministically -- --nocapture
    ...
    test milestone29_diff_impact_max_results_caps_deterministically ... ok

    # 29C refactor
    cargo test
    ...
    test result: ok. (full suite)

Milestone 29 post-dogfood evidence:

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo test

    # expected at Milestone 29:
    # - verify-plan and diff-impact scoped commands now succeed
    # - refs --max-results still fails with "unexpected argument" until Milestone 30
    # - full cargo test suite passes

Milestone 30 strict TDD evidence:

    # 30A red
    cargo test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests -- --nocapture
    ...
    assertion `left == right` failed
    left: String("docs/guide.md")
    right: "src/code.rs"

    # 30A green
    cargo test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests -- --nocapture
    ...
    test milestone30_refs_fallback_prefers_code_paths_over_docs_and_tests ... ok

    # 30A refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 30B red
    cargo test milestone30_find_and_refs_max_results_cap_deterministically -- --nocapture
    ...
    error: unexpected argument '--max-results' found

    # 30B green
    cargo test milestone30_find_and_refs_max_results_cap_deterministically -- --nocapture
    ...
    test milestone30_find_and_refs_max_results_cap_deterministically ... ok

    # 30B refactor
    cargo test
    ...
    test result: ok. (full suite)

    # 30C red check (regression guard already satisfied)
    cargo test milestone30_query_caps_compose_with_code_only_and_exclude_tests -- --nocapture
    ...
    test milestone30_query_caps_compose_with_code_only_and_exclude_tests ... ok

    # 30C green re-run
    cargo test milestone30_query_caps_compose_with_code_only_and_exclude_tests -- --nocapture
    ...
    test milestone30_query_caps_compose_with_code_only_and_exclude_tests ... ok

    # 30C refactor
    cargo test
    ...
    test result: ok. (full suite)

Milestone 30 post-dogfood evidence:

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo test

    # expected at Milestone 30:
    # - verify-plan and diff-impact scoped commands succeed
    # - refs --max-results now succeeds with schema 1 deterministic output
    # - full cargo test suite passes

Milestone 31 documentation refresh and final validation evidence:

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json

    # docs update pass:
    # README.md
    # docs/cli-reference.md
    # docs/json-output.md
    # docs/architecture.md
    # docs/dogfood-log.md
    # docs/performance-baseline.md

    cargo run -- index --repo .
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json
    cargo run -- refs helper --repo . --max-results 10 --json
    cargo fmt
    cargo test

    # expected at Milestone 31:
    # - docs and evidence artifacts match implemented Phase 6 behavior
    # - post-refresh dogfood command pack succeeds unchanged
    # - formatting and full suite both pass

## Interfaces and Dependencies

Phase 6 should not require new external crates by default. Continue using the current dependency
set (`clap`, `rusqlite`, `serde`, `tree-sitter` adapters) unless profiling demonstrates a concrete
need.

Likely interfaces to update:

- `src/cli.rs`:
  - `ContextArgs` gains `--code-only` and `--exclude-tests`.
  - `VerifyPlanArgs` gains `--changed-line` and `--changed-symbol`.
  - `DiffImpactArgs` gains `--changed-symbol`, `--exclude-changed`, and `--max-results`.
  - `FindArgs` and `RefsArgs` gain `--max-results`.
- `src/main.rs`:
  - command handlers pass additive options into query-layer functions,
  - changed-line parsing logic is reused for both `verify-plan` and `diff-impact`.
- `src/query/mod.rs`:
  - context scope filtering,
  - verify-plan symbol/line scoping,
  - diff-impact seed filtering + changed-row suppression + cap handling,
  - fallback ranking tie-break updates and cap handling for `find`/`refs`.
- `src/output.rs`:
  - keep schema envelopes stable; only adjust rendering if required by additive option behavior.
- `tests/`:
  - add `tests/milestone27_context_scope.rs`,
  - add `tests/milestone28_verify_plan_scope.rs`,
  - add `tests/milestone29_diff_impact_scope.rs`,
  - add `tests/milestone30_query_focus.rs`.

Backward compatibility requirements:

- Preserve schema 1/2/3 top-level envelopes and required fields.
- Preserve AST-priority behavior for `find`/`refs`.
- Preserve verify-plan full-suite gate semantics.
- Maintain deterministic ordering/tie-break behavior under all new flags.

---

Revision Note (2026-02-07): Created initial Phase 6 execution plan focused on change-scope
precision controls for `context`, `verify-plan`, and `diff-impact`, plus fallback focus controls
for `find`/`refs`, based on post-Phase-5 dogfood evidence. No production code changes were made as
part of this planning step.

Revision Note (2026-02-07): Updated live plan during Milestone 27 implementation with strict TDD
transcripts, post-dogfood command evidence, milestone outcomes, and explicit rationale for running
future-flag dogfood commands before their owning milestones.

Revision Note (2026-02-08): Updated live plan during Milestone 28 implementation with verify-plan
scope transcripts (`--changed-line`, `--changed-symbol`, safety regression guard), post-dogfood
evidence, and updated decision/progress/outcome logs.

Revision Note (2026-02-08): Updated live plan during Milestone 29 implementation with diff-impact
scope transcripts (`--changed-symbol`, `--exclude-changed`, `--max-results`), post-dogfood
evidence, and milestone decision/progress/outcome updates.

Revision Note (2026-02-08): Updated live plan during Milestone 30 implementation with find/refs
fallback ranking and cap transcripts, post-dogfood evidence (including successful
`refs --max-results`), and milestone decision/progress/outcome updates.

Revision Note (2026-02-08): Completed Milestone 31 documentation/evidence refresh with final
post-refresh dogfood transcripts, formatting/test validation, and Phase 6 closeout outcomes.
