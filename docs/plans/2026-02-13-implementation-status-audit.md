# Implementation Status Audit — Plans Folder

Date: 2026-02-13
Scope: `docs/plans/*`
Method: cross-check plan claims against current CLI surface, source code, milestone tests, and `repo-scout` queries (`index/find/refs`).

## Summary

- Implemented: core Foundation Phase 1 (`tree`, `health`, `circular`, `orient` + schema v4 + query module split)
- Mostly implemented: dogfood roadmap tasks (19/20)
- Partial: status staleness enrichment
- Remaining: future/refactoring phases (`anatomy`, `coupling`, `dead`, `test-gaps`, `suggest`, `boundary`, preflight/refactor-support/verification commands)

---

## 1) `2026-02-13-phase1-foundation-design.md`

Status: `Mostly Implemented`

### Done

- [x] Schema v4 migration and reserved columns
  - Evidence: `src/store/schema.rs` (`SCHEMA_VERSION = 4`; `line_count`, `visibility`, `param_count`, `nesting_depth`, `branch_count`, `complexity_score` migrations)
- [x] Query module split for new foundation work
  - Evidence: `src/query/orientation.rs`, `src/query/diagnostics.rs`, `src/query/mod.rs`
- [x] `tree` command
  - Evidence: `src/cli.rs`, `src/main.rs`, `src/query/orientation.rs`, `tests/milestone91_tree.rs`
- [x] `health` command (largest files/functions)
  - Evidence: `src/cli.rs`, `src/main.rs`, `src/query/diagnostics.rs`, `tests/milestone92_health.rs`
- [x] `circular` command
  - Evidence: `src/cli.rs`, `src/main.rs`, `src/query/diagnostics.rs`, `tests/milestone93_circular.rs`
- [x] `orient` command
  - Evidence: `src/cli.rs`, `src/main.rs`, `src/query/orientation.rs`, `tests/milestone94_orient.rs`

### Deferred / Partial (explicitly noted by plan)

- [ ] `health --save-baseline` / `health --diff`
- [ ] health code-marker scanning (TODO/FIXME/HACK)
- [ ] health symbol-density section

(These are explicitly deferred in the plan itself.)

---

## 2) `2026-02-13-dogfood-roadmap.md`

Status: `Mostly Implemented (19/20 complete, 1 partial)`

### Phase 1: Bug Fix

- [x] Task 1: `explain --include-snippets` terminal rendering
  - Evidence: `tests/milestone82_explain_snippets.rs`, snippet output path in `src/output.rs`

### Phase 2: Quick Wins

- [x] Task 2: `--help` subcommand descriptions
  - Evidence: `src/cli.rs` about strings; `tests/milestone82_help_text.rs`
- [x] Task 3: `index` output uses `non_source_files`
  - Evidence: `src/output.rs` (`print_index_summary`)
- [~] Task 4: `status` enrichment
  - Done: counts + languages
  - Missing: staleness indicator
  - Evidence: `src/query/mod.rs` (`StatusSummary`), `src/output.rs` (`print_status`), `tests/milestone82_status.rs`
- [x] Task 5: `refs --code-only` (exists)
  - Evidence: `src/cli.rs`, `src/main.rs`
- [x] Task 6: `diff-impact --max-results` default 30
  - Evidence: `src/cli.rs`

### Phase 3: New Commands (Medium)

- [x] Task 7: `snippet`
- [x] Task 8: `outline`
- [x] Task 9: `summary`

Evidence: CLI/main/output/query wiring plus `tests/milestone82_snippet.rs`, `tests/milestone82_outline.rs`, `tests/milestone82_summary.rs`.

### Phase 4: New Commands (Higher)

- [x] Task 10: `--since` on `diff-impact` and `verify-plan`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_since.rs`, `tests/milestone82_git_flags.rs`
- [x] Task 11: `callers`, `callees`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_callers_callees.rs`
- [x] Task 12: `deps`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_deps.rs`
- [x] Task 13: `hotspots`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_hotspots.rs`

### Phase 5: Scoring & Quality

- [x] Task 14: context relevance improvements
  - Evidence: `tests/milestone24_context_relevance.rs`
- [x] Task 15: tests-for recall improvements
  - Evidence: `tests/milestone22_recommendation_quality.rs`, `tests/milestone82_tests_for_recall.rs`
- [x] Task 16: grouped refs output
  - Evidence: `src/output.rs` (`print_refs_grouped`), `src/main.rs` refs path

### Phase 6: Stretch

- [x] Task 17: call path discovery (implemented as `call-path`)
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_path.rs`
- [x] Task 18: `related`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_related.rs`
- [x] Task 19: `--compact`
  - Evidence: `src/cli.rs`, `src/main.rs`, `tests/milestone82_compact.rs`
- [x] Task 20: fuzzy/did-you-mean for `find`
  - Evidence: `src/query/mod.rs` (`suggest_similar_symbols`), `src/main.rs`, `tests/milestone82_fuzzy.rs`, `tests/milestone82_did_you_mean.rs`

---

## 3) `2026-02-13-future-phases-roadmap.md`

Status: `Remaining (Vision document)`

Document declares itself as vision/not committed.

### Remaining command families

- [ ] Phase 2: `anatomy`, `coupling`, `dead`
- [ ] Phase 3: `test-gaps`, `test-quality`
- [ ] Phase 4: `boundary`, `suggest`
- [ ] Phase 5: `extract-check`, `move-check`, `rename-check`, `split-check`
- [ ] Phase 6: `test-scaffold`, `safe-steps`
- [ ] Phase 7: `verify-refactor`, `health --diff`
- [ ] Phase 8 enhancements dependent on above

CLI evidence of non-implementation: these commands are absent from `src/cli.rs` and no `run_*` handlers exist in `src/main.rs` for these names.

---

## 4) `refactoring-features.md` / `refactoring-features-v2.md` / `refactoring-features-v3.md`

Status: `Partially Implemented (foundation and several adjacent commands); majority remaining`

### Implemented portions

- [x] Foundation orientation/diagnostic substrate from v3 Phase 1 (`tree`, `health`, `circular`, `orient`, schema v4)
- [x] Some adjacent commands in broader plan family already exist: `deps`, `hotspots`, `call-path`, `related`, `summary`, `outline`, `snippet`

### Remaining planned capabilities

- [ ] `anatomy`
- [ ] `coupling`
- [ ] `dead`
- [ ] `test-gaps`
- [ ] `test-quality`
- [ ] `suggest`
- [ ] `boundary`
- [ ] `extract-check`
- [ ] `move-check`
- [ ] `rename-check`
- [ ] `split-check`
- [ ] `test-scaffold`
- [ ] `safe-steps`
- [ ] `verify-refactor`
- [ ] `health --diff` / baseline persistence

---

## Repo-Scout Verification Notes

Representative `repo-scout` checks run during this audit:

- `cargo run -- index --repo .`
- `cargo run -- find tree_report --repo . --json --code-only --exclude-tests --max-results 5`
- `cargo run -- refs tree_report --repo . --json --code-only --exclude-tests --max-results 5`
- `cargo run -- find run_anatomy --repo . --json --code-only --exclude-tests --max-results 3` (no results)
- `cargo run -- find run_coupling --repo . --json --code-only --exclude-tests --max-results 3` (no results)
- `cargo run -- find run_verify_refactor --repo . --json --code-only --exclude-tests --max-results 3` (no results)

Interpretation: foundation + dogfood command surface is implemented; future/refactoring phases remain open.

---

## Recommended Next Slice

1. Complete remaining partial from dogfood roadmap: `status` staleness signal.
2. Start future roadmap Phase 2 with one command (`anatomy`) as a vertical slice (CLI + query + output + tests).
3. Defer `health --diff` until after a stable baseline format decision.
