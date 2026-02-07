# Build `repo-scout` Phase 4 Precision, Disambiguation, and Noise Controls

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance
with that file.

This plan builds on `agents/repo-scout-phase3-execplan.md`, which delivered schema v3 contracts,
`diff-impact`, `explain`, and Rust/TypeScript/Python adapter extraction.

## Purpose / Big Picture

Phase 4 focuses on result quality, not command count. After this change, `repo-scout` should stop
creating graph edges from ambiguous symbol-name collisions, `diff-impact` should prioritize
high-signal changed symbols, and `find`/`refs` should provide deterministic scope controls that
reduce documentation and test-noise fallback rows when requested.

User-visible outcome: code-navigation results stay deterministic but become more trustworthy for
editing loops, especially in repositories where repeated symbol names exist across files.

## Progress

- [x] (2026-02-07 01:12Z) Re-read `agents/PLANS.md`, `agents/repo-scout-phase3-execplan.md`,
      `docs/architecture.md`, `docs/cli-reference.md`, and `docs/json-output.md` to align Phase 4
      scope with current contracts.
- [x] (2026-02-07 01:15Z) Captured baseline dogfood output for current behavior:
      `cargo run -- index --repo .`,
      `cargo run -- refs impact_matches --repo . --json`,
      `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json`.
- [x] (2026-02-07 01:16Z) Quantified `diff-impact` baseline noise using
      `cargo run --quiet -- diff-impact ... | jq`: `68` total results, `62` impacted symbols,
      `5` distance-0 imports, `6` test targets.
- [x] (2026-02-07 01:18Z) Built a temporary duplicate-symbol fixture repo and confirmed ambiguous
      call resolution drops one true edge (`entry` only linked to `src/a.rs::run`, missing
      `src/b.rs::run`).
- [x] (2026-02-07 01:24Z) Authored this Phase 4 ExecPlan as planning-only work (no production code
      changes outside plan docs).
- [x] (2026-02-07 15:47Z) Created branch stack per workflow:
      `codex/phase4-plan-and-precision-contracts` from `main`, then
      `codex/phase4-implementation` for execution work.
- [x] (2026-02-07 15:47Z) Ran required pre-milestone dogfood baseline:
      `cargo run -- index --repo .`,
      `cargo run -- find verify_plan_for_changed_files --repo . --json`,
      `cargo run -- refs verify_plan_for_changed_files --repo . --json`.
- [x] (2026-02-07 15:47Z) Milestone 18 complete: added
      `tests/fixtures/phase4/ambiguity/{disambiguated,ambiguous}/` and
      `tests/milestone18_precision_graph.rs` with strict red evidence for slices 18A/18B/18C.
- [x] (2026-02-07 15:55Z) Milestone 19 complete: shipped `SymbolKey` disambiguator fields,
      resolver hardening in `src/indexer/mod.rs`, adapter-emitted scoped keys across
      Rust/TypeScript/Python, and green/refactor validation for all Milestone 18 contracts plus
      full suite.
- [x] (2026-02-07 15:58Z) Milestone 20 complete: added
      `tests/milestone20_diff_impact_precision.rs`, implemented `--include-imports` and
      `--changed-line` parsing/normalization, and updated `diff-impact` seed selection logic with
      deterministic range filtering and actionable malformed-spec errors.
- [ ] Milestone 21 complete: `find`/`refs` scope controls (`--code-only`, `--exclude-tests`) plus
      docs, dogfood transcript, and full validation pass.

## Surprises & Discoveries

- Observation: `diff-impact --changed-file src/query/mod.rs --repo . --json` currently emits a very
  broad seed set that includes imports and many helper functions by default.
  Evidence: baseline JSON summary reported `68` total rows with `5` distance-0 `kind=import`
  symbols and `6` test-target rows.

- Observation: current edge resolution resolves by plain symbol text, so repeated names across files
  can be over-linked or under-linked.
  Evidence: temporary fixture with `src/a.rs::run` and `src/b.rs::run` produced `entry -> a::run`
  but missed `entry -> b::run`; SQLite query showed only one `calls` target for `entry`.

- Observation: `diff-impact` currently seeds every symbol defined in each changed file before
  traversal, which amplifies low-value matches when files contain many declarations.
  Evidence: in repository self-dogfood, changing only `src/query/mod.rs` emitted dozens of
  `distance = 0` rows before any neighbor ranking was applied.

- Observation: Rust qualified calls like `a::run()` and `b::run()` currently collapse onto one
  duplicate target because resolver fallback remains symbol-text-only.
  Evidence: `milestone18_disambiguates_duplicate_rust_call_targets` red run reported
  `left: [\"src/a.rs\"]` vs `right: [\"src/a.rs\", \"src/b.rs\"]`.

- Observation: integration tests were selecting an external `codex-5-3` binary before
  `repo-scout`, masking local implementation behavior.
  Evidence: contract test remained red while manual local `cargo run -- index` query showed both
  expected disambiguated edges; updating `tests/common/mod.rs` candidate order aligned outcomes.

- Observation: strict unique-global resolver fallback broke prior TypeScript `implements` edge
  behavior when both interface and import rows shared the same symbol.
  Evidence: `milestone15_typescript_edges_and_queries` failed until adapter import-path hints
  mapped `Runner implements Contract` to `src/contracts.ts::Contract`.

- Observation: before Milestone 20 implementation, new `diff-impact` controls were rejected at CLI
  parse time (`--include-imports`, `--changed-line`) and default seeds still included imports.
  Evidence: milestone20 red tests showed both unknown-argument errors and failing import-seed
  assertions.

## Decision Log

- Decision: prioritize precision over recall when symbol resolution is ambiguous and no
  deterministic disambiguator is available.
  Rationale: automation loops are harmed more by incorrect edges than by missing low-confidence
  edges.
  Date/Author: 2026-02-07 / Codex

- Decision: sequence Phase 4 as contracts-first for precision failures, then resolver plumbing,
  then query-surface controls.
  Rationale: explicit red tests prevent accidental fallback to name-only edge linkage during
  implementation.
  Date/Author: 2026-02-07 / Codex

- Decision: keep schema 1/2/3 JSON envelopes intact and focus Phase 4 on deterministic ranking and
  option-driven filtering behavior.
  Rationale: avoid schema churn for existing automation consumers while improving practical signal.
  Date/Author: 2026-02-07 / Codex

- Decision: make import-seed behavior explicit in `diff-impact` with an opt-in include flag, while
  defaulting to higher-signal changed symbols.
  Rationale: import rows are useful for some workflows but currently dominate many changed-file
  outputs.
  Date/Author: 2026-02-07 / Codex

- Decision: lock Milestone 18 contracts with two fixture variants (`disambiguated` and
  `ambiguous`) under `tests/fixtures/phase4/ambiguity/` to isolate precision and fail-safe behavior.
  Rationale: separate fixtures avoid conflating qualified and ambiguous call behavior in one test
  corpus and keep red failures actionable.
  Date/Author: 2026-02-07 / Codex

- Decision: prioritize `repo-scout` binary discovery in `tests/common/mod.rs`.
  Rationale: integration tests must exercise local repository code, not external tool binaries.
  Date/Author: 2026-02-07 / Codex

- Decision: emit deterministic import-path hints from the TypeScript adapter for imported symbols.
  Rationale: resolver ambiguity safeguards should not regress valid cross-file `implements` and
  `imports` edges when import source paths are syntactically known.
  Date/Author: 2026-02-07 / Codex

- Decision: keep schema v3 payload shape backward-compatible while adding Milestone 20 controls.
  Rationale: `--include-imports` and `--changed-line` alter selection semantics but do not require
  mandatory new JSON envelope fields for automation consumers.
  Date/Author: 2026-02-07 / Codex

## Outcomes & Retrospective

Planning outcome at this stage: Phase 4 scope is constrained to precision and signal quality on the
existing command surface. The plan avoids adding new command families and instead improves edge
correctness and result focus in `diff-impact`, `find`, and `refs`.

Expected Phase 4 completion outcome: duplicate-name graph ambiguity no longer creates arbitrary
edges, changed-file impact starts from higher-signal seeds by default, and users can request
code-focused `find`/`refs` output when fallback noise is undesirable.

Expected residual work after this plan: deeper type-aware semantic resolution (for example,
module-aware import mapping and full language type inference), and longitudinal quality metrics
across larger real-world repositories.

Milestone 18 retrospective (2026-02-07): precision defect expectations are now locked as failing
integration contracts before any production refactor. This preserved strict TDD ordering and
provides deterministic pass/fail gates for resolver hardening in Milestone 19.

Milestone 19 retrospective (2026-02-07): symbol-key-aware resolution now fails safe on ambiguity
while preserving existing graph behavior through adapter hints. Duplicate-call disambiguation and
ambiguous-call suppression are now enforced by passing integration tests and full-suite validation.

Milestone 20 retrospective (2026-02-07): `diff-impact` now defaults to higher-signal changed
symbols by excluding import seeds, offers explicit opt-in restoration via `--include-imports`, and
supports deterministic line-range seed scoping with clear parse errors for malformed input.

## Context and Orientation

`repo-scout` command parsing is in `src/cli.rs`, command dispatch in `src/main.rs`, indexing and
edge persistence in `src/indexer/mod.rs`, language adapters in `src/indexer/languages/`, query
behavior in `src/query/mod.rs`, and user/JSON output in `src/output.rs`. Integration tests are in
`tests/` and remain milestone-oriented.

Term definitions used in this plan:

- A "symbol identity" means the stable metadata used to refer to a definition row in `symbols_v2`.
  Today the resolver can fall back to plain symbol text; Phase 4 hardens this behavior.
- An "edge resolution" step means turning adapter-emitted edge endpoints into concrete
  `symbol_id -> symbol_id` rows in `symbol_edges_v2`.
- A "changed-symbol seed set" means the initial `distance = 0` symbols that `diff-impact` starts
  from before walking one-hop inbound neighbors.
- "Noise controls" means deterministic command-line options that reduce fallback matches from files
  outside intended scope (for example docs or tests).

Current known hot spots:

- `src/indexer/languages/mod.rs`: `SymbolKey` currently stores only `symbol`.
- `src/indexer/mod.rs`: `resolve_symbol_id_in_tx` currently resolves edge endpoints via
  `WHERE symbol = ?`.
- `src/query/mod.rs`: `diff_impact_for_changed_files` currently seeds all symbols in changed files.
- `src/cli.rs` and `src/main.rs`: `find`/`refs` currently share generic args with no scope controls.

## Strict TDD Contract

Phase 4 enforces strict per-slice red-green-refactor. No production code is allowed before a
failing test exists for that exact feature slice.

A "feature slice" in this plan is one user-visible behavior change, such as "duplicate call targets
are disambiguated" or "`diff-impact` excludes imports by default."

For every slice, record:

- red transcript: failing integration test command,
- green transcript: same test command passing after minimal code change,
- refactor transcript: full-suite `cargo test` passing.

Evidence should be appended to the relevant milestone notes in this file and in
`docs/dogfood-log.md`.

## Plan of Work

### Milestone 18: Lock precision contracts with failing integration tests

Milestone goal: encode current precision defects as deterministic red tests before implementation.
At milestone end, tests exist that fail on current behavior and define expected edge correctness.

Feature slice 18A defines duplicate-call disambiguation behavior. Add a new fixture repository under
`tests/fixtures/phase4/ambiguity/` with duplicate function names in separate modules. Add test
`milestone18_disambiguates_duplicate_rust_call_targets` in
`tests/milestone18_precision_graph.rs` asserting `entry` connects to both module-qualified `run`
definitions.

Feature slice 18B defines changed-file impact expectations on duplicate symbols. Add test
`milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target` asserting
`diff-impact --changed-file src/b.rs` includes `entry` as an inbound caller.

Feature slice 18C defines ambiguity safety behavior. Add test
`milestone18_ambiguous_unqualified_call_does_not_cross_link` asserting that when two candidate
targets remain ambiguous and no deterministic key is available, the call edge is skipped rather than
linked arbitrarily.

### Milestone 19: Implement symbol-key disambiguation and resolver hardening

Milestone goal: replace name-only edge endpoint resolution with deterministic symbol identity
hints across adapters and indexer resolver paths.

Feature slice 19A updates adapter contracts. In `src/indexer/languages/mod.rs`, expand `SymbolKey`
to include optional disambiguators (`qualified_symbol`, `file_path`, `language`) while preserving
`symbol` as fallback text.

Feature slice 19B updates edge persistence in `src/indexer/mod.rs`:

- carry full `SymbolKey` through `pending_edges` and `deferred_edges`,
- replace `resolve_symbol_id_in_tx(tx, symbol: &str)` with
  `resolve_symbol_id_in_tx(tx, key: &SymbolKey)`,
- resolve in deterministic order:
  1. exact `qualified_symbol`,
  2. exact `(file_path, symbol)` non-import preference,
  3. unique global `symbol` match when unambiguous,
  4. otherwise unresolved (`None`).

Feature slice 19C updates adapters (`rust`, `typescript`, `python`) so emitted edges provide
disambiguation hints where known (at minimum caller file path + language and deterministic qualified
symbols for local definitions). Keep deterministic sorting and dedupe semantics unchanged.

### Milestone 20: Improve `diff-impact` seed quality with explicit controls

Milestone goal: make changed-file impact focus predictable and high-signal by default.

Feature slice 20A adds CLI and query options:

- `diff-impact` gains `--include-imports` (default false),
- `diff-impact` gains repeatable `--changed-line <path:start[:end]>`.

In `src/query/mod.rs`, default `distance = 0` seed collection excludes `kind = import` unless
`include_imports` is true.

Feature slice 20B adds changed-line scoping. When any `--changed-line` values are provided, seed
symbols must overlap those line ranges in the specified file; changed files without line ranges keep
file-level behavior.

Feature slice 20C keeps deterministic behavior explicit:

- changed-line specs are normalized and deduped,
- malformed specs return actionable errors,
- result ordering contract remains stable and deterministic.

### Milestone 21: Add deterministic noise controls to `find` and `refs`

Milestone goal: let users request code-focused lookup output without changing default behavior for
existing consumers.

Feature slice 21A adds command-specific args in `src/cli.rs`:

- `find` and `refs` gain `--code-only`,
- `find` and `refs` gain `--exclude-tests`.

Feature slice 21B updates query logic in `src/query/mod.rs` so scope flags affect text fallback
queries only:

- `--code-only` restricts fallback to recognized code extensions (`.rs`, `.ts`, `.tsx`, `.py`),
- `--exclude-tests` excludes paths matching existing test-target heuristics.

AST exact matches remain highest priority and are still returned when present.

Feature slice 21C finalizes docs and dogfood evidence updates in:

- `README.md`,
- `docs/cli-reference.md`,
- `docs/json-output.md`,
- `docs/architecture.md`,
- `docs/dogfood-log.md`,
- `docs/performance-baseline.md`.

## Concrete Steps

Run all commands from repository root:

    cd /Users/robertguss/Projects/experiments/repo-scout

Dogfood baseline before each milestone:

    cargo run -- index --repo .
    cargo run -- find verify_plan_for_changed_files --repo . --json
    cargo run -- refs verify_plan_for_changed_files --repo . --json

Milestone 18 command set:

    cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture
    cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture
    cargo test

    cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture
    cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture
    cargo test

    cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture
    cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture
    cargo test

Milestone 19 command set:

    cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture
    cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture
    cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture
    cargo test

Milestone 20 command set:

    cargo test milestone20_diff_impact_excludes_import_seeds_by_default -- --nocapture
    cargo test milestone20_diff_impact_excludes_import_seeds_by_default -- --nocapture
    cargo test

    cargo test milestone20_diff_impact_include_imports_restores_import_rows -- --nocapture
    cargo test milestone20_diff_impact_include_imports_restores_import_rows -- --nocapture
    cargo test

    cargo test milestone20_diff_impact_changed_line_limits_seed_symbols -- --nocapture
    cargo test milestone20_diff_impact_changed_line_limits_seed_symbols -- --nocapture
    cargo test

Milestone 21 command set:

    cargo test milestone21_refs_code_only_omits_docs_text_fallback -- --nocapture
    cargo test milestone21_refs_code_only_omits_docs_text_fallback -- --nocapture
    cargo test

    cargo test milestone21_refs_exclude_tests_omits_test_paths -- --nocapture
    cargo test milestone21_refs_exclude_tests_omits_test_paths -- --nocapture
    cargo test

    cargo test milestone21_find_scope_flags_keep_ast_priority_and_determinism -- --nocapture
    cargo test milestone21_find_scope_flags_keep_ast_priority_and_determinism -- --nocapture
    cargo test

Post-milestone dogfood checks:

    cargo run -- index --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo .
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --changed-line src/query/mod.rs:132:220 --repo .
    cargo run -- refs verify_plan_for_changed_files --repo .
    cargo run -- refs verify_plan_for_changed_files --repo . --code-only --exclude-tests
    cargo test

Before finalizing:

    cargo fmt
    cargo test

## Validation and Acceptance

Acceptance is behavior-first and repository-observable.

For resolver precision, the duplicate-symbol fixture tests must prove:

- `entry` links to both module-qualified run definitions when syntax is disambiguated,
- ambiguous unqualified symbols are not cross-linked arbitrarily.

For `diff-impact`, default behavior must:

- exclude `kind=import` from `distance=0` changed-symbol rows,
- include imports only when `--include-imports` is present,
- honor `--changed-line` ranges deterministically.

For `find`/`refs`, scope controls must:

- keep AST-priority ranking unchanged,
- reduce fallback rows from docs/tests when flags are enabled,
- preserve deterministic ordering.

Final acceptance requires:

- strict per-slice red-green-refactor evidence,
- full suite pass,
- dogfood transcript entries for each milestone,
- updated user and architecture docs reflecting shipped behavior.

## Idempotence and Recovery

Indexing and query operations must remain idempotent. Re-indexing unchanged repositories must not
duplicate symbols or edges, and running the same command twice against unchanged index state must
produce deterministic row ordering.

If resolver hardening leaves an edge unresolved, behavior must fail safe by omitting that edge
rather than linking to an arbitrary symbol.

If parsing of `--changed-line` fails, command output must return an actionable error that includes
the malformed token and expected format (`path:start[:end]`) without mutating index state.

## Artifacts and Notes

Baseline evidence recorded during planning:

    cargo run --quiet -- diff-impact --changed-file src/query/mod.rs --repo . --json | jq \
      '{total_results:(.results|length), impacted_symbols:([.results[]|select(.result_kind=="impacted_symbol")]|length), test_targets:([.results[]|select(.result_kind=="test_target")]|length), distance0_imports:([.results[]|select(.result_kind=="impacted_symbol" and .distance==0 and .kind=="import")]|length), distance1_results:([.results[]|select(.result_kind=="impacted_symbol" and .distance==1)]|length)}'

    {
      "total_results": 68,
      "impacted_symbols": 62,
      "test_targets": 6,
      "distance0_imports": 5,
      "distance1_results": 19
    }

Temporary duplicate-symbol fixture evidence:

    src/a.rs|run|function|src/a.rs|helper|function|calls
    src/lib.rs|entry|function|src/a.rs|run|function|calls
    src/lib.rs|entry|function|src/lib.rs|a|module|calls
    src/lib.rs|entry|function|src/lib.rs|b|module|calls

The missing `src/lib.rs|entry|...|src/b.rs|run|...|calls` row defines the precision defect locked
by Milestone 18 red tests.

Milestone 18 strict-TDD red evidence:

    cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture
    # FAILED: left ["src/a.rs"] right ["src/a.rs", "src/b.rs"]

    cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture
    # FAILED: missing impacted_symbol entry caller for src/lib.rs::entry when changing src/b.rs

    cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture
    # FAILED: ambiguous_run_targets left 1 right 0

Milestone 19 strict-TDD green/refactor evidence:

    cargo test milestone18_disambiguates_duplicate_rust_call_targets -- --nocapture
    # PASS

    cargo test milestone18_diff_impact_includes_true_callers_for_changed_duplicate_target -- --nocapture
    # PASS

    cargo test milestone18_ambiguous_unqualified_call_does_not_cross_link -- --nocapture
    # PASS

    cargo test
    # PASS (full integration suite green after resolver + adapter refactor)

Milestone 20 strict-TDD red evidence:

    cargo test milestone20_diff_impact_excludes_import_seeds_by_default -- --nocapture
    # FAILED: distance=0 import seeds still present by default

    cargo test milestone20_diff_impact_include_imports_restores_import_rows -- --nocapture
    # FAILED: CLI rejected --include-imports before implementation

    cargo test milestone20_diff_impact_changed_line_limits_seed_symbols -- --nocapture
    # FAILED: CLI rejected --changed-line before implementation

Milestone 20 strict-TDD green/refactor evidence:

    cargo test milestone20_diff_impact_excludes_import_seeds_by_default -- --nocapture
    # PASS

    cargo test milestone20_diff_impact_include_imports_restores_import_rows -- --nocapture
    # PASS

    cargo test milestone20_diff_impact_changed_line_limits_seed_symbols -- --nocapture
    # PASS

    cargo test
    # PASS (full integration suite green after diff-impact option refactor)

## Interfaces and Dependencies

Phase 4 does not require new external crates by default. Continue using current dependencies
(`tree-sitter`, language grammars, `rusqlite`, `serde`, `clap`) and justify any additions in the
Decision Log.

In `src/indexer/languages/mod.rs`, revise `SymbolKey` contract:

    pub struct SymbolKey {
        pub symbol: String,
        pub qualified_symbol: Option<String>,
        pub file_path: Option<String>,
        pub language: Option<String>,
    }

In `src/indexer/mod.rs`, update resolver signature:

    fn resolve_symbol_id_in_tx(
        tx: &rusqlite::Transaction<'_>,
        key: &languages::SymbolKey,
    ) -> anyhow::Result<Option<i64>>;

Resolver ordering contract:

1. match `qualified_symbol` exactly when present.
2. else match `(file_path, symbol)`; if multiple rows match, prefer non-`import` kinds over `import` kinds and then apply stable ordering.
3. else match unique global `symbol`.
4. else unresolved (`None`).

In `src/cli.rs`, add Phase 4 args:

    pub struct DiffImpactArgs {
        #[arg(long = "changed-file", required = true)]
        pub changed_files: Vec<String>,
        #[arg(long = "changed-line")]
        pub changed_lines: Vec<String>,
        #[arg(long, default_value_t = 2)]
        pub max_distance: u32,
        #[arg(long, default_value_t = true)]
        pub include_tests: bool,
        #[arg(long, default_value_t = false)]
        pub include_imports: bool,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
    }

    pub struct FindArgs {
        pub symbol: String,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
        #[arg(long, default_value_t = false)]
        pub code_only: bool,
        #[arg(long, default_value_t = false)]
        pub exclude_tests: bool,
    }

    pub struct RefsArgs {
        pub symbol: String,
        #[arg(long)]
        pub repo: PathBuf,
        #[arg(long)]
        pub json: bool,
        #[arg(long, default_value_t = false)]
        pub code_only: bool,
        #[arg(long, default_value_t = false)]
        pub exclude_tests: bool,
    }

In `src/query/mod.rs`, add:

    pub struct QueryScope {
        pub code_only: bool,
        pub exclude_tests: bool,
    }

    pub struct ChangedLineRange {
        pub file_path: String,
        pub start_line: u32,
        pub end_line: u32,
    }

    pub struct DiffImpactOptions {
        pub max_distance: u32,
        pub include_tests: bool,
        pub include_imports: bool,
        pub changed_lines: Vec<ChangedLineRange>,
    }

    pub fn find_matches_scoped(
        db_path: &Path,
        symbol: &str,
        scope: &QueryScope,
    ) -> anyhow::Result<Vec<QueryMatch>>;

    pub fn refs_matches_scoped(
        db_path: &Path,
        symbol: &str,
        scope: &QueryScope,
    ) -> anyhow::Result<Vec<QueryMatch>>;

    pub fn diff_impact_for_changed_files(
        db_path: &Path,
        changed_files: &[String],
        options: &DiffImpactOptions,
    ) -> anyhow::Result<Vec<DiffImpactMatch>>;

## Revision Note

2026-02-07: Created this Phase 4 planning-only ExecPlan to target precision and noise quality on
the existing command surface. Chosen approach emphasizes strict per-slice TDD, deterministic edge
resolution, explicit `diff-impact` seed controls, and scoped fallback filtering for `find`/`refs`
without JSON schema-family churn.

2026-02-07: Updated living sections during Milestone 18 execution with branch/workflow status,
contract-fixture additions, and strict red transcript evidence for slices 18A/18B/18C.

2026-02-07: Updated living sections during Milestone 19 implementation with resolver/adapter
decisions, discovered regressions, and green/refactor transcript evidence.

2026-02-07: Updated living sections during Milestone 20 implementation with option-surface
decisions and strict red/green/refactor transcripts for import and changed-line controls.
