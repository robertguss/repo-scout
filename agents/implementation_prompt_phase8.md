You are implementing Phase 8 for `repo-scout` in `/Users/robertguss/Projects/experiments/repo-scout`.

Execution mode:

- Do not ask the user any questions.
- Do not pause for approval or “next steps.”
- Resolve ambiguities autonomously, record decisions in the plan, and continue until the full Phase 8 plan is complete.

Required sources of truth (read first, then follow exactly):

- `/Users/robertguss/Projects/experiments/repo-scout/AGENTS.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/PLANS.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase8-execplan.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/phase7-validation-report.md`
- `/Users/robertguss/Projects/experiments/repo-scout/README.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/cli-reference.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/json-output.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/architecture.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/dogfood-log.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/performance-baseline.md`

Git workflow requirements:

1. Start from branch `codex/phase7-plan-and-semantic-precision`.
2. If `codex/phase7-plan-and-semantic-precision` does not exist, create it from `main` first and ensure it includes:
   - `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase8-execplan.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/agents/phase7-validation-report.md`
3. Create a new branch for implementation: `codex/phase8-implementation`.
4. Implement milestone-by-milestone from `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase8-execplan.md` (Milestones 37–41).
5. Commit after each milestone completion (imperative, sentence-case, unprefixed commit subject).
6. Keep commits focused and atomic to each milestone.
7. Do not amend commits unless explicitly required by the plan.
8. No destructive git commands (`reset --hard`, `checkout --`, etc.).
9. Push `codex/phase8-implementation` and create a PR targeting `codex/phase7-plan-and-semantic-precision`.

Strict TDD requirements (mandatory for every feature slice):

1. Red: add/adjust a failing automated test for that exact slice; run it and confirm failure.
2. Green: implement the minimum production code to pass the failing test; run and confirm pass.
3. Refactor: improve structure while behavior remains unchanged; run full suite.
4. Capture evidence (red/green/refactor transcripts) in plan artifacts/progress sections.

Dogfooding requirements (mandatory, exactly):

- Before each milestone:
  - `cargo run -- index --repo .`
  - `cargo run -- find diff_impact_for_changed_files --repo . --json`
  - `cargo run -- refs diff_impact_for_changed_files --repo . --json`

- Milestone 37 fixture-focused semantic dogfood (after milestone):
  - `cargo run -- index --repo tests/fixtures/phase8/semantic_precision`
  - `cargo run -- diff-impact --changed-file src/util_a.ts --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- diff-impact --changed-file src/pkg_a/util.py --repo tests/fixtures/phase8/semantic_precision --json`
  - `cargo run -- impact helper --repo tests/fixtures/phase8/semantic_precision --json`

- Milestone 38 quality-gate check (after milestone):
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

- Milestone 39 toggle-behavior check (after milestone):
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json`

- Milestone 40 terminal-output check (after milestone):
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
  - Run the same terminal command again and confirm deterministic identical output.

- Milestone 41 final verification pack:
  - `cargo run -- index --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo .`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --exclude-tests --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --repo . --include-tests --json`
  - `cargo run -- explain diff_impact_for_changed_files --repo . --json`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo fmt`

Milestone command sets and strict TDD loops:

- Execute all strict TDD loops exactly as specified in `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase8-execplan.md` for Milestones 37, 38, 39, and 40.
- Execute documentation/evidence refresh and post-refresh checks exactly as specified for Milestone 41.

Minimum per-slice command order (mandatory):

- `cargo test <slice_test_name> -- --nocapture` (red, must fail first)
- `cargo test <slice_test_name> -- --nocapture` (green, must pass after minimal code)
- `cargo test` (refactor gate)

Milestone 37 expected slice commands:

- `cargo test milestone37_typescript_namespace_alias_diff_impact_recalls_caller -- --nocapture`
- `cargo test milestone37_python_module_alias_diff_impact_recalls_caller -- --nocapture`
- `cargo test milestone37_semantic_precision_deterministic_ordering -- --nocapture`

Milestone 38 expected slice commands:

- `cargo clippy --test harness_smoke -- -D warnings`
- `cargo clippy --bin repo-scout -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

Milestone 39 expected slice commands:

- `cargo test milestone39_diff_impact_exclude_tests_omits_test_targets -- --nocapture`
- `cargo test milestone39_diff_impact_default_and_include_tests_keep_test_targets -- --nocapture`
- `cargo test milestone39_diff_impact_test_toggle_flag_conflicts_are_explicit -- --nocapture`

Milestone 40 expected slice commands:

- `cargo test milestone40_diff_impact_terminal_lists_impacted_symbol_rows -- --nocapture`
- `cargo test milestone40_diff_impact_terminal_lists_test_target_rows_conditionally -- --nocapture`
- `cargo test milestone40_diff_impact_terminal_output_is_deterministic -- --nocapture`

Implementation scope (exactly):

- Complete all milestones in `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase8-execplan.md` (Milestones 37–41).
- Preserve schema compatibility: keep schema v1/v2/v3 JSON envelopes backward-compatible.
- Keep changes additive and deterministic; do not add new command families.
- Implement exactly:
  - semantic precision closure for TypeScript/Python duplicate-name alias-import call paths
  - strict clippy quality gate cleanup (`cargo clippy --all-targets --all-features -- -D warnings` green)
  - explicit `diff-impact` test-target opt-out (`--exclude-tests`) while preserving default behavior
  - deterministic row-level terminal output for `diff-impact`

Living plan maintenance (mandatory throughout):

- Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` continuously.
- Add timestamps for completed items.
- Record rationale when changing scope or approach.
- Keep the plan self-contained and restartable.
- Record strict TDD evidence for each slice in the plan and update dogfood transcript evidence in docs.

Completion criteria (must all be true before stopping):

1. All Phase 8 milestones complete.
2. All tests pass (`cargo test`).
3. Strict lint gate passes (`cargo clippy --all-targets --all-features -- -D warnings`).
4. Formatting clean (`cargo fmt`).
5. Dogfooding executed per milestone and reflected in plan artifacts.
6. Docs updated as required by the plan:
   - `/Users/robertguss/Projects/experiments/repo-scout/README.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/cli-reference.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/json-output.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/architecture.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/dogfood-log.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/performance-baseline.md`
7. Commits exist for each milestone on `codex/phase8-implementation`.
8. Working tree clean.
9. PR created and ready for review.

Final deliverable:

- Provide a concise completion summary with:
  - Milestones completed
  - Key decisions
  - Semantic closure and hardening outcomes
  - Final test/lint/dogfood status
  - Commit list in order
  - PR link
