# Build `repo-scout` Phase 5 Recommendation Quality and Multi-Hop Impact Fidelity

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan builds on `agents/repo-scout-phase4-execplan.md`, which delivered resolver disambiguation,
`diff-impact` seed controls, and fallback scope flags for `find` and `refs`.

## Purpose / Big Picture

Phase 5 focuses on recommendation quality for agent loops, not command count. After this change,
`tests-for` and `verify-plan` should return fewer broad or non-runnable suggestions by default, and
`context` should better match realistic task phrasing. This phase also closes a contract mismatch in
`diff-impact`: the CLI already accepts `--max-distance > 1`, but current behavior only emits
distance 0 and 1 rows.

User-visible outcome: a tighter “what should I run next” loop with higher-signal suggestions, and
`diff-impact` distance settings that behave as users expect.

## Progress

- [x] (2026-02-07 17:07Z) Re-read `agents/PLANS.md`, `agents/repo-scout-phase4-execplan.md`,
      `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, and `docs/architecture.md` to
      align Phase 5 scope with existing contracts.
- [x] (2026-02-07 17:07Z) Ran required pre-milestone dogfood baseline:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 17:10Z) Captured baseline recommendation-noise evidence for `verify-plan`,
      `tests-for`, and `context`.
- [x] (2026-02-07 17:11Z) Captured baseline traversal evidence proving
      `diff-impact --max-distance > 1` currently emits no distance-2/3 rows.
- [x] (2026-02-07 17:12Z) Authored this Phase 5 ExecPlan as planning-only work.
- [x] (2026-02-07 22:51Z) Ran required pre-milestone dogfood baseline for Milestone 22:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 22:51Z) Completed Milestone 22 strict TDD slices:
      `milestone22_tests_for_excludes_support_paths_by_default`,
      `milestone22_tests_for_prefers_runnable_targets`,
      `milestone22_tests_for_include_support_restores_paths`.
- [x] (2026-02-07 22:51Z) Ran Milestone 22 post-dogfood checks; observed expected
      `verify-plan --max-targeted` CLI failure pending Milestone 23 implementation.
- [x] (2026-02-07 22:51Z) Milestone 22 complete: `tests-for` target-quality contracts and
      implementation.
- [x] (2026-02-07 22:56Z) Ran required pre-milestone dogfood baseline for Milestone 23:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 22:56Z) Completed Milestone 23 strict TDD slices:
      `milestone23_verify_plan_downranks_generic_changed_symbols`,
      `milestone23_verify_plan_applies_targeted_cap_deterministically`,
      `milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate`.
- [x] (2026-02-07 22:56Z) Ran Milestone 23 post-dogfood checks with passing
      `verify-plan --max-targeted` behavior and deterministic targeted capping.
- [x] (2026-02-07 22:56Z) Milestone 23 complete: `verify-plan` high-signal recommendation contracts
      and implementation.
- [x] (2026-02-07 23:00Z) Ran required pre-milestone dogfood baseline for Milestone 24:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 23:00Z) Completed Milestone 24 strict TDD slices:
      `milestone24_context_matches_relevant_symbols_for_paraphrased_task`,
      `milestone24_context_prioritizes_definitions_over_incidental_tokens`,
      `milestone24_context_json_is_stable_with_relevance_scoring`.
- [x] (2026-02-07 23:00Z) Ran Milestone 24 post-dogfood checks with richer context recall and
      deterministic relevance-scored JSON output.
- [x] (2026-02-07 23:00Z) Milestone 24 complete: `context` relevance/recall contracts and
      implementation.
- [x] (2026-02-07 23:07Z) Ran required pre-milestone dogfood baseline for Milestone 25:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 23:07Z) Completed Milestone 25 strict TDD slices:
      `milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors`,
      `milestone25_diff_impact_respects_max_distance_bound`,
      `milestone25_diff_impact_handles_cycles_without_duplicate_growth`.
- [x] (2026-02-07 23:07Z) Ran Milestone 25 post-dogfood checks with deterministic multi-hop
      traversal output and bounded distance behavior.
- [x] (2026-02-07 23:07Z) Milestone 25 complete: true multi-hop `diff-impact` traversal contracts
      and implementation.
- [x] (2026-02-07 23:07Z) Ran required pre-milestone dogfood baseline for Milestone 26:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 23:07Z) Updated Phase 5 docs and evidence artifacts: `README.md`,
      `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`, `docs/dogfood-log.md`,
      `legacy performance baseline doc (removed)`.
- [x] (2026-02-07 23:07Z) Re-ran required post-milestone dogfood checks after docs refresh:
      `cargo run -- index --repo .`, `cargo run -- tests-for Path --repo . --json`,
      `cargo run -- tests-for Path --repo . --include-support --json`,
      `cargo run -- verify-plan --changed-file src/main.rs --repo . --json`,
      `cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json`,
      `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json`,
      `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json`,
      `cargo test`.
- [x] (2026-02-07 23:07Z) Phase 5 docs and dogfood evidence updates complete.

## Surprises & Discoveries

- Observation: `verify-plan` is still over-sensitive to generic symbols in changed files. Evidence:
  `cargo run --quiet -- verify-plan --changed-file src/main.rs --repo . --json | jq` reported
  `total_steps: 21`, `targeted: 20`, with 16 targeted rows sharing why text
  `targeted test references changed symbol 'output'`.

- Observation: `tests-for` can recommend non-runnable support modules. Evidence:
  `cargo run --quiet -- tests-for Path --repo . --json | jq` showed `has_common_mod: true` for
  `tests/common/mod.rs`.

- Observation: `context` has low recall for realistic natural-language phrasing. Evidence:
  `cargo run --quiet -- context --task "...recommendation quality..." --repo . --budget 1200 --json | jq`
  returned `results: 1` and a low-signal module token row (`symbol: "files"`).

- Observation: `diff-impact` currently ignores effective distance beyond one hop. Evidence:
  `cargo run --quiet -- diff-impact --changed-file src/query/mod.rs --max-distance 3 --repo . --json | jq`
  reported `dist2: 0`, `dist3: 0`.

- Observation: fallback text noise remains high when AST matches are absent. Evidence:
  `cargo run --quiet -- refs helper --repo . --json | jq` showed `total: 72`, `tests: 58`, all
  `text_fallback`.

- Observation: post-Milestone-22 dogfood still fails on `verify-plan --max-targeted`. Evidence: CLI
  error `unexpected argument '--max-targeted' found` while running the required post-milestone
  command list; implementation is tracked in Milestone 23.

- Observation: `tests-for` default output now omits support paths, while `--include-support`
  restores them with explicit support classification. Evidence: default `tests-for Path --json` no
  longer emits `tests/common/mod.rs`; with `--include-support`, the row appears as
  `target_kind: "support_test_file"`.

- Observation: strict per-slice TDD required staging future-slice tests after each refactor gate.
  Evidence: adding Milestone 23 tests for slices 23B/23C before finishing 23A caused the 23A
  full-suite refactor gate to fail; resolved by re-adding tests slice-by-slice.

- Observation: post-Milestone-23 verify-plan output for `src/main.rs` shrank to high-signal rows.
  Evidence: required dogfood now returns 2 targeted rows (`milestone14_adapter`,
  `milestone6_schema_migration`) plus the full-suite gate instead of broad 20+ targeted rows.

- Observation: richer context scoring increases recall but can surface test-function symbols quickly
  when task wording overlaps many verification terms. Evidence: post-Milestone-24 dogfood `context`
  returned 6 high-score rows, including `tests/milestone10_validation.rs` and
  `tests/milestone23_verify_plan_precision.rs` symbols.

- Observation: multi-hop traversal can reintroduce changed symbols at `distance > 0` when changed
  files seed many symbols by default. Evidence: slice-25C red test showed `changed_c` emitted as an
  impacted symbol in a cycle until traversal suppressed symbol re-entry for all changed-seed IDs.

- Observation: documentation drifted behind behavior for `context` and `diff-impact` details.
  Evidence: pre-update docs still described one-hop `diff-impact` output and pre-Phase-5 context
  rationale wording.

## Decision Log

- Decision: prioritize recommendation precision and actionability before adding any new command
  families. Rationale: current command surface is broad and stable; remaining user pain is signal
  quality. Date/Author: 2026-02-07 / Codex

- Decision: keep schema versions stable where possible and prefer additive behavior/option changes.
  Rationale: Phase 1-4 contracts are already consumed by automation and should not be churned.
  Date/Author: 2026-02-07 / Codex

- Decision: treat true multi-hop `diff-impact` as required in Phase 5 because the public CLI already
  exposes `--max-distance`. Rationale: current behavior violates the expectation implied by the
  existing interface. Date/Author: 2026-02-07 / Codex

- Decision: default recommendations should prefer runnable test files and suppress support/fixture
  paths unless explicitly requested. Rationale: agent loops need executable steps first; support
  files are useful but secondary. Date/Author: 2026-02-07 / Codex

- Decision: add `tests-for --include-support` as the explicit opt-in, and classify restored support
  results with `target_kind = "support_test_file"` plus support-specific rationale text. Rationale:
  keeps default recommendations runnable-first while preserving additive, deterministic schema-2
  compatibility. Date/Author: 2026-02-07 / Codex

- Decision: treat Milestone 22 `verify-plan --max-targeted` command failure as expected post-check
  evidence until Milestone 23 lands. Rationale: the command is mandated by the plan, but its
  implementation scope belongs to the next milestone. Date/Author: 2026-02-07 / Codex

- Decision: set default verify-plan targeted cap to `DEFAULT_VERIFY_PLAN_MAX_TARGETED = 8` and apply
  `--max-targeted` only to symbol-derived targeted recommendations. Rationale: keeps omitted
  behavior bounded/non-zero while preserving deterministic command quality. Date/Author: 2026-02-07
  / Codex

- Decision: preserve changed runnable test targets even when `--max-targeted=0`. Rationale: changed
  test files are high-priority safety steps and should not be dropped by symbol-target truncation.
  Date/Author: 2026-02-07 / Codex

- Decision: replace exact-symbol-only context matching with deterministic token-overlap scoring
  (including snake/camel tokenization, stopword filtering, and singular/plural overlap handling).
  Rationale: realistic task phrasing is often paraphrased and should still retrieve relevant symbols
  without introducing nondeterminism. Date/Author: 2026-02-07 / Codex

- Decision: bias context ranking toward multi-token definition specificity and standardize why text
  around “token-overlap relevance”. Rationale: prevents short incidental tokens from outranking
  meaningful definitions and enables stable, auditable recommendation rationale. Date/Author:
  2026-02-07 / Codex

- Decision: enforce cycle-safe multi-hop traversal by tracking minimum discovered distance per seed
  and suppressing changed-seed symbol re-entry at non-zero distance. Rationale: prevents duplicate
  growth, avoids changed-symbol echo rows, and keeps traversal deterministic while honoring
  `--max-distance`. Date/Author: 2026-02-07 / Codex

- Decision: keep schema envelopes unchanged for Milestone 26 docs refresh and document Phase 5 as
  behavior/option semantics over existing schema 1/2/3 payloads. Rationale: avoids contract churn
  while making option-driven behavior explicit to users. Date/Author: 2026-02-07 / Codex

## Outcomes & Retrospective

Planning outcome: Phase 5 scope is constrained to recommendation quality and traversal fidelity on
the existing command surface. This avoids schema churn while directly addressing observable dogfood
defects in `tests-for`, `verify-plan`, `context`, and `diff-impact`.

Expected completion outcome: recommendation commands return fewer generic/noisy entries, context
retrieval better reflects user intent text, and `diff-impact` honors multi-hop distance settings
deterministically.

Expected residual work after this plan: deeper semantic/type-aware resolution, broader benchmark
corpora, and potential language-server-backed confidence upgrades.

Milestone 22 outcome: `tests-for` now defaults to runnable integration targets, ranks runnable
targets ahead of support paths, and supports explicit support-path restoration through
`--include-support` without changing schema envelopes.

Milestone 23 outcome: `verify-plan` now dampens generic changed symbols, supports deterministic
targeted capping via `--max-targeted`, and preserves changed runnable test targets alongside the
required full-suite gate.

Milestone 24 outcome: `context` now matches paraphrased task text using deterministic token-overlap
relevance scoring, ranks meaningful definitions above incidental short tokens, and keeps stable JSON
output across repeated runs.

Milestone 25 outcome: `diff-impact` now performs bounded true multi-hop inbound traversal with
deterministic dedupe, emits valid distance-2 neighbors on chain fixtures, and avoids cycle-driven
duplicate growth while preserving schema-3 output shape.

Milestone 26 outcome: user and contributor docs now reflect Phase 5 recommendation-quality and
multi-hop traversal behavior, with updated command examples, schema notes, dogfood transcripts, and
legacy performance command coverage.

## Context and Orientation

`repo-scout` command definitions are in `src/cli.rs`, dispatch is in `src/main.rs`, indexing and
edge persistence are in `src/indexer/mod.rs`, query and recommendation behavior is in
`src/query/mod.rs`, and output contracts are in `src/output.rs`. Integration tests are
milestone-oriented under `tests/`.

Terms used in this plan:

- A “runnable test target” means a file that maps directly to `cargo test --test <name>` under
  `tests/<name>.rs` with no nested module path.
- A “support test path” means a path under `tests/` that is not directly runnable (for example
  `tests/common/mod.rs`) and should not dominate default recommendations.
- A “generic symbol” means a high-frequency token (for example `Path`, `output`, `common`) that
  appears across many files and can overwhelm recommendation ranking.
- A “distance frontier” in `diff-impact` means the set of impacted symbols discovered at each graph
  hop distance from changed-symbol seeds.

Current hot spots:

- `src/query/mod.rs::test_targets_for_symbol` currently groups test-like text matches without a
  runnable/support distinction.
- `src/query/mod.rs::verify_plan_for_changed_files` currently expands all changed-file symbols into
  test recommendations, which amplifies generic tokens.
- `src/query/mod.rs::context_matches` currently relies on exact lowercased symbol equality for
  direct hits, which misses paraphrased tasks.
- `src/query/mod.rs::diff_impact_for_changed_files` currently performs only one-hop incoming
  traversal even when `max_distance > 1`.

## Strict TDD Contract

Phase 5 enforces strict per-slice red-green-refactor. No production code is allowed for a feature
slice before a failing test exists for that exact slice.

A feature slice in this plan is one user-visible behavior unit, such as “support files are excluded
from default `tests-for` output” or “`diff-impact --max-distance 2` emits distance-2 results on a
chain fixture.”

For every slice, record:

- red transcript: failing integration test command,
- green transcript: the same test command passing after minimal implementation,
- refactor transcript: full-suite `cargo test` pass.

Record evidence in this plan and in `docs/dogfood-log.md`.

## Plan of Work

### Milestone 22: Test-target quality for `tests-for`

Milestone goal: prioritize runnable test recommendations and suppress support/fixture noise by
default.

Feature slice 22A defines default suppression of support paths. Add test
`tests/milestone22_recommendation_quality.rs::milestone22_tests_for_excludes_support_paths_by_default`
that asserts `tests/common/mod.rs` is not returned for generic symbol queries when runnable test
targets exist.

Feature slice 22B defines runnable-target priority. Add test
`tests/milestone22_recommendation_quality.rs::milestone22_tests_for_prefers_runnable_targets` that
asserts direct `tests/<name>.rs` targets rank ahead of non-runnable paths when both are present.

Feature slice 22C defines explicit opt-in behavior for suppressed paths. Add test
`tests/milestone22_recommendation_quality.rs::milestone22_tests_for_include_support_restores_paths`
and add a CLI flag (`tests-for --include-support`) to include support/fixture paths when requested.
Keep default behavior unchanged for existing automation unless this flag is present.

Implementation orientation: update `src/cli.rs`, `src/main.rs`, and `src/query/mod.rs` so target
classification and ranking remain deterministic. Keep schema 2 envelopes stable; if new target kind
labels are added, do so additively.

### Milestone 23: High-signal `verify-plan` recommendations

Milestone goal: reduce targeted-step explosion from generic changed-file symbols while preserving
deterministic, actionable outputs.

Feature slice 23A defines generic-symbol damping. Add test
`tests/milestone23_verify_plan_precision.rs::milestone23_verify_plan_downranks_generic_changed_symbols`
that asserts recommendations are not dominated by high-frequency symbols like `output` and `Path`.

Feature slice 23B defines deterministic targeted-step bounds. Add test
`tests/milestone23_verify_plan_precision.rs::milestone23_verify_plan_applies_targeted_cap_deterministically`
for a bounded targeted recommendation list with stable order. Add additive flag
`verify-plan --max-targeted <N>` (default non-zero bounded value) and define clear behavior for `0`
(no targeted rows) versus omitted (default bounded behavior).

Feature slice 23C defines preservation of high-priority safety steps. Add test
`tests/milestone23_verify_plan_precision.rs::milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate`
that asserts changed runnable test files remain included and `cargo test` full-suite gate is always
present regardless of cap/filter settings.

Implementation orientation: update symbol selection and recommendation scoring in
`src/query/mod.rs::verify_plan_for_changed_files`, preserving deterministic tie-breakers and schema
2 output shape.

### Milestone 24: Context retrieval relevance improvements

Milestone goal: improve task-to-symbol recall and ranking for realistic request text without adding
nondeterminism.

Feature slice 24A defines paraphrase recall. Add test
`tests/milestone24_context_relevance.rs::milestone24_context_matches_relevant_symbols_for_paraphrased_task`
that validates direct hits for task text that does not exactly equal symbol names.

Feature slice 24B defines ranking quality. Add test
`tests/milestone24_context_relevance.rs::milestone24_context_prioritizes_definitions_over_incidental_tokens`
that ensures direct definition matches and graph-neighbor rows outrank incidental module/token
matches.

Feature slice 24C defines deterministic stability under richer matching. Add test
`tests/milestone24_context_relevance.rs::milestone24_context_json_is_stable_with_relevance_scoring`
that compares repeated JSON runs for byte-identical output.

Implementation orientation: refine `src/query/mod.rs::extract_keywords` and
`src/query/mod.rs::context_matches` with deterministic matching/scoring adjustments, avoiding schema
changes.

### Milestone 25: True multi-hop `diff-impact` traversal

Milestone goal: make `--max-distance` semantics real for distances greater than one while keeping
precision and deterministic output.

Feature slice 25A defines distance-2 traversal. Add test
`tests/milestone25_diff_impact_multihop.rs::milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors`
using a call-chain fixture where changed symbol C is called by B, which is called by A. Expected
result includes `A` at `distance = 2`.

Feature slice 25B defines traversal bounds and no-overreach behavior. Add test
`tests/milestone25_diff_impact_multihop.rs::milestone25_diff_impact_respects_max_distance_bound`
that confirms `distance > max_distance` rows are absent.

Feature slice 25C defines cycle safety and deterministic dedupe. Add test
`tests/milestone25_diff_impact_multihop.rs::milestone25_diff_impact_handles_cycles_without_duplicate_growth`
that validates stable outputs in cyclic graphs.

Implementation orientation: update `src/query/mod.rs::diff_impact_for_changed_files` to perform
bounded multi-hop inbound traversal with visited tracking and deterministic ordering. Preserve
schema 3 payload shape and existing provenance/confidence vocabulary rules.

### Milestone 26: Documentation and dogfood evidence refresh

Milestone goal: align docs and recorded transcripts with Phase 5 behavior.

Update:

- `README.md`
- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`
- `docs/dogfood-log.md`
- `legacy performance baseline doc (removed)`

Document new options and default recommendation behavior, including support-path handling,
verify-plan cap/filter semantics, context relevance improvements, and multi-hop diff-impact rules.

## Concrete Steps

Run all commands from repository root:

    cd /Users/robertguss/Projects/experiments/repo-scout

Pre-milestone dogfood baseline (before each milestone):

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json

Milestone 22 strict TDD loop:

    cargo test milestone22_tests_for_excludes_support_paths_by_default -- --nocapture
    cargo test milestone22_tests_for_excludes_support_paths_by_default -- --nocapture
    cargo test

    cargo test milestone22_tests_for_prefers_runnable_targets -- --nocapture
    cargo test milestone22_tests_for_prefers_runnable_targets -- --nocapture
    cargo test

    cargo test milestone22_tests_for_include_support_restores_paths -- --nocapture
    cargo test milestone22_tests_for_include_support_restores_paths -- --nocapture
    cargo test

Milestone 23 strict TDD loop:

    cargo test milestone23_verify_plan_downranks_generic_changed_symbols -- --nocapture
    cargo test milestone23_verify_plan_downranks_generic_changed_symbols -- --nocapture
    cargo test

    cargo test milestone23_verify_plan_applies_targeted_cap_deterministically -- --nocapture
    cargo test milestone23_verify_plan_applies_targeted_cap_deterministically -- --nocapture
    cargo test

    cargo test milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate -- --nocapture
    cargo test milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate -- --nocapture
    cargo test

Milestone 24 strict TDD loop:

    cargo test milestone24_context_matches_relevant_symbols_for_paraphrased_task -- --nocapture
    cargo test milestone24_context_matches_relevant_symbols_for_paraphrased_task -- --nocapture
    cargo test

    cargo test milestone24_context_prioritizes_definitions_over_incidental_tokens -- --nocapture
    cargo test milestone24_context_prioritizes_definitions_over_incidental_tokens -- --nocapture
    cargo test

    cargo test milestone24_context_json_is_stable_with_relevance_scoring -- --nocapture
    cargo test milestone24_context_json_is_stable_with_relevance_scoring -- --nocapture
    cargo test

Milestone 25 strict TDD loop:

    cargo test milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors -- --nocapture
    cargo test milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors -- --nocapture
    cargo test

    cargo test milestone25_diff_impact_respects_max_distance_bound -- --nocapture
    cargo test milestone25_diff_impact_respects_max_distance_bound -- --nocapture
    cargo test

    cargo test milestone25_diff_impact_handles_cycles_without_duplicate_growth -- --nocapture
    cargo test milestone25_diff_impact_handles_cycles_without_duplicate_growth -- --nocapture
    cargo test

Post-milestone dogfood checks:

    cargo run -- index --repo .
    cargo run -- tests-for Path --repo . --json
    cargo run -- tests-for Path --repo . --include-support --json
    cargo run -- verify-plan --changed-file src/main.rs --repo . --json
    cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json
    cargo test

Before finalizing:

    cargo fmt
    cargo test

## Validation and Acceptance

Acceptance is behavior-first and repository-observable.

For `tests-for`:

- default output excludes support/fixture paths when runnable targets exist,
- runnable targets stay deterministically ordered and stable across runs,
- `--include-support` restores suppressed paths deterministically.

For `verify-plan`:

- targeted recommendations are no longer dominated by generic symbols,
- targeted-step counts obey deterministic cap behavior,
- full-suite `cargo test` step is always retained.

For `context`:

- realistic task text produces higher recall than exact-symbol-only matching,
- ranking remains deterministic and reproducible in JSON.

For `diff-impact`:

- `--max-distance 2+` emits valid multi-hop impacted symbols when graph topology allows,
- traversal never emits rows beyond the requested distance,
- cycles do not cause duplicate growth or nondeterministic output.

Final acceptance requires strict red/green/refactor evidence for every slice, full-suite pass, and
updated docs/dogfood logs.

## Idempotence and Recovery

Indexing remains idempotent and unchanged by this phase. Query-layer changes must preserve
determinism across repeated runs on identical index state.

New recommendation filters/caps must be explicit, reversible by option when applicable, and should
not require database resets or schema rewrites unless an additive migration is justified and tested.

If traversal changes in Milestone 25 increase result volume unexpectedly, the implementation must
include deterministic dedupe and bounded frontier handling to maintain predictable runtime and
output.

## Artifacts and Notes

Baseline evidence captured before implementation:

    cargo run --quiet -- verify-plan --changed-file src/main.rs --repo . --json | jq \
      '{total_steps:(.results|length), targeted:(.results|map(select(.scope=="targeted"))|length), full_suite:(.results|map(select(.scope=="full_suite"))|length), top_reasons:(.results|map(select(.scope=="targeted")|.why_included)|group_by(.)|map({why:.[0],n:length})|sort_by(-.n)|.[0:5])}'

    {
      "total_steps": 21,
      "targeted": 20,
      "full_suite": 1,
      "top_reasons": [
        { "why": "targeted test references changed symbol 'output'", "n": 16 }
      ]
    }

    cargo run --quiet -- tests-for Path --repo . --json | jq \
      '{total_targets:(.results|length), has_common_mod:(.results|any(.target=="tests/common/mod.rs"))}'

    {
      "total_targets": 6,
      "has_common_mod": true
    }

    cargo run --quiet -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json | jq \
      '{results:(.results|length), top:(.results|.[0:3])}'

    {
      "results": 1
    }

    cargo run --quiet -- diff-impact --changed-file src/query/mod.rs --max-distance 3 --repo . --json | jq \
      '{dist1:(.results|map(select(.distance==1))|length), dist2:(.results|map(select(.distance==2))|length), dist3:(.results|map(select(.distance==3))|length)}'

    {
      "dist1": 18,
      "dist2": 0,
      "dist3": 0
    }

Milestone 22 strict TDD evidence:

    # Slice 22A red
    cargo test milestone22_tests_for_excludes_support_paths_by_default -- --nocapture
    # observed: FAILED (default output still contained tests/common/mod.rs)

    # Slice 22A green
    cargo test milestone22_tests_for_excludes_support_paths_by_default -- --nocapture
    # observed: ok

    # Slice 22A refactor gate
    cargo test
    # observed: full suite passed

    # Slice 22B red
    cargo test milestone22_tests_for_prefers_runnable_targets -- --nocapture
    # observed: FAILED (CLI rejected --include-support)

    # Slice 22B green
    cargo test milestone22_tests_for_prefers_runnable_targets -- --nocapture
    # observed: ok

    # Slice 22B refactor gate
    cargo test
    # observed: full suite passed

    # Slice 22C red
    cargo test milestone22_tests_for_include_support_restores_paths -- --nocapture
    # observed: FAILED (support row reason did not mention support path)

    # Slice 22C green
    cargo test milestone22_tests_for_include_support_restores_paths -- --nocapture
    # observed: ok

    # Slice 22C refactor gate
    cargo test
    # observed: full suite passed

Milestone 22 post-dogfood evidence:

    cargo run -- tests-for Path --repo . --json
    # observed: no tests/common/mod.rs row

    cargo run -- tests-for Path --repo . --include-support --json
    # observed: tests/common/mod.rs returned with target_kind "support_test_file"

    cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
    # observed: unexpected argument '--max-targeted' found (expected before Milestone 23)

Milestone 23 strict TDD evidence:

    # Slice 23A red
    cargo test milestone23_verify_plan_downranks_generic_changed_symbols -- --nocapture
    # observed: FAILED (verify-plan still recommended generic_output/generic_path)

    # Slice 23A green
    cargo test milestone23_verify_plan_downranks_generic_changed_symbols -- --nocapture
    # observed: ok

    # Slice 23A refactor gate
    cargo test
    # observed: full suite passed

    # Slice 23B red
    cargo test milestone23_verify_plan_applies_targeted_cap_deterministically -- --nocapture
    # observed: FAILED (CLI rejected --max-targeted)

    # Slice 23B green
    cargo test milestone23_verify_plan_applies_targeted_cap_deterministically -- --nocapture
    # observed: ok

    # Slice 23B refactor gate
    cargo test
    # observed: full suite passed

    # Slice 23C red
    cargo test milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate -- --nocapture
    # observed: FAILED (changed runnable test target dropped at --max-targeted=0)

    # Slice 23C green
    cargo test milestone23_verify_plan_preserves_changed_test_target_and_full_suite_gate -- --nocapture
    # observed: ok

    # Slice 23C refactor gate
    cargo test
    # observed: full suite passed

Milestone 23 post-dogfood evidence:

    cargo run -- verify-plan --changed-file src/main.rs --repo . --json
    # observed: 2 targeted rows + full-suite gate

    cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
    # observed: deterministic output identical to capped baseline for this fixture

Milestone 24 strict TDD evidence:

    # Slice 24A red
    cargo test milestone24_context_matches_relevant_symbols_for_paraphrased_task -- --nocapture
    # observed: FAILED (paraphrased task did not return verify_plan_for_changed_files)

    # Slice 24A green
    cargo test milestone24_context_matches_relevant_symbols_for_paraphrased_task -- --nocapture
    # observed: ok

    # Slice 24A refactor gate
    cargo test
    # observed: full suite passed

    # Slice 24B red
    cargo test milestone24_context_prioritizes_definitions_over_incidental_tokens -- --nocapture
    # observed: FAILED (incidental symbol 'plan' ranked first)

    # Slice 24B green
    cargo test milestone24_context_prioritizes_definitions_over_incidental_tokens -- --nocapture
    # observed: ok

    # Slice 24B refactor gate
    cargo test
    # observed: full suite passed

    # Slice 24C red
    cargo test milestone24_context_json_is_stable_with_relevance_scoring -- --nocapture
    # observed: FAILED (why_included lacked standardized token-overlap rationale text)

    # Slice 24C green
    cargo test milestone24_context_json_is_stable_with_relevance_scoring -- --nocapture
    # observed: ok

    # Slice 24C refactor gate
    cargo test
    # observed: full suite passed

Milestone 24 post-dogfood evidence:

    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    # observed: 6 deterministic high-signal rows with token-overlap rationale text

Milestone 25 strict TDD evidence:

    # Slice 25A red
    cargo test milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors -- --nocapture
    # observed: FAILED (no distance=2 impacted row before multi-hop traversal)

    # Slice 25A green
    cargo test milestone25_diff_impact_max_distance_two_emits_distance_two_neighbors -- --nocapture
    # observed: ok

    # Slice 25A refactor gate
    cargo test
    # observed: full suite passed

    # Slice 25B red
    cargo test milestone25_diff_impact_respects_max_distance_bound -- --nocapture
    # observed: FAILED (distance exceeded requested bound before traversal cap enforcement)

    # Slice 25B green
    cargo test milestone25_diff_impact_respects_max_distance_bound -- --nocapture
    # observed: ok

    # Slice 25B refactor gate
    cargo test
    # observed: full suite passed

    # Slice 25C red
    cargo test milestone25_diff_impact_handles_cycles_without_duplicate_growth -- --nocapture
    # observed: FAILED (cycle traversal re-emitted changed symbol at non-zero distance)

    # Slice 25C green
    cargo test milestone25_diff_impact_handles_cycles_without_duplicate_growth -- --nocapture
    # observed: ok

    # Slice 25C refactor gate
    cargo test
    # observed: full suite passed

Milestone 25 post-dogfood evidence:

    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json
    # observed: command succeeded with deterministic schema-3 payload and bounded distances

    cargo run -- tests-for Path --repo . --json
    # observed: runnable-only targets by default (no support path rows)

    cargo run -- tests-for Path --repo . --include-support --json
    # observed: support row restored as target_kind "support_test_file"

    cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
    # observed: deterministic capped targeted steps plus full-suite gate

Milestone 26 post-dogfood evidence:

    cargo run -- tests-for Path --repo . --json
    # observed: runnable-only deterministic target list

    cargo run -- tests-for Path --repo . --include-support --json
    # observed: support path restored with target_kind "support_test_file"

    cargo run -- verify-plan --changed-file src/main.rs --repo . --max-targeted 6 --json
    # observed: bounded targeted rows remained deterministic; full-suite gate retained

    cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json
    # observed: token-overlap rationale rows remained stable at schema_version 2

    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --max-distance 3 --json
    # observed: schema_version 3 output remained deterministic with bounded traversal

## Interfaces and Dependencies

Phase 5 should not require new external crates by default. Continue using existing dependencies
(`clap`, `rusqlite`, `serde`, `tree-sitter` family) unless profiling or determinism requirements
demonstrate a concrete need.

Likely interfaces to update:

- `src/cli.rs`:
  - `TestsFor` command args may gain `--include-support`.
  - `VerifyPlan` command args may gain `--max-targeted`.
- `src/main.rs`:
  - command handlers pass new options into query functions.
- `src/query/mod.rs`:
  - test target classification/ranking behavior.
  - verify-plan symbol filtering/scoring/cap behavior.
  - context matching/scoring behavior.
  - bounded multi-hop traversal in diff-impact.
- `src/output.rs`:
  - JSON/terminal output only if additive fields are required by approved slices.

Backward compatibility requirements:

- Preserve schema 1, 2, and 3 envelope shapes unless an additive field is required and explicitly
  documented.
- Keep deterministic ordering/tie-break behavior explicit and test-locked.

---

Revision Note (2026-02-07): Created initial Phase 5 execution plan to address recommendation noise
(`tests-for`, `verify-plan`, `context`) and `diff-impact` multi-hop fidelity gaps discovered during
post-Phase-4 dogfooding. No production code changes were made as part of this planning step.

Revision Note (2026-02-07): Updated progress, decisions, surprises, outcomes, and artifacts with
Milestone 22 implementation details, strict TDD transcripts, and post-milestone dogfood evidence.

Revision Note (2026-02-07): Updated progress, decisions, surprises, outcomes, and artifacts with
Milestone 23 implementation details, strict TDD transcripts, and capped verify-plan dogfood
evidence.

Revision Note (2026-02-07): Updated progress, decisions, surprises, outcomes, and artifacts with
Milestone 24 context relevance work, strict TDD transcripts, and refreshed dogfood evidence.

Revision Note (2026-02-07): Updated progress, decisions, surprises, outcomes, and artifacts with
Milestone 25 multi-hop traversal work, strict TDD transcripts, and post-milestone dogfood evidence.

Revision Note (2026-02-07): Updated Phase 5 documentation set (`README`, CLI/JSON/architecture,
dogfood log, performance baseline) and marked Milestone 26 completion in the living plan.
