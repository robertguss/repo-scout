# Repo-Scout Refactoring Features — Implementation Plan

## Context

Repo-scout is a Rust CLI that indexes codebases into SQLite and answers code-navigation queries. It
already has `find`, `refs`, `impact`, `tests-for`, `diff-impact`, etc. This plan adds a suite of
**refactoring-focused features** — analysis tools that help both AI agents and developers identify
refactoring opportunities, understand code health, and execute refactoring safely.

**Two audiences:**

1. **AI agents (Claude)** — structured `--json` output for programmatic refactoring decisions
2. **Developers** — human-readable terminal output, code health dashboard

**Key insight:** `symbols_v2` already stores `start_line`/`end_line` for every symbol, so function
line counts are computable today with zero schema changes. Phases are ordered to deliver value
early.

---

## Phase 1: `health` command (no schema changes)

**What:** Whole-repo code health dashboard using existing indexed data.

**Output includes:**

- Largest files by line count (from `indexed_files` + filesystem or symbol aggregation)
- Largest functions by line count (`end_line - start_line + 1` from `symbols_v2` where kind =
  function)
- Files with the most symbols (symbol density)
- TODO/FIXME/HACK counts (from `text_occurrences` — index these tokens)

**Files to modify:**

- `src/cli.rs` — add `Health(HealthArgs)` command with flags: `--top N`, `--threshold N`,
  `--large-files`, `--large-functions`, `--json`, `--repo`
- `src/query/mod.rs` — add `health_report()` query function, `HealthReport` result struct with
  sub-sections (largest files, largest functions, etc.)
- `src/output.rs` — add `print_health()` and `print_health_json()`
- `src/main.rs` — add `run_health()` handler

**Key design decisions:**

- Opinionated defaults: top 20 results per category, sorted by size desc
- File line counts: count from `symbols_v2` aggregate OR read file line counts during indexing
  (prefer storing in `indexed_files` — add `line_count INTEGER` column, schema v4)
- JSON schema version: v4 for new commands

**Schema change (minimal):**

- Add `line_count INTEGER` column to `indexed_files` table (populated during indexing)
- Schema version bump to v4

---

## Phase 2: `anatomy` command (no additional schema changes)

**What:** Structural map of a single file — all its symbols, their sizes, and relationships.

**Output includes:**

- All functions/structs/types in the file with line counts, sorted by size
- Which functions call which (from `symbol_edges_v2` where both symbols are in the same file)
- Cluster detection: groups of functions that call each other (connected components in the
  intra-file call graph)
- Suggested split points based on clusters

**Files to modify:**

- `src/cli.rs` — add `Anatomy(AnatomyArgs)` with `--file`, `--json`, `--repo`
- `src/query/mod.rs` — add `file_anatomy()` query, `AnatomyReport` struct with `FileSymbol` entries
  and `Cluster` groups
- `src/output.rs` — add `print_anatomy()` and `print_anatomy_json()`
- `src/main.rs` — add `run_anatomy()` handler

**Cluster algorithm:** Simple connected-components on the intra-file call subgraph. Group functions
that transitively call each other. Utility functions (called by multiple clusters) form a separate
"shared" group.

---

## Phase 3: Complexity metrics (schema extension)

**What:** Add structural complexity data to the index, computed during AST extraction.

**New columns on `symbols_v2`:**

- `line_count INTEGER` — `end_line - start_line + 1` (precomputed for query speed)
- `param_count INTEGER` — number of parameters (parsed from signature or AST)
- `nesting_depth INTEGER` — max nesting depth within function body
- `branch_count INTEGER` — number of if/match/else/for/while nodes in body
- `visibility TEXT` — "public", "private", "crate" (language-dependent)

**Schema migration:** v4 → v5 (or combine with Phase 1's v4 if implemented together)

**Files to modify:**

- `src/store/schema.rs` — migration adding columns, schema version bump
- `src/indexer/languages/rust.rs` — extract param_count, nesting_depth, branch_count, visibility
- `src/indexer/languages/typescript.rs` — same
- `src/indexer/languages/python.rs` — same
- `src/indexer/languages/go.rs` — same
- `src/indexer/mod.rs` — pass new fields through to symbol insertion
- `ExtractedSymbol` struct — add new optional fields

**Complexity computation (per language adapter):**

- **param_count**: count `parameter` / `typed_parameter` children of the parameters node
- **nesting_depth**: walk function body, track depth for `if_expression`, `match_expression`,
  `for_expression`, `while_expression`, `loop_expression`, record max
- **branch_count**: count all branching nodes within function body
- **visibility**: check for `visibility_modifier` node (Rust), `export` keyword (TS), leading `_`
  convention (Python), uppercase first letter (Go)

**Enhance `health` and `anatomy` commands** to include complexity metrics in output once available.

---

## Phase 4: `dead` command (no schema changes)

**What:** Find symbols that are defined but never referenced anywhere.

**Algorithm:**

1. Query all `symbols_v2` definitions (functions, structs, enums, traits, types, consts)
2. For each, check if any row in `ast_references` or `text_occurrences` references it
3. Also check `symbol_edges_v2` — if a symbol has no incoming edges (no `to_symbol_id` matches), it
   may be dead
4. Filter out known entry points: `main`, `#[test]` functions, `pub` items in library crates
   (configurable)
5. Report unreferenced symbols sorted by file, then line

**Files to modify:**

- `src/cli.rs` — add `Dead(DeadArgs)` with `--include-public`, `--include-tests`, `--json`, `--repo`
- `src/query/mod.rs` — add `dead_symbols()` query, `DeadSymbol` result struct
- `src/output.rs` — add `print_dead()` and `print_dead_json()`
- `src/main.rs` — add `run_dead()` handler

**SQL approach:** LEFT JOIN `symbols_v2` against `ast_references` and `symbol_edges_v2` (incoming),
filter for NULL (no references).

---

## Phase 5: `deps` command (no schema changes)

**What:** Show what a specific function depends on — its upstream dependencies.

**Output includes:**

- Parameters (parsed from signature)
- Functions it calls (outgoing `calls` edges in `symbol_edges_v2`)
- Types it uses (outgoing `imports` edges + types referenced in body)
- Modules it reaches into (derived from file paths of dependencies)

**Algorithm:**

1. Find the target symbol in `symbols_v2`
2. Query `symbol_edges_v2` for outgoing edges (`from_symbol_id = target`)
3. Join with `symbols_v2` to get dependency metadata
4. Group by category (calls, imports, types)
5. Parse signature for parameter info

**Files to modify:**

- `src/cli.rs` — add `Deps(DepsArgs)` with symbol, `--json`, `--repo`
- `src/query/mod.rs` — add `symbol_deps()` query, `DepsReport` struct
- `src/output.rs` — add `print_deps()` and `print_deps_json()`
- `src/main.rs` — add `run_deps()` handler

---

## Phase 6: `coupling` command (no schema changes)

**What:** Show cross-file reference density — which files are most tangled together.

**Algorithm:**

1. Query `symbol_edges_v2` joined with `symbols_v2` on both sides
2. Group edges by (source_file, target_file) where source_file != target_file
3. Count edges per file pair
4. Sort by count descending

**Output:** File pairs with cross-reference counts, optionally filtered by threshold.

**Files to modify:**

- `src/cli.rs` — add `Coupling(CouplingArgs)` with `--threshold N`, `--top N`, `--json`, `--repo`
- `src/query/mod.rs` — add `file_coupling()` query, `CouplingPair` result struct
- `src/output.rs` — add `print_coupling()` and `print_coupling_json()`
- `src/main.rs` — add `run_coupling()` handler

---

## Phase 7: `move-check` command (no schema changes)

**What:** Pre-flight checklist for moving a symbol to a new location.

**Algorithm (composes existing queries):**

1. Run `deps` on the target symbol → what must come with it or be imported
2. Run `refs` on the target symbol → what call sites need updating
3. Run `tests-for` on the target symbol → what tests are affected
4. Report all three in a structured checklist

**Files to modify:**

- `src/cli.rs` — add `MoveCheck(MoveCheckArgs)` with symbol, `--to <path>`, `--json`, `--repo`
- `src/query/mod.rs` — add `move_check()` query that orchestrates deps + refs + tests-for
- `src/output.rs` — add `print_move_check()` and `print_move_check_json()`
- `src/main.rs` — add `run_move_check()` handler

---

## Phase 8: `health --diff` (no schema changes)

**What:** Before/after comparison of health metrics.

**Approach:**

- `health` command saves a snapshot to `.repo-scout/health-baseline.json` with `--save-baseline`
- `health --diff` compares current metrics against the saved baseline
- Shows improvements and regressions

**Files to modify:**

- Extend `HealthArgs` with `--save-baseline` and `--diff` flags
- Add baseline save/load logic in `src/store/mod.rs` or `src/query/mod.rs`
- Add diff rendering in `src/output.rs`

---

## Future (not in this plan): Duplication Detection

Structural duplication detection (functions with similar AST shapes) is high-value but significantly
harder to implement. Would require:

- AST fingerprinting (normalize variable names, compute structural hash)
- Similarity comparison across all function pairs
- Clustering similar functions

Deferred to a future plan after the above phases ship and we learn from usage.

---

## Implementation Order & Dependencies

```
Phase 1: health          ← start here, immediate value, small schema change
Phase 2: anatomy         ← builds on same data, no new schema
Phase 3: complexity      ← schema extension, enhances Phase 1 & 2
Phase 4: dead            ← independent, uses existing data
Phase 5: deps            ← independent, uses existing edges
Phase 6: coupling        ← independent, uses existing edges
Phase 7: move-check      ← depends on Phase 5 (deps)
Phase 8: health --diff   ← depends on Phase 1 (health)
```

Phases 4, 5, 6 are independent and can be built in any order or in parallel.

---

## Verification

For each phase:

1. Write failing integration test first (`tests/milestone<N>_<feature>.rs`)
2. Implement minimal code to pass
3. Run `just check` (fmt + clippy + full test suite)
4. Dogfood: `cargo run -- index --repo . && cargo run -- <new-command> --repo .`
5. Verify `--json` output is deterministic (run twice, diff)
6. Verify human-readable output is scannable and useful

---

## Deliverable

When this plan is approved, a standalone markdown document will be created at
`docs/plans/refactoring-features.md` capturing the full plan for implementation in future sessions.
