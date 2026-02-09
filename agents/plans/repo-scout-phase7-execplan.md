# Build `repo-scout` Phase 7 Cross-Language Semantic Precision and Quality Benchmark Guardrails

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan builds on `agents/repo-scout-phase6-execplan.md`, which delivered change-scope controls
for `context`, `verify-plan`, `diff-impact`, and deterministic caps for `find`/`refs`.

## Purpose / Big Picture

Phase 7 focuses on semantic correctness and measurable quality for existing language adapters (Rust,
TypeScript, Python), not on adding new command families. After this change, users should be able to
trust that `diff-impact` and `impact` follow imported module aliases in TypeScript and Python even
when duplicate symbol names exist across files, and maintain deterministic high-signal ranking in
mixed fallback-heavy results. The phase also introduces benchmark-style quality fixtures and
repeatable command packs so recommendation/ranking regressions can be caught early.

User-visible outcome: fewer missed impacted callers in cross-file TypeScript/Python changes, fewer
ambiguous symbol collisions in impact analysis, and a repeatable benchmark corpus that quantifies
recommendation quality under known noisy scenarios.

## Progress

- [x] (2026-02-08 01:31Z) Re-read `agents/PLANS.md` and `agents/implementation_prompt.md` to align
      this plan with strict TDD, living-plan, and evidence requirements.
- [x] (2026-02-08 01:31Z) Re-read `agents/repo-scout-hybrid-rust-execplan.md`,
      `agents/repo-scout-agent-first-phase2-execplan.md`, `agents/repo-scout-phase3-execplan.md`,
      `agents/repo-scout-phase4-execplan.md`, `agents/repo-scout-phase5-execplan.md`, and
      `agents/repo-scout-phase6-execplan.md` to consolidate residual-work threads.
- [x] (2026-02-08 01:31Z) Ran planning baseline dogfood commands: `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-08 01:31Z) Captured baseline semantic-gap evidence for TypeScript namespace imports
      and Python module-alias calls where `diff-impact` currently returns only changed symbols when
      duplicate callee names exist.
- [x] (2026-02-08 01:31Z) Authored this Phase 7 ExecPlan as planning-only work.
- [x] (2026-02-08 01:39Z) Ran required pre-milestone baseline dogfood for Milestone 32:
      `cargo run -- index --repo .`, `cargo run -- find resolve_symbol_id_in_tx --repo . --json`,
      `cargo run -- refs resolve_symbol_id_in_tx --repo . --json`.
- [x] Milestone 32 strict TDD contract-lock slices (red failures established in
      `tests/milestone32_semantic_contracts.rs`; green/refactor completion landed in Milestones
      33-34 and is now closed).
- [x] (2026-02-08 01:43Z) Ran required pre-milestone baseline dogfood for Milestone 33:
      `cargo run -- index --repo .`, `cargo run -- find resolve_symbol_id_in_tx --repo . --json`,
      `cargo run -- refs resolve_symbol_id_in_tx --repo . --json`.
- [x] (2026-02-08 01:46Z) Milestone 33 strict TDD slices for TypeScript namespace/module-aware call
      resolution completed, including delayed full-suite refactor closure once Milestone 34 resolved
      remaining Milestone 32 Python contract failures.
- [x] (2026-02-08 01:43Z) Ran required pre-milestone baseline dogfood for Milestone 34:
      `cargo run -- index --repo .`, `cargo run -- find resolve_symbol_id_in_tx --repo . --json`,
      `cargo run -- refs resolve_symbol_id_in_tx --repo . --json`.
- [x] (2026-02-08 01:46Z) Completed Milestone 34 strict TDD slices for Python module-aware call
      resolution; reran full suite (`cargo test`) green and restored Milestone 32 contract tests to
      passing.
- [x] (2026-02-08 01:46Z) Closed Milestone 32 contract-lock suite with green behavior after
      Milestones 33-34 semantic implementations.
- [x] (2026-02-08 01:46Z) Ran required pre-milestone baseline dogfood for Milestone 35:
      `cargo run -- index --repo .`, `cargo run -- find resolve_symbol_id_in_tx --repo . --json`,
      `cargo run -- refs resolve_symbol_id_in_tx --repo . --json`.
- [x] (2026-02-08 01:50Z) Completed Milestone 35 strict TDD slices for semantic-confidence
      ranking/benchmark guardrails, including calibrated scoring in `impact`/`diff-impact`,
      benchmark fixture assertions, and full-suite refactor gate (`cargo test`) green.
- [x] (2026-02-08 02:12Z) Completed Milestone 36 documentation/evidence refresh across `README.md`,
      `docs/cli-reference.md`, `docs/json-output.md`, `docs/architecture.md`, `docs/dogfood-log.md`,
      and `docs/performance-baseline.md`.
- [x] (2026-02-08 02:13Z) Ran Milestone 36 post-refresh verification pack:
      `cargo run -- index --repo .`,
      `cargo run -- diff-impact --changed-file src/indexer/languages/typescript.rs --repo . --json`,
      `cargo run -- diff-impact --changed-file src/indexer/languages/python.rs --repo . --json`,
      `cargo run -- refs helper --repo . --code-only --exclude-tests --max-results 10 --json`,
      `cargo test`, `cargo fmt`.

## Surprises & Discoveries

- Observation: TypeScript namespace alias calls currently lose `called_by` edges when duplicate
  callee names exist across modules. Evidence: In a temporary repo with `util_a.helper`,
  `util_b.helper`, and `import * as utilA from "./util_a"; utilA.helper();`, running
  `cargo run -- diff-impact --changed-file src/util_a.ts --repo <tmp> --json` returned only one
  impacted row (`changed_symbol` for `src/util_a.ts::helper`) and no caller row.

- Observation: Python module-alias attribute calls currently lose `called_by` edges when duplicate
  callee names exist across modules. Evidence: In a temporary repo with `pkg_a.util.helper`,
  `pkg_b.util.helper`, and `import pkg_a.util as util; util.helper()`, running
  `cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo <tmp> --json` returned only one
  impacted row (`changed_symbol` for `src/pkg_a/util.py::helper`) and no caller.

- Observation: Resolver fallback is intentionally conservative when symbol-only lookup is ambiguous,
  which avoids wrong edges but can suppress real edges when import/module context is missing.
  Evidence: `src/indexer/mod.rs::resolve_symbol_id_in_tx` returns `Ok(None)` when language-scoped
  symbol lookup returns more than one row (`LIMIT 2` ambiguity check).

- Observation: Repository self-dogfood still shows fallback-heavy noise for broad text tokens.
  Evidence: `cargo run --quiet -- refs helper --repo . --json | jq ...` reported `total: 99`,
  `tests: 71`, `docs: 3`.

- Observation: Phase 3 TypeScript/Python adapter coverage exists for named/default imports and alias
  forms, but namespace/module-alias call disambiguation is not yet contract-locked. Evidence:
  existing suites (`tests/milestone15_typescript.rs` and `tests/milestone16_python.rs`) validate MVP
  extraction/edges but do not currently lock the duplicate-name namespace/module-alias caller-impact
  scenario.

- Observation: New Milestone 32 contract tests fail exactly at missing caller rows while schema
  envelopes remain stable. Evidence:
  `cargo test milestone32_typescript_namespace_alias_call_contract -- --nocapture` failed with
  missing `src/app.ts::run` `called_by`;
  `cargo test milestone32_python_module_alias_call_contract -- --nocapture` failed with missing
  `src/py_app.py::run_py` `called_by`;
  `cargo test milestone32_schema_contracts_stay_stable -- --nocapture` failed on missing schema-3
  semantic caller rows.

- Observation: Milestone 33 TypeScript changes produce caller rows for namespace alias calls in
  fixture dogfood, while Python fixture still shows only changed-symbol rows before Milestone 34.
  Evidence:
  `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json`
  returned `src/app.ts::run` `called_by`, while the matching Python command for `src/pkg_a/util.py`
  returned only `changed_symbol`.

- Observation: Milestone 33 regression guard slice (preserve Milestone 15 behavior) was already
  green before TypeScript production edits and stayed green after import-context updates. Evidence:
  `cargo test milestone33_typescript_semantics_preserve_existing_m15_behavior -- --nocapture` passed
  in both pre-change and post-change runs.

- Observation: Fixture behavior-check output can remain stale after extractor-code changes when file
  contents are unchanged because incremental indexing keys off content hash, not binary extractor
  version. Evidence: first Milestone 34 fixture pack run reported `indexed_files: 0` and omitted
  Python caller rows; after a no-behavior fixture content touch and rerun (`indexed_files: 1`), the
  same command emitted `src/py_app.py::run_py` as `called_by`.

- Observation: Query-layer calibration can ship independently of index rebuild because it only
  transforms persisted edge/test evidence during read-time ranking. Evidence: Milestone 35 fixture
  behavior-check pack reported calibrated `score: 0.97` for both TypeScript and Python `called_by`
  rows even with `indexed_files: 0` on fixture reindex.

- Observation: Final closeout `cargo fmt` touched only formatting in already-green Phase 7
  production/test files; behavior stayed unchanged. Evidence: `git diff` after Milestone 36
  verification showed line-wrap-only edits in `src/indexer/languages/typescript.rs`,
  `src/indexer/languages/python.rs`, and milestone test files with no semantic logic deltas.

## Decision Log

- Decision: Keep schema envelopes stable (`schema_version` 1/2/3) through Phase 7 and implement
  behavior improvements through adapter resolution and deterministic ranking updates. Rationale:
  Existing automation depends on current output contracts; Phase 7 targets precision, not contract
  churn. Date/Author: 2026-02-08 / Codex

- Decision: Sequence Phase 7 as contract-lock first, then TypeScript semantics, then Python
  semantics, then query-ranking/benchmark guardrails. Rationale: This preserves strict TDD and
  avoids writing production behavior before failures are observable for each user-visible slice.
  Date/Author: 2026-02-08 / Codex

- Decision: Prioritize deepening existing TypeScript/Python semantics over adding a new language in
  this phase. Rationale: TypeScript and Python adapters are already shipped; current high-impact gap
  is resolution precision in realistic alias/module workflows. Date/Author: 2026-02-08 / Codex

- Decision: Defer full runtime language-server integration in Phase 7, while introducing
  deterministic confidence calibration and extension seams for future LSP-backed upgrades.
  Rationale: External language-server runtime dependencies would weaken deterministic offline
  behavior and expand phase risk; confidence improvements can be delivered now from local evidence.
  Date/Author: 2026-02-08 / Codex

- Decision: Preserve Rust-specific runnable test command generation semantics in `verify-plan` for
  this phase. Rationale: Phase 7 scope is cross-language semantic impact precision and quality
  measurement; multi-runner command inference (`pytest`, `jest`, etc.) is a separate problem and
  should be planned explicitly in a later phase. Date/Author: 2026-02-08 / Codex

- Decision: Keep Milestone 32 contract tests asserting final semantic behavior (not current gaps),
  and carry their green/refactor closure into Milestones 33-34 while the owning TypeScript/Python
  behavior slices are implemented. Rationale: This is the smallest plan-aligned path that preserves
  strict red-before-production evidence while avoiding duplicate temporary contract rewrites.
  Date/Author: 2026-02-08 / Codex

- Decision: Add the shared benchmark fixture tree at `tests/fixtures/phase7/semantic_precision/`
  during Milestone 33 so required behavior-check command packs can run after Milestones 33/34/35
  without ad-hoc setup. Rationale: Required command packs execute immediately after each milestone;
  creating the fixture once early keeps evidence deterministic and repeatable. Date/Author:
  2026-02-08 / Codex

- Decision: Keep fixture semantics unchanged but allow benign content refreshes when required to
  force reindexing after extractor implementation changes during dogfood verification. Rationale:
  Incremental index skips unchanged files by design; forcing at least one changed file avoids stale
  edge evidence without altering command contracts or schema behavior. Date/Author: 2026-02-08 /
  Codex

- Decision: Introduce deterministic query-time semantic score baselines keyed by
  relationship/provenance/distance, and keep confidence labels stable (`graph_likely`) for semantic
  edge rows. Rationale: This improves ranking precision and benchmark signal without schema churn or
  broad downstream label contract changes. Date/Author: 2026-02-08 / Codex

- Decision: Keep rustfmt-only edits produced during Milestone 36 closeout in the milestone commit
  instead of attempting selective rollback. Rationale: Final acceptance requires `cargo fmt` clean
  state; retaining deterministic formatter output is the smallest plan-aligned path and avoids risky
  manual reformat divergence. Date/Author: 2026-02-08 / Codex

## Outcomes & Retrospective

Planning outcome: Phase 7 is constrained to high-value semantic precision and benchmark guardrails
on the existing command surface, with strict per-slice TDD and no new command families.

Expected completion outcome: `impact`/`diff-impact` correctly include callers for TypeScript
namespace imports and Python module aliases even under duplicate symbol names, while deterministic
ranking and confidence semantics remain stable and benchmarked.

Expected residual work after this plan: broader multi-repository telemetry ingestion, optional
runtime language-server confidence augmentation, and explicit planning for language-specific test
runner recommendation commands.

Milestone 33 outcome (2026-02-08): TypeScript namespace/member-call extraction now consumes
module-aware import hints, producing deterministic `called_by` rows for `utilA.helper()` style calls
under duplicate-name ambiguity while preserving Milestone 15 import/implements behavior.

Milestone 34 outcome (2026-02-08): Python attribute-call extraction now uses module-alias import
context, producing deterministic `called_by` rows for `import pkg.mod as alias; alias.func()`
patterns under duplicate-name ambiguity while preserving Milestone 16 dotted-import behavior.

Milestone 35 outcome (2026-02-08): `impact`/`diff-impact` now apply deterministic semantic score
calibration by relationship/provenance/distance, benchmark fixture checks lock high-confidence
caller ranking (`score >= 0.96` for semantic caller rows), and ordering remains deterministic
without schema contract changes.

Milestone 36 outcome (2026-02-08): documentation and evidence now reflect shipped Phase 7 behavior
(module-aware TypeScript/Python semantic resolution, calibrated ranking, and benchmark command
packs), post-refresh dogfood checks passed, full test suite passed, and formatter state is clean.

## Context and Orientation

`repo-scout` command parsing lives in `src/cli.rs`, dispatch and argument normalization in
`src/main.rs`, indexing and resolver logic in `src/indexer/mod.rs`, language adapters in
`src/indexer/languages/`, query/ranking behavior in `src/query/mod.rs`, schema management in
`src/store/schema.rs`, and integration tests under `tests/`.

Terms used in this plan:

- A "module-aware import mapping" means resolving an imported alias to the actual module file path
  (for example `utilA -> src/util_a.ts`) so member calls can target the correct definition.
- A "type-aware or semantic resolution" in this phase means call/import/implements edges use
  import/module context, not only raw symbol name matching.
- A "semantic edge" means a `symbol_edges_v2` row derived from AST plus module/import hints and
  persisted with deterministic provenance (`call_resolution`, `import_resolution`, `ast_definition`,
  `ast_reference`).
- A "benchmark corpus" means deterministic fixture repositories that encode noisy/ambiguous
  scenarios and are exercised via automated tests and dogfood command packs.
- A "confidence calibration" means deterministic mapping of evidence quality (provenance,
  relationship type, and ambiguity) to score/confidence ordering so results are stable and
  interpretable.

Current hot spots for this phase:

- `src/indexer/mod.rs::resolve_symbol_id_in_tx` currently falls back from qualified/scoped lookups
  to symbol-only disambiguation and returns unresolved on ambiguity.
- `src/indexer/languages/typescript.rs::collect_call_symbols` currently records member-call target
  symbols without module-path import context.
- `src/indexer/languages/python.rs::import_bindings` currently captures limited import symbol data,
  and `collect_call_symbols` currently resolves attribute-call targets by symbol text only.
- `src/query/mod.rs::diff_impact_for_changed_files` and `src/query/mod.rs::impact_matches` currently
  depend on persisted edge quality; ranking can be improved via confidence calibration.
- `docs/performance-baseline.md` currently tracks runtime timings but does not include a dedicated
  phase-level semantic quality benchmark corpus.

## Strict TDD Contract

Phase 7 enforces strict red-green-refactor for every feature slice. No production code changes are
allowed before a failing automated test exists for the exact behavior slice being implemented.

A feature slice in this plan is one user-visible behavior unit, such as "TypeScript namespace alias
call resolves to imported module function under duplicate-name ambiguity" or "semantic
call-resolution rows outrank fallback-only rows deterministically."

For each slice, record:

- red transcript: failing test command with expected failure reason,
- green transcript: same test command passing after minimum implementation,
- refactor transcript: full-suite `cargo test` pass.

Evidence must be recorded in this plan and appended to `docs/dogfood-log.md`.

## Plan of Work

### Milestone 32: Lock Phase 7 semantic contracts with failing integration tests

Milestone goal: encode current semantic precision gaps as deterministic failing tests before
implementation.

Feature slice 32A locks the TypeScript namespace-import caller-impact gap. Add
`tests/milestone32_semantic_contracts.rs` coverage for a fixture with `util_a.helper`,
`util_b.helper`, and `utilA.helper()` namespace call usage. The red state proves that changing
`src/util_a.ts` currently misses caller impact rows.

Feature slice 32B locks the Python module-alias caller-impact gap. In the same test file, add a
fixture with `pkg_a.util.helper`, `pkg_b.util.helper`, and `util.helper()` usage via
`import pkg_a.util as util`. The red state proves that changing `src/pkg_a/util.py` currently misses
caller impact rows.

Feature slice 32C locks contract stability constraints: schema versions remain 1/2/3 and output
shape remains backward compatible while new impacted rows are added.

### Milestone 33: Implement TypeScript module-aware import and call resolution

Milestone goal: make namespace/member-call edges deterministic and precise in TypeScript when
duplicate symbol names exist.

Feature slice 33A extends TypeScript import parsing to capture namespace/default/named bindings with
resolved module paths in a normalized form (repository-relative `.ts`/`.tsx` path candidates).
Update helper structures in `src/indexer/languages/typescript.rs` so call resolution can consume
binding context.

Feature slice 33B upgrades member-call extraction to use alias/module hints. For calls like
`utilA.helper()`, emit `SymbolKey` targets with explicit module-qualified intent
(`typescript:<module_path>::helper`) so resolver ambiguity is removed.

Feature slice 33C preserves existing Milestone 15 behavior and deterministic ordering: named/default
imports, implements edges, and existing call/reference extraction remain green. Refactors should
remain local to adapter helper functions and avoid query-layer behavior changes in this milestone.

### Milestone 34: Implement Python module-aware import and call resolution

Milestone goal: make module-alias attribute-call edges deterministic and precise in Python when
duplicate symbol names exist.

Feature slice 34A extends Python import parsing in `src/indexer/languages/python.rs` to capture
module-path hints for `import pkg.mod as alias` and `from pkg.mod import fn as alias` forms, with
normalized repository-relative `.py` targets where resolvable.

Feature slice 34B upgrades attribute-call extraction (`alias.func()` and dotted forms) to use alias
maps and emit qualified `SymbolKey` targets when module context is known. This must produce stable
edges even when duplicate function names exist in different Python modules.

Feature slice 34C preserves existing Milestone 16 behavior and determinism: AST references for
imports remain present, symbol/edge ordering remains stable, and prior fixture assertions stay
green.

### Milestone 35: Query precision calibration and benchmark-quality guardrails

Milestone goal: convert improved semantic edges into stable ranking improvements and measurable
quality checks.

Feature slice 35A introduces deterministic confidence calibration helpers in `src/query/mod.rs` for
edge-derived impacted rows. Calibration must treat stronger semantic evidence
(`call_resolution`/`import_resolution` with resolved qualified targets) above fallback-only rows
without changing schema shape.

Feature slice 35B applies calibrated ordering in `impact`/`diff-impact` sorting while preserving
existing deterministic tie-breaks and scope-cap behaviors (`--max-results`, `--exclude-changed`,
`--code-only`, `--exclude-tests`).

Feature slice 35C adds a benchmark-quality fixture corpus under
`tests/fixtures/phase7/semantic_precision/` and integration tests
(`tests/milestone35_quality_benchmark.rs`) that assert high-signal impacted caller recall under
duplicate-name ambiguity and stable ordering across repeated runs.

### Milestone 36: Documentation, dogfood evidence, and baseline refresh

Milestone goal: align docs/evidence with Phase 7 semantics and benchmark guardrails.

Feature slice 36A updates user-facing docs (`README.md`, `docs/cli-reference.md`,
`docs/json-output.md`, `docs/architecture.md`) to describe improved TypeScript/Python semantic
resolution behavior and confidence/ranking semantics.

Feature slice 36B updates `docs/dogfood-log.md` with Phase 7 red/green/refactor and dogfood
transcripts, and extends `docs/performance-baseline.md` with a Phase 7 quality command pack covering
semantic fixtures.

Feature slice 36C re-runs the full required dogfood pack and full-suite tests after docs refresh to
verify behavior/docs alignment remains stable.

## Concrete Steps

Run all commands from `<repo-root>` (the repository root directory).

Before each milestone, run baseline dogfood:

    cargo run -- index --repo .
    cargo run -- find resolve_symbol_id_in_tx --repo . --json
    cargo run -- refs resolve_symbol_id_in_tx --repo . --json

Strict per-slice TDD loop (required order, every slice):

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure before production edits
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimum implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 32 expected slice commands:

    cargo test milestone32_typescript_namespace_alias_call_contract -- --nocapture
    cargo test milestone32_python_module_alias_call_contract -- --nocapture
    cargo test milestone32_schema_contracts_stay_stable -- --nocapture

Milestone 33 expected slice commands:

    cargo test milestone33_typescript_namespace_alias_resolves_changed_callee -- --nocapture
    cargo test milestone33_typescript_member_call_prefers_import_context -- --nocapture
    cargo test milestone33_typescript_semantics_preserve_existing_m15_behavior -- --nocapture

Milestone 34 expected slice commands:

    cargo test milestone34_python_module_alias_resolves_changed_callee -- --nocapture
    cargo test milestone34_python_attribute_call_prefers_import_context -- --nocapture
    cargo test milestone34_python_semantics_preserve_existing_m16_behavior -- --nocapture

Milestone 35 expected slice commands:

    cargo test milestone35_diff_impact_semantic_confidence_ranking -- --nocapture
    cargo test milestone35_impact_semantic_rows_rank_deterministically -- --nocapture
    cargo test milestone35_fixture_quality_benchmark_is_stable -- --nocapture

Milestone 36 verification commands:

    cargo run -- index --repo .
    cargo run -- diff-impact --changed-file src/indexer/languages/typescript.rs --repo . --json
    cargo run -- diff-impact --changed-file src/indexer/languages/python.rs --repo . --json
    cargo run -- refs helper --repo . --code-only --exclude-tests --max-results 10 --json
    cargo test
    cargo fmt

Fixture-focused dogfood pack (run after Milestones 33, 34, and 35):

    cargo run -- index --repo tests/fixtures/phase7/semantic_precision
    cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- impact helper --repo tests/fixtures/phase7/semantic_precision --json

Expected observable milestone progression:

- Before Milestones 33/34, fixture `diff-impact` payloads contain mostly changed-symbol rows.
- After Milestones 33/34, fixture payloads include caller rows (for example `run` as
  `relationship = called_by`) tied to the changed module.
- After Milestone 35, impacted rows from semantic resolution rank ahead of fallback-only rows under
  deterministic ordering.

## Validation and Acceptance

Acceptance is behavior-first and must be observable through CLI output and tests.

After Milestone 32, new semantic-gap tests must fail in a controlled way before production edits.

After Milestone 33, changing `src/util_a.ts` in the Phase 7 TypeScript fixture must include caller
impact rows in `diff-impact` output even when `src/util_b.ts` defines the same callee symbol.

After Milestone 34, changing `src/pkg_a/util.py` in the Phase 7 Python fixture must include caller
impact rows in `diff-impact` output even when `pkg_b` defines the same callee symbol.

After Milestone 35, ranking/confidence behavior for impacted rows must be deterministic across
repeated runs, and quality-benchmark tests must pass in CI-style execution.

After Milestone 36, docs must describe shipped behavior (not planned behavior), dogfood logs must
show red/green/refactor evidence per slice, and full-suite `cargo test` must pass.

Strict TDD acceptance is mandatory: every feature slice must have recorded red, green, and refactor
transcripts.

## Idempotence and Recovery

Indexing and query behavior must remain idempotent. Re-running `index` on unchanged repositories
must preserve deterministic results and must not duplicate symbols or edges.

Phase 7 should avoid schema-version churn by default. If any schema/storage change becomes
necessary, it must be additive, migration-safe, and repeatable without destructive resets.

If semantic-resolution changes increase unexpected result volume, apply deterministic gating in
ranking/filter layers rather than introducing nondeterministic heuristics.

Corruption recovery behavior must remain intact: invalid index files must still produce an explicit
delete-and-rerun hint.

## Artifacts and Notes

Initial planning baseline transcripts:

    $ cargo run -- index --repo .
    index_path: ./.repo-scout/index.db
    schema_version: 3
    indexed_files: 0
    skipped_files: 77

    $ cargo run -- find verify_plan_for_changed_files --repo . --json | jq ...
    { "schema_version": 1, "command": "find", "total": 2, ... }

    $ cargo run -- refs verify_plan_for_changed_files --repo . --json | jq ...
    { "schema_version": 1, "command": "refs", "total": 1, ... }

Noise baseline transcript:

    $ cargo run -- refs helper --repo . --json | jq ...
    { "total": 99, "tests": 71, "docs": 3 }

Semantic-gap planning transcript (TypeScript namespace alias scenario):

    $ cargo run -- diff-impact --changed-file src/util_a.ts --repo <tmp> --json | jq ...
    { "total": 1, "impacted": [ { "symbol": "helper", "relationship": "changed_symbol" } ] }

Semantic-gap planning transcript (Python module alias scenario):

    $ cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo <tmp> --json | jq ...
    { "total": 1, "impacted": [ { "symbol": "helper", "relationship": "changed_symbol" } ] }

All Milestone 32-36 red/green/refactor transcripts must be appended here during implementation.

Milestone 32 strict TDD evidence (red stage):

    # 32A red
    cargo test milestone32_typescript_namespace_alias_call_contract -- --nocapture
    ...
    expected TypeScript namespace call to produce called_by row for src/app.ts::run

    # 32B red
    cargo test milestone32_python_module_alias_call_contract -- --nocapture
    ...
    expected Python module-alias call to produce called_by row for src/py_app.py::run_py

    # 32C red
    cargo test milestone32_schema_contracts_stay_stable -- --nocapture
    ...
    schema 3 should include semantic caller row for TypeScript fixture

Milestone 33 strict TDD evidence:

    # 33A red
    cargo test milestone33_typescript_namespace_alias_resolves_changed_callee -- --nocapture
    ...
    expected namespace alias call to resolve changed callee with called_by row

    # 33A green
    cargo test milestone33_typescript_namespace_alias_resolves_changed_callee -- --nocapture
    ...
    test milestone33_typescript_namespace_alias_resolves_changed_callee ... ok

    # 33B red
    cargo test milestone33_typescript_member_call_prefers_import_context -- --nocapture
    ...
    expected utilA.helper() to resolve to util_a helper

    # 33B green
    cargo test milestone33_typescript_member_call_prefers_import_context -- --nocapture
    ...
    test milestone33_typescript_member_call_prefers_import_context ... ok

    # 33C red check (regression guard already satisfied)
    cargo test milestone33_typescript_semantics_preserve_existing_m15_behavior -- --nocapture
    ...
    test milestone33_typescript_semantics_preserve_existing_m15_behavior ... ok

    # 33C green re-run
    cargo test milestone33_typescript_semantics_preserve_existing_m15_behavior -- --nocapture
    ...
    test milestone33_typescript_semantics_preserve_existing_m15_behavior ... ok

    # 33 refactor gate (expected to remain red until Milestone 34)
    cargo test
    ...
    failures:
      milestone32_python_module_alias_call_contract
      milestone32_schema_contracts_stay_stable

Milestone 33 behavior-check pack evidence:

    cargo run -- index --repo tests/fixtures/phase7/semantic_precision
    cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- impact helper --repo tests/fixtures/phase7/semantic_precision --json

    # observed after Milestone 33:
    # - TypeScript fixture includes `src/app.ts::run` as `called_by` for `src/util_a.ts::helper`
    # - Python fixture still returns changed-symbol-only for `src/pkg_a/util.py::helper`
    # - `impact helper` on fixture now includes TypeScript caller row

Milestone 34 strict TDD evidence:

    # 34A red
    cargo test milestone34_python_module_alias_resolves_changed_callee -- --nocapture
    ...
    expected Python module alias call to resolve changed callee

    # 34A green
    cargo test milestone34_python_module_alias_resolves_changed_callee -- --nocapture
    ...
    test milestone34_python_module_alias_resolves_changed_callee ... ok

    # 34B red
    cargo test milestone34_python_attribute_call_prefers_import_context -- --nocapture
    ...
    expected util_a.helper() to resolve to src/pkg_a/util.py::helper

    # 34B green
    cargo test milestone34_python_attribute_call_prefers_import_context -- --nocapture
    ...
    test milestone34_python_attribute_call_prefers_import_context ... ok

    # 34C red check (regression guard already satisfied)
    cargo test milestone34_python_semantics_preserve_existing_m16_behavior -- --nocapture
    ...
    test milestone34_python_semantics_preserve_existing_m16_behavior ... ok

    # 34C green re-run
    cargo test milestone34_python_semantics_preserve_existing_m16_behavior -- --nocapture
    ...
    test milestone34_python_semantics_preserve_existing_m16_behavior ... ok

    # 34 refactor gate
    cargo test
    ...
    test result: ok. (full suite)

Milestone 34 behavior-check pack evidence:

    cargo run -- index --repo tests/fixtures/phase7/semantic_precision
    cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- impact helper --repo tests/fixtures/phase7/semantic_precision --json

    # observed after Milestone 34:
    # - TypeScript fixture retains `src/app.ts::run` called_by row for `src/util_a.ts::helper`
    # - Python fixture now includes `src/py_app.py::run_py` called_by row for `src/pkg_a/util.py::helper`
    # - impact helper returns both `run` and `run_py` caller rows

Milestone 35 strict TDD evidence:

    # 35A red
    cargo test milestone35_diff_impact_semantic_confidence_ranking -- --nocapture
    ...
    semantic caller rows should receive calibrated high confidence score

    # 35A green
    cargo test milestone35_diff_impact_semantic_confidence_ranking -- --nocapture
    ...
    test milestone35_diff_impact_semantic_confidence_ranking ... ok

    # 35B red
    cargo test milestone35_impact_semantic_rows_rank_deterministically -- --nocapture
    ...
    assertion failed: results.iter().all(... score >= 0.96)

    # 35B green
    cargo test milestone35_impact_semantic_rows_rank_deterministically -- --nocapture
    ...
    test milestone35_impact_semantic_rows_rank_deterministically ... ok

    # 35C red
    cargo test milestone35_fixture_quality_benchmark_is_stable -- --nocapture
    ...
    benchmark semantic caller average score should stay in calibrated high-confidence band

    # 35C green
    cargo test milestone35_fixture_quality_benchmark_is_stable -- --nocapture
    ...
    test milestone35_fixture_quality_benchmark_is_stable ... ok

    # 35 refactor gate
    cargo test
    ...
    test result: ok. (full suite)

Milestone 35 behavior-check pack evidence:

    cargo run -- index --repo tests/fixtures/phase7/semantic_precision
    cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase7/semantic_precision --json
    cargo run -- impact helper --repo tests/fixtures/phase7/semantic_precision --json

    # observed after Milestone 35:
    # - semantic `called_by` rows in fixture now carry calibrated `score: 0.97`
    # - semantic rows still outrank fallback rows deterministically
    # - schema envelopes remain unchanged (1/2/3)

Milestone 36 verification evidence:

    cargo run -- index --repo .
    cargo run -- diff-impact --changed-file src/indexer/languages/typescript.rs --repo . --json
    cargo run -- diff-impact --changed-file src/indexer/languages/python.rs --repo . --json
    cargo run -- refs helper --repo . --code-only --exclude-tests --max-results 10 --json
    cargo test
    cargo fmt

    # observed after Milestone 36:
    # - docs reflect implemented behavior, not planned-only language.
    # - repo dogfood queries continue to return deterministic JSON envelopes and ranked rows.
    # - full suite remains green after docs refresh.
    # - rustfmt produced only deterministic formatting deltas.

## Interfaces and Dependencies

Phase 7 should continue using current dependencies (`tree-sitter`, language grammars, `rusqlite`,
`serde`, `clap`) unless a new dependency is justified in the plan and recorded in `Decision Log`.

Expected interface-level touch points:

- `src/indexer/languages/typescript.rs`
  - extend import binding metadata to carry module path context for namespace/default/named forms,
  - pass import context into member-call resolution so `utilA.helper()` can resolve to qualified
    targets.

- `src/indexer/languages/python.rs`
  - extend import binding metadata to carry module path context for `import ... as ...` and
    `from ... import ... as ...`,
  - pass alias/module context into attribute-call resolution so `util.helper()` can resolve to
    qualified targets.

- `src/indexer/mod.rs`
  - preserve resolver determinism in `resolve_symbol_id_in_tx`,
  - ensure new qualified/scoped keys continue to resolve without introducing ambiguous over-linking.

- `src/query/mod.rs`
  - add deterministic semantic-confidence calibration helpers,
  - apply calibrated ordering for impacted rows while preserving schema fields and existing
    deterministic tie-break rules.

- `tests/`
  - add milestone suites for Phase 7 (`tests/milestone32_semantic_contracts.rs`,
    `tests/milestone33_typescript_semantics.rs`, `tests/milestone34_python_semantics.rs`,
    `tests/milestone35_quality_benchmark.rs`),
  - add fixtures under `tests/fixtures/phase7/semantic_precision/`.

- Documentation targets:
  - `README.md`
  - `docs/cli-reference.md`
  - `docs/json-output.md`
  - `docs/architecture.md`
  - `docs/dogfood-log.md`
  - `docs/performance-baseline.md`

## Revision Note

2026-02-08: Created initial Phase 7 execution plan to implement cross-language semantic resolution
precision and benchmark guardrails, based on residual-work threads from Phase 2 through Phase 6 and
aligned to `agents/PLANS.md` strict TDD/living-plan requirements.

2026-02-08: Updated live plan with Milestone 32 pre-dogfood transcripts, strict red-stage
contract-test evidence, and decision-log handling for cross-milestone green/refactor closure.

2026-02-08: Updated live plan with Milestone 33 TypeScript red/green transcripts, fixture
behavior-check evidence, and decision log entries for shared fixture setup and deferred
cross-milestone refactor closure.

2026-02-08: Updated live plan with Milestone 34 Python red/green/refactor transcripts, resolved
Milestone 32 contract closure, and refreshed fixture behavior-check evidence reflecting both
TypeScript and Python caller rows.

2026-02-08: Updated live plan with Milestone 35 ranking-calibration transcripts, query-time
calibration design decisions, benchmark-guardrail outcomes, and fixture behavior-check evidence.

2026-02-08: Updated live plan with Milestone 36 documentation/evidence refresh completion,
post-refresh verification transcripts, formatter closeout notes, and outcomes.
