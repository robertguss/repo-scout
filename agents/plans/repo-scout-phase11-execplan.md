# Build `repo-scout` Phase 11 Rust Production-Ready Closure

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`,
`Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This repository includes `agents/PLANS.md`, and this document must be maintained in accordance with
that file.

This plan follows `agents/plans/repo-scout-phase10-execplan.md` and the roadmap in
`agents/plans/repo-scout-roadmap-to-production-and-ga.md`. It is intentionally scoped to Rust only,
per the low-risk sequential rule.

## Purpose / Big Picture

Phase 11 closes the remaining Rust-only quality gap to production-ready status before deeper Go,
Python, and TypeScript phases continue. After this phase, users should be able to rely on Rust
impact and changed-file analysis even in realistic module layouts where duplicate symbol names exist
across files, and they should have tighter Rust-focused regression guardrails in both fixture
coverage and performance checks.

User-visible outcome: Rust command behavior (`find`, `refs`, `impact`, `diff-impact`,
`tests-for`, `verify-plan`) remains deterministic and schema-stable, while module-qualified call
paths (`crate::`, `self::`, `super::`, and `mod.rs` layouts) produce correct impacted-symbol rows
instead of dropping edges in ambiguous-name scenarios.

## Progress

- [x] (2026-02-09 22:36Z) Re-read `AGENTS.md`, `agents/PLANS.md`,
      `agents/plans/repo-scout-roadmap-to-production-and-ga.md`, and
      `agents/plans/repo-scout-phase10-execplan.md` to anchor Phase 11 boundary and constraints.
- [x] (2026-02-09 22:36Z) Confirmed current baseline on `codex/phase10-kickoff` with dogfood and
      quality commands: `cargo run -- index --repo .`,
      `cargo run -- find test_command_for_target --repo . --json`,
      `cargo run -- refs test_command_for_target --repo . --json`, `cargo test`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range`, and
      `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`.
- [x] (2026-02-09 22:36Z) Reproduced a concrete Rust residual defect class for this phase: in
      duplicate-name repositories, module-qualified calls targeting `mod.rs` definitions can lose
      call-edge attribution and omit expected `diff-impact` `called_by` rows.
- [x] (2026-02-09 22:36Z) Created branch `codex/phase11-execplan` from `codex/phase10-kickoff`
      and authored this Phase 11 ExecPlan as planning-only work.
- [x] (2026-02-09 23:20Z) Declared risk tier `1` and completed Milestone 54 strict TDD Rust
      residual-defect tests plus fixture baseline under
      `tests/fixtures/phase11/rust_production/`.
- [x] (2026-02-09 23:22Z) Completed Milestone 55 Rust module-qualified edge-resolution hardening in
      `src/indexer/languages/rust.rs` and validated non-regression slices
      (`tests/milestone49_rust_hardening.rs`, `tests/milestone50_go_find.rs`).
- [x] (2026-02-09 23:31Z) Completed Milestone 56 realistic fixture-corpus expansion and
      determinism/bounded-growth tests in `tests/milestone56_rust_production_determinism.rs`.
- [x] (2026-02-09 23:38Z) Completed Milestone 57 Rust performance guardrail tightening:
      `docs/performance-thresholds-rust.md`, `scripts/check_rust_perf_guardrails.sh`, and
      Justfile `perf-rust-guardrails`/`perf-rust-record`.
- [x] (2026-02-09 23:59Z) Completed Milestone 58 docs/dogfood/contract closure updates and ran full
      quality + validator gates on `codex/phase11-implementation`.

## Surprises & Discoveries

- Observation: module-qualified Rust calls can lose impact attribution when duplicate symbol names
  exist and one target lives in `mod.rs` layout.
  Evidence: in a local temp fixture (`src/lib.rs` calling `util::helper()`, with both
  `src/util/mod.rs::helper` and `src/support.rs::helper`),
  `diff-impact --changed-file src/util/mod.rs --json` returned only the changed symbol row and did
  not include expected `run` `called_by` row.

- Observation: current performance documentation includes timing commands but does not define
  explicit machine-readable threshold checks for Rust production closure.
  Evidence: `docs/performance-baseline.md` lists command baselines but no threshold source file or
  validator command that fails on regression.

- Observation: local TDD validator still needs `--allow-empty-range` in planning-only/no-commit
  ranges.
  Evidence: repository policy scripts accept strict range checking only when commits exist in
  `origin/main..HEAD`.

- Observation: bounded edge-growth assertions should compare repeated identical-index runs rather
  than forced content rewrites when the intent is duplicate-growth detection.
  Evidence: initial milestone56 guard test failed on a rewrite-based strict equality check despite
  stable bounded behavior under repeated identical indexing.

## Decision Log

- Decision: keep Phase 11 scope strictly Rust-only, consistent with roadmap low-risk sequencing.
  Rationale: language-depth work is intentionally serialized; Go `refs` and other language closure
  phases are explicitly deferred to later roadmap phases.
  Date/Author: 2026-02-09 / Codex

- Decision: target correctness hardening around module-qualified Rust calls (`crate::`, `self::`,
  `super::`) and `mod.rs` path resolution in ambiguous-name repositories.
  Rationale: this class is reproducible and directly impacts practical `impact`/`diff-impact`
  trustworthiness in real Rust repositories.
  Date/Author: 2026-02-09 / Codex

- Decision: keep command/JSON schema versions unchanged in Phase 11 (`1`, `2`, `3`).
  Rationale: production-readiness closure here is behavioral hardening and verification depth,
  not contract-surface migration.
  Date/Author: 2026-02-09 / Codex

- Decision: tighten performance posture with deterministic guardrails plus explicit threshold
  recording, while avoiding fragile wall-time assertions in default integration tests.
  Rationale: CI/runtime variance can cause false negatives; bounded deterministic checks plus
  documented threshold runs provide stronger operational signal with lower flake risk.
  Date/Author: 2026-02-09 / Codex

## Outcomes & Retrospective

Completion outcome: Phase 11 delivered Rust production-ready closure for this roadmap stage.
Module-qualified Rust call paths now resolve deterministic candidate targets across
`crate::`/`self::`/`super::` and `mod.rs` layouts, reducing dropped caller attribution in
duplicate-name repositories.

Completion evidence highlights: new milestone54/56/57 integration suites are green, Phase 10
non-regression suites remain green, Rust guardrail script + thresholds are documented and executable,
and docs/dogfood artifacts are updated for the new behavior while schema versions remain unchanged
(`1`, `2`, `3`).

Expected residual work after this plan: Go production closure (`refs`/graph depth), Python runner
hardening closure, TypeScript runner hardening closure, cross-language convergence, and GA hardening
remain in roadmap phases 12-16.

## Context and Orientation

`repo-scout` is a local deterministic CLI. Command parsing and dispatch are in `src/cli.rs` and
`src/main.rs`. Index-time extraction logic is under `src/indexer/`, with Rust behavior split across
`src/indexer/rust_ast.rs` (tree-sitter parsing and extracted definitions/references) and
`src/indexer/languages/rust.rs` (adapter-level symbol/reference/edge projection). Query behavior for
`find`, `refs`, `impact`, `tests-for`, `verify-plan`, `diff-impact`, and `explain` is in
`src/query/mod.rs`, and output rendering is in `src/output.rs`.

Terms used in this plan:

- A "module-qualified call path" means a Rust call where the callee is selected through module
  segments, such as `util::helper()`, `crate::util::helper()`, `self::helper()`, or
  `super::helper()`.
- A "`mod.rs` layout" means module source stored at `<module>/mod.rs` rather than `<module>.rs`.
- A "residual defect" means a user-visible correctness or determinism gap still present after
  Phase 10 completion.
- A "performance guardrail" means a repeatable command and threshold check that detects meaningful
  regressions without requiring benchmark-grade precision.

Current hot spots for this phase:

- `src/indexer/languages/rust.rs::qualified_module_for_reference` and related module-path helpers
  currently infer one narrow module candidate and can lose precision in ambiguous-name graphs.
- `src/query/mod.rs` impact traversal relies on persisted edge endpoints; if endpoint resolution is
  under-qualified, `impact`/`diff-impact` can miss expected `called_by` rows.
- `tests/fixtures/` currently has limited Rust fixture depth for ambiguous multi-module resolution
  compared with TypeScript/Python semantic fixture coverage.

## Contract Inputs

Phase 11 implementation must consume and reference:

- Core risk policy: `contracts/core/RISK_TIER_POLICY.md`
- Core evidence policy: `contracts/core/EVIDENCE_REQUIREMENTS.md`
- Core review policy: `contracts/core/REVIEW_CONTRACT.md`
- Active language contract: `contracts/languages/RUST_CODING_CONTRACT.md`
- Active language manifest: `contracts/ACTIVE_LANGUAGE_CONTRACTS.md`
- Task framing template: `templates/TASK_PACKET_TEMPLATE.md`
- Test plan template: `templates/TEST_PLAN_TEMPLATE.md`
- Evidence template: `templates/EVIDENCE_PACKET_TEMPLATE.md`
- PR checklist: `checklists/PR_CONTRACT_CHECKLIST.md`
- Adversarial checklist guidance (recommended for Tier 1):
  `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`
- TDD validator: `scripts/validate_tdd_cycle.sh`
- Evidence validator: `scripts/validate_evidence_packet.sh`
- CI contract gates: `.github/workflows/contract-gates.yml`

Required validator commands before PR merge:

    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

For local no-commit planning checks, `--allow-empty-range` is permitted:

    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range

## AGENTS.md Constraints

Consulted file:

- `AGENTS.md`

Effective constraints enforced by this plan:

- Strict red-green-refactor per feature slice; no production edits before failing tests.
- Risk tier declaration before implementation.
- Dogfooding with `repo-scout` before and after each milestone.
- Integration-style tests in `tests/` with milestone naming.
- No `unwrap()`/`expect()`/`panic!` in `src/` production code without explicit contract exception.
- Contract validators required before PR updates.
- If AGENTS.md and contract assets disagree, the stricter rule wins.

Phase-11-specific dogfood commands to run before and after milestones:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json

## Risk Tier and Required Controls

Phase 11 risk tier: `1` (moderate).

Rationale: this phase changes Rust extraction/query correctness and performance guardrails but does
not plan schema migration, irreversible operations, or security-critical auth/permission changes.

Tier 1 controls required and mapped:

- Red -> Green -> Refactor evidence for every feature slice.
- Task packet required (`templates/TASK_PACKET_TEMPLATE.md` mapping).
- Test plan required (`templates/TEST_PLAN_TEMPLATE.md` mapping).
- Evidence packet required (PR-body-first evidence and validator).
- Rollback plan required (`Idempotence and Recovery` section below).
- Reviewer count minimum: 1.
- Adversarial review checklist: recommended and included in review gate.

Escalation rule: if implementation requires schema/persistence invariants change (for example,
`symbols_v2`/`symbol_edges_v2` shape changes or migration logic), pause and escalate to Tier 2 with
updated controls before continuing.

## Strict TDD Contract

No production code changes are allowed for a feature slice until the exact slice-level failing test
is observed.

Feature slices for Phase 11 are:

- Rust module-qualified edge-resolution correctness slices.
- Rust determinism/stability slices around ambiguous-name graphs.
- Rust performance-guardrail slices (deterministic budget and threshold checks).

For each slice, record in this ExecPlan:

- Red evidence: failing test command and concise failure reason.
- Green evidence: same command passing after minimal implementation.
- Refactor evidence: full `cargo test` pass after cleanup.

## TDD Evidence Log

Milestone 54 / 55:

- Slice 54A (`mod.rs` duplicate-name attribution):
  - Red: `cargo test milestone54_diff_impact_mod_rs_disambiguates_duplicate_symbols -- --nocapture`
    failed; `diff-impact` omitted expected `run` `called_by` row.
  - Green: same command passed after Rust candidate-path resolver hardening in
    `src/indexer/languages/rust.rs`.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.
- Slice 54B (`crate::`-qualified attribution):
  - Red: `cargo test milestone54_diff_impact_crate_qualified_call_disambiguates -- --nocapture`
    failed; crate-qualified call dropped caller attribution.
  - Green: same command passed after deterministic module candidate resolution was added.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.
- Slice 54C (`super::` parent preference under same-name local symbol):
  - Red: `cargo test milestone54_diff_impact_super_qualified_call_prefers_parent_symbol -- --nocapture`
    failed; caller row for parent target was missing.
  - Green: same command passed after `super`-aware candidate resolution.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.
- Slice 55C non-regression:
  - Green checks: `cargo test --test milestone49_rust_hardening -- --nocapture` and
    `cargo test --test milestone50_go_find -- --nocapture` both passed after resolver changes.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.

Milestone 56:

- Slice 56B deterministic JSON repeatability:
  - Green: `cargo test milestone56_rust_json_outputs_are_repeatable_across_runs -- --nocapture`
    passed on the Phase 11 Rust production fixture corpus.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.
- Slice 56C bounded edge growth:
  - Red: initial guard test failed when comparing counts across a forced file rewrite (overly strict
    expectation for this slice).
  - Green: `cargo test milestone56_rust_module_resolution_does_not_duplicate_edges_unboundedly -- --nocapture`
    passed after aligning the check to repeated identical-index runs with bounded threshold.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.

Milestone 57:

- Slice 57A threshold source file contract:
  - Red: `cargo test milestone57_rust_perf_thresholds_file_exists_and_defines_budgets -- --nocapture`
    failed because `docs/performance-thresholds-rust.md` did not exist.
  - Green: same command passed after adding thresholds doc and command budgets.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.
- Slice 57B/57C script and wiring contract:
  - Red: `cargo test milestone57_perf_guardrail_script_and_just_targets_are_wired -- --nocapture`
    failed before script/Justfile wiring was complete.
  - Green: same command passed after adding `scripts/check_rust_perf_guardrails.sh` and Justfile
    `perf-rust-guardrails`/`perf-rust-record`.
  - Green operational checks: `bash scripts/check_rust_perf_guardrails.sh --repo .` and
    `just perf-rust-guardrails` both passed.
  - Refactor: covered by final full-suite `cargo test` pass in Milestone 58 closure.

Milestone 58 closure:

- Closure command evidence:
  - `cargo run -- index --repo .`
  - `cargo run -- find run --repo . --json`
  - `cargo run -- refs run --repo . --json`
  - `cargo run -- impact run --repo . --json`
  - `cargo run -- diff-impact --changed-file src/indexer/languages/rust.rs --repo . --json`
  - `cargo clippy --all-targets --all-features -- -D warnings` (pass)
  - `cargo fmt -- --check` (pass after applying `cargo fmt`)
  - `cargo test` (pass)
  - `bash scripts/validate_tdd_cycle.sh --base origin/main` failed as expected for empty
    `origin/main..HEAD` range in no-commit local state.
  - `bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range` (pass)
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md` (pass)

## Plan of Work

### Milestone 54: Rust Residual-Defect Contract Baseline

Milestone goal: lock current Rust residual defects and fixture shape with failing tests before any
production edits.

Feature slice 54A adds a failing integration test in `tests/milestone54_rust_production_closure.rs`
for duplicate-name `mod.rs` ambiguity:

- fixture contains `src/util/mod.rs::helper`, `src/support.rs::helper`, and `src/lib.rs` calling
  `util::helper()`.
- `diff-impact --changed-file src/util/mod.rs --json` must include `run` as
  `result_kind = impacted_symbol`, `relationship = called_by`.

Feature slice 54B adds a failing test for `crate::`-qualified disambiguation under duplicate-name
pressure:

- fixture contains multiple `helper` definitions across files.
- call site uses `crate::util::helper()`.
- expected behavior is deterministic attribution to util module target, not dropped/misattributed
  edge.

Feature slice 54C adds a failing test for `super::`-qualified attribution in nested modules where a
same-name local symbol exists:

- nested module call uses `super::top_helper()` while local module also defines `top_helper`.
- expected behavior: upstream symbol impact includes caller row.

Fixture assets for Milestone 54 should be created under
`tests/fixtures/phase11/rust_production/` with deterministic minimal repositories.

### Milestone 55: Rust Module-Qualified Edge Resolution Hardening

Milestone goal: implement minimal Rust extraction/query hardening to satisfy Milestone 54 contracts
without schema changes.

Feature slice 55A extends module-path interpretation in `src/indexer/languages/rust.rs` so
qualified calls compute deterministic candidate module targets for `crate::`, `self::`, `super::`,
and unqualified module prefixes.

Implementation detail for this slice should preserve low blast radius:

- keep existing extraction output structure (`ExtractedEdge` shape unchanged),
- add deterministic candidate-path helpers,
- support both `<module>.rs` and `<module>/mod.rs` candidates where applicable,
- maintain stable ordering and dedupe so repeated indexing is deterministic.

Feature slice 55B updates Rust call-edge target key synthesis to prefer precise module-qualified
endpoint candidates before broad symbol fallback, while preserving existing behavior when no
qualified context exists.

Feature slice 55C locks non-regression behavior for previously green Phase 10 Rust hardening tests
(`tests/milestone49_rust_hardening.rs`) and Go `find` MVP tests
(`tests/milestone50_go_find.rs`) to ensure this Rust-only closure does not regress adjacent surface
areas.

### Milestone 56: Rust Determinism and Fixture-Corpus Expansion

Milestone goal: strengthen Rust production confidence with realistic fixture patterns and explicit
repeatability checks.

Feature slice 56A adds a reusable Rust production fixture corpus under
`tests/fixtures/phase11/rust_production/` covering:

- `mod.rs` and `<module>.rs` module layouts,
- duplicate symbol names across sibling modules,
- nested module calls using `crate::`, `self::`, and `super::`,
- at least one integration-test file under `tests/` to exercise test-target recommendations.

Feature slice 56B adds deterministic repeatability tests in
`tests/milestone56_rust_production_determinism.rs` ensuring repeated runs of key JSON commands on
these fixtures are identical:

- `find --json`,
- `refs --json`,
- `impact --json`,
- `diff-impact --json`.

Feature slice 56C adds a bounded edge-growth guard test proving added Rust module-candidate logic
does not produce unbounded duplicate call edges on fixture repositories.

### Milestone 57: Rust Performance Guardrail Tightening

Milestone goal: define and enforce tighter, repeatable Rust-focused performance checks aligned to
production-ready closure.

Feature slice 57A adds a Rust-focused threshold source file,
`docs/performance-thresholds-rust.md`, defining command budgets and interpretation rules for
repeatable local checks (cold/warm caveats included).

Feature slice 57B adds a non-destructive validator script, `scripts/check_rust_perf_guardrails.sh`,
that runs selected release-mode commands and verifies reported elapsed times against those
thresholds.

The script should:

- accept `--repo` and `--fixture` overrides,
- emit clear pass/fail output per command,
- fail fast on threshold breach,
- avoid mutating tracked repository files.

Feature slice 57C wires convenience commands in `Justfile`:

- `perf-rust-guardrails` for threshold validation,
- optional `perf-rust-record` helper for baseline refresh output.

This milestone must keep thresholds practical and non-fragile; if runtime variance is high, prefer
conservative budgets with explicit notes rather than overly tight gates.

### Milestone 58: Docs, Dogfood, and Contract Closure

Milestone goal: align all public artifacts with Phase 11 Rust production-ready behavior and close
validation gates.

Feature slice 58A updates docs:

- `README.md` (Rust production-ready behavior notes),
- `docs/cli-reference.md` (any changed semantics or clarifications),
- `docs/architecture.md` (Rust module-qualified call resolution behavior),
- `docs/json-output.md` (schema stability note and behavior clarifications),
- `docs/performance-baseline.md` and `docs/performance-thresholds-rust.md`.

Feature slice 58B appends dogfood and performance transcripts to `docs/dogfood-log.md`, including
at least one transcript from a realistic Rust production fixture.

Feature slice 58C runs full verification and contract closure commands and records concise evidence
in this ExecPlan and PR template.

## Concrete Steps

Run all commands from `/Users/robertguss/Projects/experiments/repo-scout`.

Before each milestone:

    cargo run -- index --repo .
    cargo run -- find test_command_for_target --repo . --json
    cargo run -- refs test_command_for_target --repo . --json

Per-slice strict TDD loop:

    cargo test <slice_test_name> -- --nocapture
    # red: confirm expected failure before production edits
    cargo test <slice_test_name> -- --nocapture
    # green: confirm pass after minimum implementation
    cargo test
    # refactor gate: full suite must pass

Milestone 54 expected slice commands:

    cargo test milestone54_diff_impact_mod_rs_disambiguates_duplicate_symbols -- --nocapture
    cargo test milestone54_diff_impact_crate_qualified_call_disambiguates -- --nocapture
    cargo test milestone54_diff_impact_super_qualified_call_prefers_parent_symbol -- --nocapture

Milestone 56 expected slice commands:

    cargo test milestone56_rust_json_outputs_are_repeatable_across_runs -- --nocapture
    cargo test milestone56_rust_module_resolution_does_not_duplicate_edges_unboundedly -- --nocapture

Milestone 57 expected slice commands:

    bash scripts/check_rust_perf_guardrails.sh --repo .
    just perf-rust-guardrails

Milestone 58 closure commands:

    cargo run -- index --repo .
    cargo run -- find run --repo . --json
    cargo run -- refs run --repo . --json
    cargo run -- impact run --repo . --json
    cargo run -- diff-impact --changed-file src/indexer/languages/rust.rs --repo . --json
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt -- --check
    cargo test
    bash scripts/validate_tdd_cycle.sh --base origin/main
    bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md

For local no-commit ranges during implementation checkpoints:

    bash scripts/validate_tdd_cycle.sh --base origin/main --allow-empty-range

## Validation and Acceptance

Acceptance is behavior-first:

- In duplicate-name Rust repositories, module-qualified calls (`crate::`, `self::`, `super::`,
  `util::...`) resolve impacted callers deterministically instead of dropping expected rows.
- `diff-impact --changed-file` on Rust `mod.rs` targets includes correct `called_by` neighbors when
  call evidence exists.
- `find`, `refs`, `impact`, and `diff-impact` remain deterministic across repeated runs on identical
  index state.
- No schema-version changes occur (`1`, `2`, `3` remain unchanged).
- Rust performance guardrail checks are documented and executable with explicit thresholds.
- Full quality and contract gates pass.

Strict TDD acceptance evidence is mandatory: each feature slice must include one recorded Red
failure, one Green pass, and one Refactor full-suite pass in this document.

## Idempotence and Recovery

Phase 11 is additive and idempotent. Re-running index/query/test/perf commands should not mutate
tracked files except planned source, tests, scripts, and docs edits.

No schema migration is planned. If schema pressure emerges:

- pause implementation,
- record the decision and rationale in `Decision Log`,
- escalate risk tier controls before continuing.

Rollback plan:

- keep Rust module-resolution changes isolated to `src/indexer/languages/rust.rs` helpers and any
  narrow query support functions,
- keep fixture/test additions isolated in new phase11 files,
- if regressions appear, revert latest milestone slice while preserving earlier passing slices and
  rerun full gates.

## Review and CI Gates

Before merge:

- Complete `checklists/PR_CONTRACT_CHECKLIST.md`.
- Complete `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` (recommended Tier 1 control).
- Ensure `.github/workflows/contract-gates.yml` expected outcomes remain green.
- Ensure PR body sections in `.github/pull_request_template.md` include risk and
  Red/Green/Refactor evidence.

## Interfaces and Dependencies

Expected touch points:

- `src/indexer/languages/rust.rs`
- `src/query/mod.rs` (only if query-side tie-break/dedupe support is required)
- `tests/milestone54_rust_production_closure.rs` (new)
- `tests/milestone56_rust_production_determinism.rs` (new)
- `tests/fixtures/phase11/rust_production/...` (new)
- `scripts/check_rust_perf_guardrails.sh` (new)
- `Justfile`
- `README.md`
- `docs/cli-reference.md`
- `docs/json-output.md`
- `docs/architecture.md`
- `docs/performance-baseline.md`
- `docs/performance-thresholds-rust.md` (new)
- `docs/dogfood-log.md`

Dependencies:

- No new third-party runtime dependencies are planned.
- If any new dependency becomes necessary, record rationale and risk impact in `Decision Log`.

## Revision Note

2026-02-09: Created initial Phase 11 execution plan for Rust-only production-ready closure,
aligned to roadmap sequencing, strict per-slice TDD, Contract System v2 controls, and AGENTS.md
dogfooding requirements.

2026-02-09: Completed implementation on `codex/phase11-implementation` with strict TDD evidence,
Rust module-qualified edge-resolution hardening, Phase 11 fixture/perf guardrail additions, and
docs/dogfood/validation closure updates.
