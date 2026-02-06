# Dogfood Log

This log captures real usage of `repo-scout` while building `repo-scout`.
Each entry should describe what task we attempted, what worked, what failed, and what follow-up action we took (especially new failing tests and fixes).

## Entry Template

Copy this template for each new task:

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
  - docs/plan update:
- Status: `open` | `fixed` | `deferred`

## Entries

- Date: `2026-02-06`
- Task: Verify index freshness when files are removed.
- Commands run:
  - `cargo run -- index --repo <tmp-repo>`
  - `cargo run -- find alpha --repo <tmp-repo>`
  - delete `a.txt`
  - `cargo run -- index --repo <tmp-repo>`
  - `cargo run -- find alpha --repo <tmp-repo>`
- What helped:
  - Query output was deterministic and easy to compare before/after deletion.
- What failed or felt weak:
  - `find` still returned a deleted file (`a.txt`) after reindex.
- Action taken:
  - failing test added: planned for Phase 2 Milestone 6 lifecycle correctness.
  - fix commit: pending.
  - docs/plan update: added lifecycle-pruning milestone in `agents/repo-scout-agent-first-phase2-execplan.md`.
- Status: `open`

- Date: `2026-02-06`
- Task: Ensure planning process itself is dogfooded and strict-TDD by default.
- Commands run:
  - review and update planning docs
- What helped:
  - Existing ExecPlan structure made it straightforward to add per-slice TDD gates.
- What failed or felt weak:
  - Base template (`agents/PLANS.md`) did not explicitly enforce strict red-green-refactor per feature slice.
- Action taken:
  - failing test added: not applicable (process/documentation change).
  - fix commit: pending.
  - docs/plan update: updated `agents/PLANS.md` with strict TDD requirements and updated phase-2 plan accordingly.
- Status: `fixed`
