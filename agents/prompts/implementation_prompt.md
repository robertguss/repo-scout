You are implementing Phase 6 for `repo-scout` in
`/Users/robertguss/Projects/experiments/repo-scout`.

Execution mode:

- Do not ask the user any questions.
- Do not pause for approval or “next steps.”
- Resolve ambiguities autonomously, record decisions in the plan, and continue until the full Phase
  6 plan is complete.

Required sources of truth (read first, then follow exactly):

- `/Users/robertguss/Projects/experiments/repo-scout/AGENTS.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/PLANS.md`
- `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase6-execplan.md`
- `/Users/robertguss/Projects/experiments/repo-scout/README.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/cli-reference.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/json-output.md`
- `/Users/robertguss/Projects/experiments/repo-scout/docs/architecture.md`

Git workflow requirements:

1. Start from branch `codex/phase6-plan-and-change-scope-precision`.
2. If `codex/phase6-plan-and-change-scope-precision` does not exist, create it from `main` first.
3. Create a new branch for implementation: `codex/phase6-implementation`.
4. Implement milestone-by-milestone from
   `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase6-execplan.md`
   (Milestones 27–31).
5. Commit after each milestone completion (imperative, sentence-case, unprefixed commit subject).
6. Keep commits focused and atomic to each milestone.
7. Do not amend commits unless explicitly required by the plan.
8. No destructive git commands (`reset --hard`, `checkout --`, etc.).
9. Push `codex/phase6-implementation` and create a PR targeting
   `codex/phase6-plan-and-change-scope-precision`.

Strict TDD requirements (mandatory for every feature slice):

1. Red: add/adjust a failing automated test for that exact slice; run it and confirm failure.
2. Green: implement the minimum production code to pass the failing test; run and confirm pass.
3. Refactor: improve structure while behavior remains unchanged; run full suite.
4. Capture evidence (red/green/refactor transcripts) in plan artifacts/progress sections.

Dogfooding requirements (mandatory, exactly):

- Before each milestone:
  - `cargo run -- index --repo .`
  - `cargo run -- find verify_plan_for_changed_files --repo . --json`
  - `cargo run -- refs verify_plan_for_changed_files --repo . --json`
- Post-milestone checks:
  - `cargo run -- index --repo .`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --exclude-tests --json`
  - `cargo run -- context --task "update verify plan recommendation quality for changed files and reduce noisy test selection" --repo . --budget 1200 --code-only --exclude-tests --json`
  - `cargo run -- verify-plan --changed-file src/query/mod.rs --changed-line src/query/mod.rs:1094:1165 --changed-symbol verify_plan_for_changed_files --repo . --json`
  - `cargo run -- diff-impact --changed-file src/query/mod.rs --changed-symbol verify_plan_for_changed_files --exclude-changed --max-results 12 --repo . --json`
  - `cargo run -- refs helper --repo . --max-results 10 --json`
  - `cargo test`

Milestone command sets and post-milestone checks:

- Execute all strict TDD loops exactly as specified in
  `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase6-execplan.md` for
  Milestones 27, 28, 29, and 30.
- Execute post-milestone dogfood checks exactly as specified in that plan.

Implementation scope:

- Complete all milestones in
  `/Users/robertguss/Projects/experiments/repo-scout/agents/repo-scout-phase6-execplan.md`
  (Milestones 27–31).
- Preserve schema compatibility: keep schema v1/v2/v3 JSON envelopes backward-compatible.
- Keep changes additive and deterministic; do not add new command families.
- Implement exactly:
  - `context` scope controls: `--exclude-tests`, `--code-only`
  - `verify-plan` change-scope controls: `--changed-line`, repeatable `--changed-symbol`
  - `diff-impact` focused controls: repeatable `--changed-symbol`, `--exclude-changed`,
    `--max-results`
  - `find`/`refs` fallback focus and deterministic caps: `--max-results`, code-first path-class
    tie-breaks at equal fallback score tiers

Living plan maintenance (mandatory throughout):

- Update `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective`
  continuously.
- Add timestamps for completed items.
- Record rationale when changing scope or approach.
- Keep the plan self-contained and restartable.
- Record strict TDD evidence for each slice in the plan and update dogfood transcript evidence in
  docs.

Completion criteria (must all be true before stopping):

1. All Phase 6 milestones complete.
2. All tests pass (`cargo test`).
3. Formatting clean (`cargo fmt`).
4. Dogfooding executed per slice and reflected in plan artifacts.
5. Docs updated as required by the plan:
   - `/Users/robertguss/Projects/experiments/repo-scout/README.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/cli-reference.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/json-output.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/architecture.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/docs/dogfood-log.md`
   - `/Users/robertguss/Projects/experiments/repo-scout/legacy performance baseline doc (removed)`
6. Commits exist for each milestone on `codex/phase6-implementation`.
7. Working tree clean.
8. PR created and ready for review.

Final deliverable:

- Provide a concise completion summary with:
  - Milestones completed
  - Key decisions
  - Change-scope precision and output-focus outcomes
  - Final test/dogfood status
  - Commit list in order
  - PR link
