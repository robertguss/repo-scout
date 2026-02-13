# Build `repo-scout` Phase 9 Runner-Aware Cross-Language Test Recommendations

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan builds on `agents/repo-scout-phase8-execplan.md`, which delivered semantic-precision
closure and `diff-impact` hardening, and explicitly left language-specific test-runner
recommendations as residual work.

## Superseded Status

This plan is now a **historical planning artifact**. Its intended outcomes were closed via later
implemented phases and convergence/hardening milestones:

- Phase 10 (`tests/milestone49_rust_hardening.rs`) removed invalid non-Rust `cargo test --test`
  targeting behavior.
- Phase 13 (`tests/milestone60_python_recommendations.rs`) shipped strict explicit-`pytest`
  detection and runnable Python recommendation synthesis.
- Phase 14 (`tests/milestone61_typescript_production.rs`) shipped strict unambiguous Jest/Vitest
  detection and runnable Node recommendation synthesis.
- Phase 15 (`tests/milestone62_cross_language_convergence.rs`,
  `tests/milestone63_cross_language_convergence_pack.rs`) shipped shared cross-language
  test-path/scope convergence and integrated recommendation contract validation.

Status statement: Phase 9 is **closed via later implemented phases** and retained for historical
context only.

## Purpose / Big Picture

Phase 9 makes test recommendations runnable across Rust, Python, and TypeScript/JavaScript while
keeping outputs deterministic and schema-stable. After this phase, users can run `tests-for`,
`verify-plan`, and `diff-impact` in mixed-language repositories without receiving incorrect Rust
`cargo test --test ...` commands for Python/TS test files.

User-visible outcome: targeted verification commands align with detected test frameworks (`pytest`,
`npx jest`, `npx vitest`) when evidence is explicit, nested/common cross-language test patterns are
detected consistently, and scope/test filtering behavior remains deterministic without JSON schema
changes.

## Progress

- [x] (2026-02-08 04:28Z) Re-read `agents/PLANS.md`, `AGENTS.md`, and
      `agents/repo-scout-phase8-execplan.md` to anchor Phase 9 scope in residual-work threads.
- [x] (2026-02-08 04:28Z) Re-ran planning baseline dogfood commands: `cargo run -- index --repo .`,
      `cargo run -- find test_command_for_target --repo . --json`, and
      `cargo run -- refs test_command_for_target --repo . --json`.
- [x] (2026-02-08 04:28Z) Captured baseline defects with temporary repos: Python/TS files under
      `tests/` currently produce `cargo test --test ...` steps; nested Python tests default to
      support-only; TS/JS `.spec/.test` files outside `tests/` are not discovered as test targets.
- [x] (2026-02-08 04:28Z) Authored this Phase 9 ExecPlan as planning-only work.
- [x] (2026-02-10 00:29Z) Milestone 42 closure objective delivered via later strict-TDD
      implementation phases and integration suites (see superseded status mapping).
- [x] (2026-02-10 00:29Z) Milestone 43 closure objective delivered in Phase 13 strict
      explicit-`pytest` runner-aware recommendation implementation.
- [x] (2026-02-10 00:57Z) Milestone 44 closure objective delivered in Phase 14 strict Jest/Vitest
      runner-aware recommendation implementation.
- [x] (2026-02-10 02:17Z) Milestone 45 closure objective delivered in Phase 15 shared
      cross-language test-like path convergence.
- [x] (2026-02-10 03:21Z) Milestone 46 closure objective delivered through later docs/evidence
      refresh and contract-validator closure in Phases 13-16.

## Surprises & Discoveries

- Observation: `verify-plan` currently treats Python and TypeScript files directly under `tests/` as
  Rust integration targets and emits invalid `cargo test --test ...` commands. Evidence: in
  temporary fixtures, `tests/test_app.py` yielded `cargo test --test test_app` and
  `tests/app.spec.ts` yielded `cargo test --test app.spec`.

- Observation: nested Python test files are discoverable only as support paths, not runnable
  targets, even when explicit `pytest` config exists. Evidence: with `pytest.ini` and
  `tests/unit/test_app.py`, `tests-for --json` returned no default rows and `--include-support`
  returned `target_kind = support_test_file`; `verify-plan` produced only the full-suite gate.

- Observation: TS/JS spec files outside `tests/` are currently omitted from test-target discovery
  and therefore absent from `tests-for` and `diff-impact` test-target output. Evidence: with
  `src/app.spec.ts` and `vitest` in `package.json`, `tests-for computePlan --json` returned
  `results: []` and `diff-impact --changed-file src/app.ts --json` emitted no
  `result_kind = test_target` rows.

- Observation: strict-mode runner detection is safer than convention guessing because unsupported
  repositories still retain the always-present full-suite `cargo test` gate. Evidence: baseline
  behavior already appends `cargo test` in `verify-plan`, so deterministic targeted-command
  suppression in ambiguous/non-detected cases does not remove safety gates.

## Decision Log

- Decision: keep schema versions unchanged (`1/2/3`) and ship runner awareness as additive behavior
  only. Rationale: downstream automation depends on current envelopes; this phase targets command
  correctness and target discovery, not contract migration. Date/Author: 2026-02-08 / Codex

- Decision: implement strict detection only (no fallback guessing) for non-Rust test runners.
  Rationale: precision-first recommendations avoid false-positive targeted commands and align with
  deterministic operator expectations. Date/Author: 2026-02-08 / Codex

- Decision: support Python + TS/JS in one phase, with TS/JS covering both Jest and Vitest.
  Rationale: both were explicitly identified as residual work and are needed for practical
  multi-language repositories. Date/Author: 2026-02-08 / Codex

- Decision: when Jest and Vitest signals are both present without a unique winner, emit no
  Node-targeted commands. Rationale: strict-mode ambiguity handling is safer than choosing an
  arbitrary runner. Date/Author: 2026-02-08 / Codex

- Decision: use direct command forms for targeted Node steps (`npx jest --runTestsByPath`,
  `npx vitest run`) and `pytest <path>` for Python. Rationale: these commands are deterministic and
  do not depend on package-manager inference or script aliasing. Date/Author: 2026-02-08 / Codex

- Decision: apply one shared cross-language test-path classifier to `tests-for`, `verify-plan`, and
  `diff-impact`, and reuse its test-like semantics for query `--exclude-tests` filtering. Rationale:
  consistency across command families reduces user confusion and avoids classifier drift.
  Date/Author: 2026-02-08 / Codex

## Outcomes & Retrospective

Historical planning outcome: Phase 9 correctly identified recommendation and scope-consistency gaps
without requiring schema or command-surface changes.

Closure outcome (superseded by later implementation phases): targeted runner-aware recommendations
and shared cross-language test-path behaviors are now shipped and validated across Phases 13-15,
with convergence and GA hardening coverage extended in Phases 15-16.

Residual work after closure remains optional maintenance/backlog scope (for example repository-level
runner overrides, richer environment-specific command synthesis, and additional benchmark corpora).

## Context and Orientation

`repo-scout` CLI parsing is in `src/cli.rs`; command dispatch and normalization are in
`src/main.rs`; query logic and test recommendation behavior are in `src/query/mod.rs`; terminal/JSON
rendering is in `src/output.rs`; integration tests are in `tests/`; docs are in `README.md` and
`docs/`.

Terms used in this plan:

- A "runner detection context" is deterministic repository metadata used to decide whether a test
  target has a runnable command (for example, `pytest` config markers or `package.json` Jest/Vitest
  signals).
- A "runnable target" is a discovered test file for which `verify-plan` can emit a concrete command
  that matches the detected framework.
- A "strict detection" policy means no command is emitted when framework evidence is missing or
  ambiguous.
- A "test-like path" is a file path that should be treated as test scope for both target discovery
  and `--exclude-tests` filtering.

Current hot spots for this phase:

- `src/query/mod.rs::test_targets_for_symbol` currently discovers only paths rooted under `tests/`
  (plus Rust `_test.rs` patterns), missing cross-language `.spec/.test` and Python naming patterns.
- `src/query/mod.rs::test_command_for_target` currently emits only Rust `cargo test --test <stem>`
  commands for direct `tests/<file>` paths.
- `src/query/mod.rs::is_runnable_test_target` and `verify_plan_for_changed_files` currently assume
  Rust-only command synthesis semantics.
- `src/query/mod.rs::is_test_like_path` currently does not include `.spec/.test` and Python
  test-name conventions used by modern TS/JS/Python projects.

## Contract Inputs

Phase 9 implementation must consume and reference these repository contract assets:

- Core policy: `contracts/core/RISK_TIER_POLICY.md`
- Language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Task framing template: `templates/TASK_PACKET_TEMPLATE.md`
- Test planning template: `templates/TEST_PLAN_TEMPLATE.md`
- Evidence template: `templates/EVIDENCE_PACKET_TEMPLATE.md`
- TDD validator: `scripts/validate_tdd_cycle.sh`
- Evidence validator: `scripts/validate_evidence_packet.sh`
- CI gate reference: `.github/workflows/contract-gates.yml`

Required validator commands before PR merge:

    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

## AGENTS.md Constraints

Consulted file:

- `AGENTS.md`

Effective constraints enforced by this plan:

- Strict red-green-refactor per feature slice, with no production edits before observed failing
  tests.
- Risk tier declaration before implementation.
- Dogfood commands before and after each milestone using `repo-scout` itself first.
- Integration-style tests in `tests/` with milestone naming.
- No schema/index DB commit drift (`.repo-scout/index.db` remains untracked/ignored).
- Contract validators required before PR.

If any instruction conflicts with Contract System v2 assets, this plan uses the stricter rule.

## Risk Tier and Required Controls

Phase 9 risk tier: `1` (moderate).

Rationale: this phase changes query/recommendation logic and CLI-observable behavior but avoids
schema migrations, irreversible operations, auth/security primitives, and persistence invariants.

Tier 1 controls required and mapped:

- Red -> Green -> Refactor evidence: required for every feature slice.
- Task packet: required via `templates/TASK_PACKET_TEMPLATE.md` mapping in this plan.
- Test plan: required via `templates/TEST_PLAN_TEMPLATE.md` mapping in milestone slices.
- Evidence packet: required via PR template headings and validator command.
- Rollback plan: required in `Idempotence and Recovery` section below.
- Reviewer count: at least one reviewer before merge.

## Strict TDD Contract

Phase 9 enforces strict red-green-refactor at feature-slice granularity. Production code changes are
forbidden until the owning failing test is observed for that exact slice.

A feature slice in this phase is one user-visible behavior unit, such as:

- "Python fixture emits `pytest <file>` in `verify-plan` when explicit pytest evidence exists."
- "Vitest fixture emits `npx vitest run <file>` and Jest fixture emits
  `npx jest --runTestsByPath <file>`."
- "Cross-language `.spec/.test` and Python test-name patterns are treated as test-like for discovery
  and exclusion filtering."

For every slice, capture:

- red transcript: failing test/check command and expected failure reason,
- green transcript: same command passing after minimal implementation,
- refactor transcript: full `cargo test` pass.

Record transcripts in this plan and append concise dogfood evidence to `docs/dogfood-log.md`.

## Plan of Work

### Milestone 42: Runner Contract Tests and Fixture Baseline

Milestone goal: establish failing contract tests and reusable fixtures before production edits.

Feature slice 42A adds a deterministic Python fixture under
`tests/fixtures/phase9/multi_runner/python_pytest/` and a failing test in
`tests/milestone42_runner_contracts.rs` asserting `verify-plan` emits `pytest <target>` for changed
Python symbols when explicit pytest evidence exists.

Feature slice 42B adds deterministic TypeScript fixtures under
`tests/fixtures/phase9/multi_runner/ts_vitest/` and `tests/fixtures/phase9/multi_runner/ts_jest/`
with failing tests asserting runner-specific targeted commands (`npx vitest run <target>` and
`npx jest --runTestsByPath <target>`).

Feature slice 42C adds an ambiguous-node fixture under
`tests/fixtures/phase9/multi_runner/ts_ambiguous/` and failing tests asserting strict-mode behavior
(no Node-targeted step when both runners are signaled), plus schema-envelope stability checks (`2`
for `tests-for`/`verify-plan`, `3` for `diff-impact`).

### Milestone 43: Python Detection and Command Synthesis

Milestone goal: ship deterministic pytest-aware runnable targeting.

Feature slice 43A introduces a private runner-detection context in `src/query/mod.rs` built from
repository root derived from `db_path`.

Pytest detection is explicit-only and true when at least one marker exists:

- `pytest.ini` file,
- `setup.cfg` containing `[tool:pytest]`,
- `tox.ini` containing `[pytest]`,
- `pyproject.toml` containing `[tool.pytest.ini_options]`,
- `pyproject.toml` or `requirements*.txt` containing explicit `pytest` dependency text.

Feature slice 43B updates command synthesis so Python runnable targets emit `pytest <target>` only
when pytest is detected; otherwise they remain non-runnable/support.

Feature slice 43C preserves current defaults:

- `tests-for` returns runnable targets by default,
- `tests-for --include-support` restores non-runnable support paths,
- `verify-plan` preserves changed-file targeted rows only when their test targets are runnable under
  detected runner rules.

### Milestone 44: Jest/Vitest Detection and Strict Node Synthesis

Milestone goal: ship deterministic Node runner-aware targeted steps.

Feature slice 44A parses `package.json` with existing `serde_json` and collects explicit runner
signals from:

- `scripts.test` string containing `jest` and/or `vitest`,
- dependency keys in `dependencies`, `devDependencies`, `peerDependencies`, and
  `optionalDependencies` with exact names `jest` and/or `vitest`.

Feature slice 44B applies deterministic runner selection:

- vitest-only signal -> Vitest,
- jest-only signal -> Jest,
- both or none -> no Node targeted command.

Feature slice 44C updates Node command synthesis:

- Jest: `npx jest --runTestsByPath <target>`
- Vitest: `npx vitest run <target>`

No command is emitted for ambiguous/non-detected Node runner contexts.

### Milestone 45: Shared Cross-Language Test-Path Classifier and Scope Consistency

Milestone goal: unify test-like semantics for discovery and filtering.

Feature slice 45A introduces a shared path-classifier helper in `src/query/mod.rs` used by
`test_targets_for_symbol`, `is_test_like_path`, and related recommendation/scoping logic.

Classifier patterns (precision-first common set):

- `tests/**`
- `**/__tests__/**`
- Python: `test_*.py`, `*_test.py`
- TS/JS: `*.spec.ts`, `*.spec.tsx`, `*.spec.js`, `*.spec.jsx`, `*.test.ts`, `*.test.tsx`,
  `*.test.js`, `*.test.jsx`

Feature slice 45B extends `tests-for` and `verify-plan` discovery to these patterns while
maintaining deterministic ranking and support/runnable behavior.

Feature slice 45C ensures `diff-impact` test-target generation and `--exclude-tests` filtering use
consistent test-like semantics via shared classifier behavior.

### Milestone 46: Documentation, Dogfood, and Validation Closure

Milestone goal: align docs/evidence with shipped Phase 9 behavior and close all gates.

Feature slice 46A updates documentation for runner-aware recommendation behavior and strict
detection rules:

- `README.md`
- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`

Feature slice 46B updates evidence/perf logs with Phase 9 dogfood command packs and transcripts:

- `docs/dogfood-log.md`
- `legacy performance baseline doc (removed)`

Feature slice 46C finalizes the living plan and runs all required gates:

- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo fmt -- --check`
- contract validators listed above.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/repo-scout`.

Before each milestone, run baseline dogfood:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json

Strict per-slice TDD loop (required order for every slice):

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure before production edits
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimal implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 42 expected slice commands:

    cargo test milestone42_python_pytest_verify_plan_command -- --nocapture
    cargo test milestone42_vitest_verify_plan_command -- --nocapture
    cargo test milestone42_jest_verify_plan_command -- --nocapture
    cargo test milestone42_ambiguous_node_runner_skips_targeted_step -- --nocapture
    cargo test milestone42_runner_contract_schema_stability -- --nocapture

Milestone 43 expected slice commands:

    cargo test milestone42_python_pytest_verify_plan_command -- --nocapture
    cargo test milestone42_python_nested_test_is_runnable_when_pytest_detected -- --nocapture
    cargo test milestone42_python_non_detected_runner_stays_support_only -- --nocapture

Milestone 44 expected slice commands:

    cargo test milestone42_vitest_verify_plan_command -- --nocapture
    cargo test milestone42_jest_verify_plan_command -- --nocapture
    cargo test milestone42_ambiguous_node_runner_skips_targeted_step -- --nocapture

Milestone 45 expected slice commands:

    cargo test milestone45_tests_for_discovers_cross_language_test_patterns -- --nocapture
    cargo test milestone45_diff_impact_includes_cross_language_test_targets -- --nocapture
    cargo test milestone45_exclude_tests_filters_spec_and_python_test_patterns -- --nocapture
    cargo test milestone45_scope_output_is_deterministic -- --nocapture

Milestone 46 verification commands:

    cargo run -- index --repo .
    cargo run -- tests-for test_command_for_target --repo . --json
    cargo run -- verify-plan --changed-file src/query/mod.rs --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json
    cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo fmt -- --check
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

Fixture-focused cross-language dogfood pack (after Milestone 44 and again after Milestone 46):

    cargo run -- index --repo tests/fixtures/phase9/multi_runner/python_pytest
    cargo run -- verify-plan --changed-file src/app.py --repo tests/fixtures/phase9/multi_runner/python_pytest --json
    cargo run -- tests-for compute_plan --repo tests/fixtures/phase9/multi_runner/python_pytest --json

    cargo run -- index --repo tests/fixtures/phase9/multi_runner/ts_vitest
    cargo run -- verify-plan --changed-file src/app.ts --repo tests/fixtures/phase9/multi_runner/ts_vitest --json
    cargo run -- tests-for computePlan --repo tests/fixtures/phase9/multi_runner/ts_vitest --json

    cargo run -- index --repo tests/fixtures/phase9/multi_runner/ts_jest
    cargo run -- verify-plan --changed-file src/app.ts --repo tests/fixtures/phase9/multi_runner/ts_jest --json

    cargo run -- index --repo tests/fixtures/phase9/multi_runner/ts_ambiguous
    cargo run -- verify-plan --changed-file src/app.ts --repo tests/fixtures/phase9/multi_runner/ts_ambiguous --json

Expected observable progression:

- Before Milestone 43 green, Python fixtures still produce Rust-targeted steps or no runnable nested
  targets.
- After Milestone 43 green, Python fixtures emit `pytest <file>` for explicit-detection contexts and
  keep non-detected contexts support-only.
- After Milestone 44 green, Vitest/Jest fixtures emit runner-specific commands; ambiguous fixture
  emits no Node-targeted command.
- After Milestone 45 green, `.spec/.test` and Python naming patterns are discoverable consistently
  in `tests-for`/`verify-plan`/`diff-impact` and respected by `--exclude-tests` filtering.
- After Milestone 46, docs and evidence match shipped behavior and all gates are green.

## Validation and Acceptance

Acceptance is behavior-first and must be observable in command output plus integration tests.

After Milestone 43:

- `verify-plan --changed-file src/app.py --repo tests/fixtures/phase9/multi_runner/python_pytest --json`
  includes a targeted step with `step = "pytest tests/test_app.py"` (or fixture-equivalent path) and
  does not include `cargo test --test test_app`.
- `tests-for compute_plan ... --json` returns nested Python test targets as runnable where pytest is
  explicitly detected.

After Milestone 44:

- Vitest fixture emits `npx vitest run <target>` in targeted steps.
- Jest fixture emits `npx jest --runTestsByPath <target>` in targeted steps.
- Ambiguous fixture emits no Node-targeted step while retaining the mandatory full-suite
  `cargo test` row.

After Milestone 45:

- `tests-for` and `diff-impact` include cross-language test patterns under shared classifier rules.
- `--exclude-tests` removes `.spec/.test` and Python test-name pattern rows consistently.
- Repeated runs with identical inputs remain byte-identical.

After Milestone 46:

- docs describe runner-aware behavior and strict detection limits accurately,
- all validation commands in `Concrete Steps` pass,
- strict TDD transcripts for each feature slice are recorded in this plan.

## Idempotence and Recovery

This phase is additive and idempotent. Re-running index/query/test commands should not mutate
repository-tracked state except planned documentation updates and new milestone/fixture files.

No schema changes are planned. If implementation pressure suggests schema changes, stop and record a
new decision with rationale before proceeding.

Recovery guidance for regressions:

- keep all runner-aware logic isolated in `src/query/mod.rs` helper functions,
- if regressions appear, revert runner synthesis paths while preserving existing Rust behavior,
- preserve `verify-plan` full-suite `cargo test` row at all times to avoid safety-gate regression.

## Artifacts and Notes

Phase 9 planning baseline transcript:

    $ cargo run -- index --repo .
    index_path: ./.repo-scout/index.db
    schema_version: 3
    indexed_files: 61
    skipped_files: 63

    $ cargo run -- find test_command_for_target --repo . --json
    { "schema_version": 1, "command": "find", "results": 1 }

    $ cargo run -- refs test_command_for_target --repo . --json
    { "schema_version": 1, "command": "refs", "results": 4 }

Baseline-gap evidence gathered during planning:

    # Python fixture with tests/test_app.py currently emits Rust-style targeted command
    $ cargo run -- verify-plan --changed-file src/app.py --repo <tmp_py_repo> --json
    results include: step="cargo test --test test_app"

    # TS fixture with tests/app.spec.ts currently emits Rust-style targeted command
    $ cargo run -- verify-plan --changed-file src/app.ts --repo <tmp_ts_repo> --json
    results include: step="cargo test --test app.spec"

    # Nested Python tests remain support-only even with pytest.ini today
    $ cargo run -- tests-for compute_plan --repo <tmp_nested_py_repo> --json
    results: []

    $ cargo run -- tests-for compute_plan --repo <tmp_nested_py_repo> --include-support --json
    results include: target="tests/unit/test_app.py", target_kind="support_test_file"

    # TS spec file outside tests/ is omitted from discovery/test-target output today
    $ cargo run -- tests-for computePlan --repo <tmp_src_spec_repo> --json
    results: []

    $ cargo run -- diff-impact --changed-file src/app.ts --repo <tmp_src_spec_repo> --json
    no result_kind="test_target" rows present

All Milestone 42-46 red/green/refactor transcripts must be appended here during implementation.

## Interfaces and Dependencies

Phase 9 should use existing dependencies only. `serde_json` is already present and sufficient for
`package.json` parsing. Do not add new crates unless required and documented in `Decision Log`.

Expected interface touch points:

- `src/query/mod.rs`
  - add runner-detection helpers and shared test-path classifier,
  - update `test_targets_for_symbol`, `test_command_for_target`, `is_runnable_test_target`,
    `verify_plan_for_changed_files`, and `is_test_like_path` semantics.

- `tests/milestone42_runner_contracts.rs`
  - new strict TDD suite for runner-aware command contracts and schema stability checks.

- `tests/milestone45_cross_language_test_scope.rs`
  - new strict TDD suite for shared classifier behavior across command families.

- `tests/fixtures/phase9/multi_runner/...`
  - new fixture corpus for pytest, vitest, jest, and ambiguous Node detection scenarios.

- documentation updates:
  - `agents/repo-scout-phase9-execplan.md`
  - `README.md`
  - `docs/cli-reference.md`
  - `docs/json-output.md`
  - `docs/architecture.md`
  - `docs/dogfood-log.md`
  - `legacy performance baseline doc (removed)`

## Revision Note

2026-02-08: Created initial Phase 9 execution plan to implement runner-aware cross-language test
recommendations and shared test-path scope consistency, based on Phase 8 residual work and live
baseline defect reproduction, aligned to `agents/PLANS.md` strict TDD and living-plan requirements.
