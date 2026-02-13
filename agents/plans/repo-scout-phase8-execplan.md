# Build `repo-scout` Phase 8 Semantic Closure and Production Hardening

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan builds on `agents/repo-scout-phase7-execplan.md` and the validation evidence captured in
`agents/phase7-validation-report.md`.

## Purpose / Big Picture

Phase 8 turns current validation findings into shipped behavior. After this phase, users should be
able to trust `diff-impact` in duplicate-name TypeScript/Python alias scenarios, run strict quality
gates (`clippy -D warnings`) cleanly, explicitly disable `diff-impact` test-target rows when they
want symbol-only impact, and inspect useful terminal output from `diff-impact` without requiring
`--json`.

User-visible outcome: semantic-impact recall improves in ambiguous cross-language cases, quality
checks become CI-safe, and day-to-day CLI usage is less noisy and easier to scan.

## Progress

- [x] (2026-02-08 02:21Z) Re-read `agents/PLANS.md`, `agents/repo-scout-phase7-execplan.md`, and
      `agents/phase7-validation-report.md` to derive Phase 8 scope from measured gaps.
- [x] (2026-02-08 02:21Z) Re-ran planning baseline dogfood commands: `cargo run -- index --repo .`,
      `cargo run -- find diff_impact_for_changed_files --repo . --json`,
      `cargo run -- refs diff_impact_for_changed_files --repo . --json`.
- [x] (2026-02-08 02:21Z) Authored this Phase 8 ExecPlan as planning-only work.
- [x] (2026-02-08 02:37Z) Completed Milestone 37 strict TDD slices for semantic-precision closure in
      TypeScript/Python alias-import call paths and added fixture
      `tests/fixtures/phase8/semantic_precision`.
- [x] (2026-02-08 02:39Z) Completed Milestone 38 strict TDD slices; strict lint gate is green on
      `cargo clippy --all-targets --all-features -- -D warnings` with full `cargo test` passing.
- [x] (2026-02-08 02:42Z) Completed Milestone 39 strict TDD slices for explicit `diff-impact`
      test-target toggles (`--exclude-tests`) with default behavior preserved.
- [x] (2026-02-08 02:44Z) Completed Milestone 40 strict TDD slices for deterministic row-level
      terminal `diff-impact` output and verified repeated command output is byte-identical.
- [x] (2026-02-08 02:46Z) Completed Milestone 41 documentation/evidence/performance refresh and
      final post-refresh dogfood plus quality gates.

## Surprises & Discoveries

- Observation: after fast-forwarding to the latest `codex/phase7-plan-and-semantic-precision`, the
  remaining semantic gap was narrower than the validation artifact described: namespace/module alias
  calls already resolved, but direct alias-import calls (`helperA()`, `helper_a()`) did not resolve
  to changed duplicate-name callees. Evidence: red tests in
  `tests/milestone37_semantic_precision.rs` failed on `run_alias_a`/`helper_a()` caller assertions
  while `run_namespace_a`/`run_module_a` were already present.

- Observation: preserving historical `impact call_helper` behavior required keeping local-import
  call edges while adding direct qualified callee edges for alias-import precision. Evidence:
  `cargo test` initially failed at `milestone16_python_references_calls_imports` when direct-edge
  logic replaced local-import edges; the test passed again after emitting both edges.

- Observation: strict clippy gates are currently red even though `cargo test` is green. Evidence:
  `cargo clippy --all-targets --all-features -- -D warnings` reports actionable lints in
  `src/indexer/languages/python.rs`, `src/indexer/languages/typescript.rs`,
  `src/indexer/languages/rust.rs`, `src/indexer/rust_ast.rs`, and `tests/common/mod.rs`.

- Observation: direct semantic fixes in language adapters triggered `clippy::too_many_arguments` for
  shared recursive helper signatures in both TypeScript and Python call collectors. Evidence:
  `cargo clippy --bin repo-scout -- -D warnings` failed on `collect_call_symbols` function arity in
  both adapter files.

- Observation: `diff-impact --include-tests` is currently a compatibility no-op and cannot disable
  test rows. Evidence: validation comparison produced identical counts with and without
  `--include-tests` (`include_tests=true`, same result length).

- Observation: terminal-mode `diff-impact` is summary-only and does not print per-result rows.
  Evidence: `src/output.rs::print_diff_impact` currently prints command metadata and count only.

- Observation: `--exclude-tests` can reduce large terminal `diff-impact` scans substantially in this
  repository when changed files have broad downstream coverage. Evidence: `src/query/mod.rs` sample
  changed from 93 rows (`include_tests=true`) to 70 rows (`include_tests=false`) in Milestone 39
  dogfood checks.

- Observation: row-oriented terminal output can stay deterministic without extra sorting logic by
  reusing already-sorted `DiffImpactMatch` query results. Evidence: repeated Milestone 40 terminal
  runs for the same command produced byte-identical output via `cmp -s`.

## Decision Log

- Decision: treat unresolved Phase 7 semantic precision as the first milestone in Phase 8 rather
  than postponing again. Rationale: layering UX and quality improvements on top of known semantic
  misses would hide core correctness debt in the primary impact workflow. Date/Author: 2026-02-08 /
  Codex

- Decision: keep all existing JSON schema versions stable (1/2/3) and implement improvements via
  additive behavior and flags. Rationale: downstream automation already depends on current
  envelopes; this phase targets quality and ergonomics without contract churn. Date/Author:
  2026-02-08 / Codex

- Decision: preserve current default `diff-impact` behavior (`include_tests=true`) while adding an
  explicit opt-out flag. Rationale: default-on test targets are existing behavior; additive opt-out
  provides operator control without breaking existing scripts. Date/Author: 2026-02-08 / Codex

- Decision: make `diff-impact` terminal output row-oriented and deterministic, mirroring JSON
  ordering. Rationale: users frequently run quick terminal checks; requiring JSON parsing for basic
  triage slows interactive debugging. Date/Author: 2026-02-08 / Codex

- Decision: include strict clippy gate recovery as a first-class milestone rather than a best-effort
  cleanup task. Rationale: a red static-analysis gate is production risk for CI and obscures future
  regressions. Date/Author: 2026-02-08 / Codex

- Decision: emit both direct qualified alias-call edges and legacy local-import call edges for
  direct alias imports. Rationale: direct edges are required for `diff-impact` caller recall at
  `distance=1`, while legacy local-import edges preserve existing `impact <import_alias>` behavior
  and avoid regressions in earlier milestones. Date/Author: 2026-02-08 / Codex

- Decision: keep recursive `collect_call_symbols` helper signatures and explicitly annotate them
  with `#[allow(clippy::too_many_arguments)]` rather than introducing high-risk refactors in the
  same milestone as strict lint-gate recovery. Rationale: this keeps semantic behavior stable while
  satisfying strict clippy gates and limiting scope creep in a hardening milestone. Date/Author:
  2026-02-08 / Codex

## Outcomes & Retrospective

Planning outcome: Phase 8 is scoped to close semantic correctness debt first, then harden quality
and operator ergonomics in the same release track.

Expected completion outcome: duplicate-name alias-import scenarios return correct caller-impact
rows, strict lint gates are green, `diff-impact` supports explicit symbol-only mode, and terminal
output is useful without `jq`.

Expected residual work after this plan: broader language-specific test-runner recommendations
(`pytest`, `jest`, etc.), optional deeper type-inference enrichment, and larger benchmark corpora
for ranking quality across non-repo fixtures.

Milestone 37 outcome (interim): `diff-impact` now returns deterministic `called_by` rows for both
namespace/module alias calls and direct alias-import calls in the new Phase 8 fixture, while
preserving legacy import-symbol impact behavior used by existing tests.

Milestone 38 outcome (interim): strict clippy quality gates are green for test and bin targets and
across all targets/features, with no behavior regressions in the full integration suite.

Milestone 39 outcome (interim): `diff-impact` now supports explicit symbol-only mode via
`--exclude-tests`, preserves default test-target inclusion, and rejects incompatible
`--include-tests --exclude-tests` combinations.

Milestone 40 outcome (interim): terminal `diff-impact` output now prints deterministic row-level
`impacted_symbol` and `test_target` lines with confidence/provenance/score fields.

Milestone 41 outcome (final): docs, dogfood evidence, and performance baseline references are
aligned to shipped Phase 8 behavior; required verification commands, strict lint gates, tests, and
format checks all pass.

## Context and Orientation

`repo-scout` command parsing is in `src/cli.rs`; command dispatch and option normalization are in
`src/main.rs`; indexing and symbol resolution are in `src/indexer/mod.rs`; language adapters are in
`src/indexer/languages/typescript.rs` and `src/indexer/languages/python.rs`; query planning and
ranking are in `src/query/mod.rs`; terminal/JSON rendering is in `src/output.rs`; integration tests
live in `tests/`.

Terms used in this plan:

- A “semantic precision closure” means converting known false-negative impact scenarios into
  deterministic passing tests and production behavior.
- An “alias-import call path” means calls such as `utilA.helper()` (TypeScript namespace import) or
  `util_a.helper()` (Python module alias import) where multiple modules define `helper`.
- A “strict clippy gate” means `cargo clippy --all-targets --all-features -- -D warnings` must pass
  with zero warnings.
- A “terminal row” for `diff-impact` means one printed output line per impacted symbol or test
  target, not just summary counters.
- A “schema-stable additive flag” means introducing new CLI behavior without changing existing JSON
  top-level contract fields or schema version numbers.

## Strict TDD Contract

Phase 8 enforces strict red-green-refactor at the feature-slice level. No production code changes
are allowed for a slice until a failing automated check for that exact slice is observed.

A feature slice in this plan is one user-visible behavior unit, such as “TypeScript namespace alias
changes include caller rows in `diff-impact`,” “`diff-impact --exclude-tests` suppresses
`test_target` rows,” or “terminal `diff-impact` prints deterministic impacted rows.”

For every slice, capture:

- red transcript: failing test/check command and expected failure reason,
- green transcript: same command passing after minimal implementation,
- refactor transcript: full-suite `cargo test` pass.

Record these transcripts in this plan and append concise evidence to `docs/dogfood-log.md`.

## Plan of Work

### Milestone 37: Semantic Precision Closure (carry-over completion)

Milestone goal: ship the missing semantic behavior that was planned in Phase 7 and confirmed missing
in validation.

Feature slice 37A adds a deterministic fixture for duplicate-name TypeScript namespace imports and
locks the failure first in `tests/milestone37_semantic_precision.rs`. The red behavior is that
changing `src/util_a.ts` returns only a `changed_symbol` row. The green behavior requires at least
one `called_by` row for the correct caller path and no cross-link to the `util_b` caller.

Feature slice 37B adds the corresponding Python module-alias fixture and failing test in the same
suite. The red behavior is that changing `src/pkg_a/util.py` misses caller impact. The green
behavior requires `called_by` rows tied to the `pkg_a` alias call path and not the `pkg_b` path.

Feature slice 37C implements minimal adapter/resolver changes in
`src/indexer/languages/typescript.rs`, `src/indexer/languages/python.rs`, and, if required for
ambiguity-safe lookup, `src/indexer/mod.rs::resolve_symbol_id_in_tx`. Keep schema unchanged and
ensure deterministic ordering remains stable across repeated runs.

### Milestone 38: Strict Clippy Gate Recovery

Milestone goal: make strict lint gates pass without semantic regressions.

Feature slice 38A starts red with a targeted lint run for test harness code
(`cargo clippy --test harness_smoke -- -D warnings`), fixes `tests/common/mod.rs`, and confirms that
helper behavior remains unchanged by running the harness and core CLI smoke tests.

Feature slice 38B starts red with binary lint checks
(`cargo clippy --bin repo-scout -- -D warnings`), then applies minimal refactors in
`src/indexer/languages/python.rs`, `src/indexer/languages/typescript.rs`,
`src/indexer/languages/rust.rs`, and `src/indexer/rust_ast.rs`. Before each refactor cluster,
add/extend integration tests to lock current behavior for the touched code path so refactors are
behavior-preserving.

Feature slice 38C runs the full strict lint gate
(`cargo clippy --all-targets --all-features -- -D warnings`) and then full `cargo test` as the
refactor gate.

### Milestone 39: Explicit `diff-impact` Test-Target Toggle

Milestone goal: allow users to suppress test-target rows when they want symbol-only impact.

Feature slice 39A adds failing integration coverage in
`tests/milestone39_diff_impact_test_toggle.rs` for `--exclude-tests` behavior. Red: test-target rows
still appear. Green: `result_kind = test_target` rows are absent and top-level `include_tests` is
`false`.

Feature slice 39B preserves default behavior and compatibility flag behavior. Add failing tests that
`diff-impact` with no toggle still includes test targets and reports `include_tests = true`, and
that explicit `--include-tests` keeps the same outcome.

Feature slice 39C wires CLI and option plumbing in `src/cli.rs`, `src/main.rs`, and
`src/query/mod.rs` with deterministic semantics. Use a conflict rule for incompatible flags (if both
toggles are present) and assert clap error messaging in tests.

### Milestone 40: Actionable `diff-impact` Terminal Rows

Milestone goal: terminal mode should provide inspectable result rows with deterministic ordering.

Feature slice 40A adds failing terminal-contract tests in
`tests/milestone40_diff_impact_terminal_output.rs` requiring impacted-symbol rows to include file,
line, symbol, relationship, confidence, and score.

Feature slice 40B extends terminal rendering for test-target rows (target path, target kind,
confidence, score) and adds failing/passing tests that verify row presence when tests are included
and absence when `--exclude-tests` is set.

Feature slice 40C ensures output determinism by asserting repeated terminal runs return identical
content for the same fixture and options.

### Milestone 41: Documentation, Dogfood Evidence, and Baseline Refresh

Milestone goal: docs and evidence must reflect shipped Phase 8 behavior, not planned behavior.

Feature slice 41A updates `README.md`, `docs/cli-reference.md`, `docs/json-output.md`, and
`docs/architecture.md` for: semantic closure outcomes, strict lint expectations, `diff-impact`
`--exclude-tests`, and row-oriented terminal output.

Feature slice 41B updates `legacy performance baseline doc (removed)` and `docs/dogfood-log.md` with Phase 8
command packs and red/green/refactor transcripts per slice.

Feature slice 41C re-runs post-refresh dogfood and quality gates to prove docs and behavior are in
sync.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/repo-scout`.

Before each milestone, run baseline dogfood:

    cargo run -- index --repo .
    cargo run -- find diff_impact_for_changed_files --repo . --json
    cargo run -- refs diff_impact_for_changed_files --repo . --json

Strict per-slice TDD loop (required order for every slice):

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure before production edits
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimum implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 37 expected slice commands:

    cargo test milestone37_typescript_namespace_alias_diff_impact_recalls_caller -- --nocapture
    cargo test milestone37_python_module_alias_diff_impact_recalls_caller -- --nocapture
    cargo test milestone37_semantic_precision_deterministic_ordering -- --nocapture

Milestone 38 expected slice commands:

    cargo clippy --test harness_smoke -- -D warnings
    cargo clippy --bin repo-scout -- -D warnings
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

Milestone 39 expected slice commands:

    cargo test milestone39_diff_impact_exclude_tests_omits_test_targets -- --nocapture
    cargo test milestone39_diff_impact_default_and_include_tests_keep_test_targets -- --nocapture
    cargo test milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit -- --nocapture

Milestone 40 expected slice commands:

    cargo test milestone40_diff_impact_terminal_lists_impacted_symbol_rows -- --nocapture
    cargo test milestone40_diff_impact_terminal_lists_test_target_rows_conditionally -- --nocapture
    cargo test milestone40_diff_impact_terminal_output_is_deterministic -- --nocapture

Milestone 41 verification commands:

    cargo run -- index --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json
    cargo run -- explain diff_impact_for_changed_files --repo . --json
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo fmt

Fixture-focused semantic dogfood pack (run after Milestone 37 and again after Milestone 41):

    cargo run -- index --repo tests/fixtures/phase8/semantic_precision
    cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json
    cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json
    cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json

Expected observable progression:

- Before Milestone 37 green, fixture `diff-impact` outputs contain only changed-symbol rows for the
  changed module files.
- After Milestone 37 green, fixture outputs include `called_by` caller rows for the correct alias
  import path and avoid duplicate-name cross-linking.
- After Milestone 39 green, `--exclude-tests` removes test-target rows and flips top-level
  `include_tests` to `false`.
- After Milestone 40 green, terminal `diff-impact` output prints deterministic, row-level details.
- After Milestone 41, docs and logs reflect current behavior and all quality gates are green.

## Validation and Acceptance

Acceptance is behavior-first and must be observable through command output and tests.

After Milestone 37, both TypeScript and Python alias-import duplicate-name fixtures must return
caller-impact rows in `diff-impact --json` for changed files, and repeated runs must remain
byte-stable.

After Milestone 38, `cargo clippy --all-targets --all-features -- -D warnings` must pass, and full
`cargo test` must still pass.

After Milestone 39, `diff-impact` default behavior remains unchanged (includes test targets), while
`--exclude-tests` produces symbol-only results with `include_tests = false` in schema 3 output.

After Milestone 40, terminal `diff-impact` must print per-result lines for both impacted symbols and
conditional test targets with deterministic ordering.

After Milestone 41, documentation and dogfood logs must describe shipped Phase 8 behavior, and the
full post-refresh command pack must pass.

Strict TDD acceptance is mandatory: every feature slice must include recorded red, green, and
refactor transcripts.

## Idempotence and Recovery

Indexing and query behavior must remain idempotent. Re-running `index` on unchanged repositories
must not duplicate symbols or edges.

This phase must avoid schema version changes unless absolutely necessary. If a schema change is
required, it must be additive, migration-safe, and fully backward-compatible with existing query
envelopes.

If semantic-resolution changes produce unexpected result growth, tighten deterministic ranking and
filter controls rather than introducing nondeterministic heuristics.

If any run fails with index corruption signatures, keep existing recovery behavior intact: surface a
clear delete-and-rerun hint with the exact index path.

## Artifacts and Notes

Phase 8 planning baseline transcript:

    $ cargo run -- index --repo .
    index_path: ./.repo-scout/index.db
    schema_version: 3
    indexed_files: 1
    skipped_files: 78

    $ cargo run -- find diff_impact_for_changed_files --repo . --json | jq ...
    { "schema_version": 1, "command": "find", "results": 1 }

    $ cargo run -- refs diff_impact_for_changed_files --repo . --json | jq ...
    { "schema_version": 1, "command": "refs", "results": 1 }

Phase 8 starting-gap evidence (from validation artifact):

    $ cargo clippy --all-targets --all-features -- -D warnings
    error: ... src/indexer/languages/python.rs ...
    error: ... src/indexer/languages/typescript.rs ...
    error: ... src/indexer/rust_ast.rs ...
    error: ... tests/common/mod.rs ...

    $ cargo run -- diff-impact --changed-file src/util_a.ts --repo <tmp> --json | jq ...
    { "results": [ { "relationship": "changed_symbol" } ] }

    $ cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo <tmp> --json | jq ...
    { "results": [ { "relationship": "changed_symbol" } ] }

All Milestone 37-41 red/green/refactor transcripts must be appended here during implementation.

Milestone 37 strict TDD evidence:

    # Slice 37A red
    $ cargo test milestone37_typescript_namespace_alias_diff_impact_recalls_caller -- --nocapture
    FAILED: expected helperA() alias-import call to resolve directly to src/util_a.ts::helper

    # Slice 37A green
    $ cargo test milestone37_typescript_namespace_alias_diff_impact_recalls_caller -- --nocapture
    ok

    # Slice 37B red
    $ cargo test milestone37_python_module_alias_diff_impact_recalls_caller -- --nocapture
    FAILED: expected helper_a() from-import alias call to resolve changed callee

    # Slice 37B green
    $ cargo test milestone37_python_module_alias_diff_impact_recalls_caller -- --nocapture
    ok

    # Slice 37C determinism
    $ cargo test milestone37_semantic_precision_deterministic_ordering -- --nocapture
    ok

    # Slice refactor gate
    $ cargo test
    ok (full suite)

Milestone 37 fixture dogfood evidence:

    $ cargo run -- index --repo tests/fixtures/phase8/semantic_precision
    indexed_files: 7

    $ cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json
    includes: run_namespace_a called_by distance=1, run_alias_a called_by distance=1
    excludes: run_namespace_b/run_alias_b called_by for util_a change

    $ cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json
    includes: run_module_a called_by distance=1, run_alias_a called_by distance=1
    excludes: run_module_b/run_alias_b called_by for pkg_a change

    $ cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json
    includes TypeScript and Python caller rows in deterministic order

Milestone 38 strict lint evidence:

    # Slice 38A red
    $ cargo clippy --test harness_smoke -- -D warnings
    error: collapsible_if in tests/common/mod.rs

    # Slice 38A green
    $ cargo clippy --test harness_smoke -- -D warnings
    Finished ... target(s)

    # Slice 38B red
    $ cargo clippy --bin repo-scout -- -D warnings
    errors: question_mark, too_many_arguments, double_ended_iterator_last, collapsible_if

    # Slice 38B green
    $ cargo clippy --bin repo-scout -- -D warnings
    Finished ... target(s)

    # Slice 38C gate
    $ cargo clippy --all-targets --all-features -- -D warnings
    Finished ... target(s)

    # Slice refactor gate
    $ cargo test
    ok (full suite)

Milestone 39 strict TDD evidence:

    # Slice 39A (red observed before toggle plumbing)
    $ cargo test milestone39_diff_impact_exclude_tests_omits_test_targets -- --nocapture
    FAILED (pre-fix): expected test_target rows to be absent and include_tests=false

    # Slice 39A green
    $ cargo test milestone39_diff_impact_exclude_tests_omits_test_targets -- --nocapture
    ok

    # Slice 39B (red observed before default/compat assertions)
    $ cargo test milestone39_diff_impact_default_and_include_tests_keep_test_targets -- --nocapture
    FAILED (pre-fix): expected default and explicit include behavior to match with test_target rows

    # Slice 39B green
    $ cargo test milestone39_diff_impact_default_and_include_tests_keep_test_targets -- --nocapture
    ok

    # Slice 39C (red observed before clap conflicts)
    $ cargo test milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit -- --nocapture
    FAILED (pre-fix): expected clap conflict error for --include-tests + --exclude-tests

    # Slice 39C green
    $ cargo test milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit -- --nocapture
    ok

    # Slice refactor gate
    $ cargo test
    ok (full suite)

Milestone 39 toggle behavior evidence:

    $ cargo run -- index --repo .
    indexed_files: 0
    skipped_files: 101

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
    include_tests=true, results=93, test_targets=23

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
    include_tests=false, results=70, test_targets=0

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json
    include_tests=true, results=93, test_targets=23

Milestone 40 strict TDD evidence:

    # Slice 40A red
    $ cargo test milestone40_diff_impact_terminal_lists_impacted_symbol_rows -- --nocapture
    FAILED: terminal output should include a changed_entry impacted_symbol row

    # Slice 40A green
    $ cargo test milestone40_diff_impact_terminal_lists_impacted_symbol_rows -- --nocapture
    ok

    # Slice 40B red
    $ cargo test milestone40_diff_impact_terminal_lists_test_target_rows_conditionally -- --nocapture
    FAILED: terminal output should include test_target rows by default

    # Slice 40B green
    $ cargo test milestone40_diff_impact_terminal_lists_test_target_rows_conditionally -- --nocapture
    ok

    # Slice 40C red
    $ cargo test milestone40_diff_impact_terminal_output_is_deterministic -- --nocapture
    FAILED: determinism check requires row-oriented impacted_symbol output

    # Slice 40C green
    $ cargo test milestone40_diff_impact_terminal_output_is_deterministic -- --nocapture
    ok

    # Slice refactor gate
    $ cargo test
    ok (full suite)

Milestone 40 terminal-output evidence:

    $ cargo run -- index --repo .
    indexed_files: 2
    skipped_files: 100

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    includes row-level lines:
      impacted_symbol src/query/mod.rs:... relationship=changed_symbol ... confidence=... score=...
      test_target tests/... confidence=... score=...

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    identical to previous run (verified by cmp -s)

Milestone 41 verification evidence:

    $ cargo run -- index --repo .
    indexed_files: 7
    skipped_files: 95

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    row-oriented terminal output present; include_tests=true

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
    include_tests=false; test_target rows absent

    $ cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json
    include_tests=true; test_target rows present

    $ cargo run -- explain diff_impact_for_changed_files --repo . --json
    schema_version=3, command=explain, deterministic payload

    $ cargo run -- index --repo tests/fixtures/phase8/semantic_precision
    indexed_files: 0
    skipped_files: 7

    $ cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json
    includes run_namespace_a/run_alias_a called_by; excludes *_b callers

    $ cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json
    includes run_module_a/run_alias_a called_by; excludes *_b callers

    $ cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json
    includes deterministic TypeScript/Python caller rows

    $ cargo clippy --all-targets --all-features -- -D warnings
    Finished ... target(s)

    $ cargo test
    ok (full suite)

    $ cargo fmt
    formatting clean

## Interfaces and Dependencies

Phase 8 should continue using existing dependencies (`tree-sitter` language grammars, `rusqlite`,
`serde`, `clap`) unless a new dependency is justified in `Decision Log` with explicit acceptance
criteria.

Expected interface-level touch points:

- `src/indexer/languages/typescript.rs`
  - enrich import binding metadata and member-call resolution so namespace/default/named alias calls
    can emit unambiguous `SymbolKey` targets for duplicate-name cases.

- `src/indexer/languages/python.rs`
  - enrich import binding metadata and attribute-call resolution so module alias paths map to the
    correct qualified callee.

- `src/indexer/mod.rs`
  - preserve deterministic `resolve_symbol_id_in_tx` ambiguity handling while supporting improved
    qualified/scoped lookup precision.

- `src/cli.rs` and `src/main.rs`
  - add and normalize explicit `diff-impact` test-row opt-out behavior (`--exclude-tests`) while
    preserving default compatibility behavior.

- `src/query/mod.rs`
  - ensure `DiffImpactOptions.include_tests` and traversal/ranking behavior remain deterministic
    with new toggle semantics.

- `src/output.rs`
  - render deterministic per-result terminal rows for `diff-impact` while preserving schema 3 JSON
    output unchanged.

- `tests/`
  - add milestone suites for Phase 8 (`tests/milestone37_semantic_precision.rs`,
    `tests/milestone39_diff_impact_test_toggle.rs`,
    `tests/milestone40_diff_impact_terminal_output.rs`),
  - add fixtures under `tests/fixtures/phase8/semantic_precision/`.

- Documentation targets:
  - `README.md`
  - `docs/cli-reference.md`
  - `docs/json-output.md`
  - `docs/architecture.md`
  - `docs/dogfood-log.md`
  - `legacy performance baseline doc (removed)`

## Revision Note

2026-02-08: Created initial Phase 8 execution plan to close Phase 7 semantic carry-over gaps and
ship production hardening based on full-app validation evidence in
`agents/phase7-validation-report.md`, aligned to `agents/PLANS.md` strict TDD and living-plan
requirements.

2026-02-08: Updated after implementation completion for Milestones 37-41 with strict TDD evidence,
dogfood transcripts, documentation refresh scope, and final verification command outcomes.
