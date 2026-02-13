# Future Phases Roadmap — Post Phase 1

**Date:** 2026-02-13 **Status:** Vision (not committed — revisit after Phase 1 dogfooding)
**Parent:** `docs/plans/refactoring-features-v3.md` **Depends on:**
`docs/plans/2026-02-13-phase1-foundation-design.md`

---

## How to Use This Document

This is a **vision document**, not a sequential implementation plan. After Phase 1 ships and gets
dogfooded, the next phase should emerge from what's actually needed rather than blindly following
this sequence. Re-evaluate priorities at each phase boundary by asking: "What does the agent
actually need right now?"

Features are grouped by theme and dependency, with design review notes from the Phase 1 planning
session included where they apply.

---

## Phase 2: Diagnosis — `anatomy`, `coupling`, `dead`

**Depends on:** Phase 1 (schema v4, line_count populated) **Note:** `circular` was promoted to
Phase 1.

### `anatomy` — single-file structural analysis

**Question answered:** "What's inside this big file, and where are the natural seams if I wanted to
split it?"

**Algorithm:**

1. Build intra-file call graph from `symbol_edges_v2` where both source and target are in the same
   file
2. Find connected components (clusters) using union-find
3. For each cluster, identify entry point (most inbound edges from outside the cluster)
4. Report clusters, unclustered symbols, and suggested splits

**Flags:** `<file>` (positional), `--clusters`, `--suggest-split`, `--json`, `--repo`

**Design review note — cohesion scoring deferred:** Ship clusters (connected components) and
suggested splits in v1. Do NOT ship numeric cohesion scores yet. The density metric (actual_edges /
possible_edges) is too noisy for small clusters and would imply precision the algorithm doesn't
support. Revisit scoring after dogfooding cluster output on real codebases.

**Design review note — cluster naming:** Needs a naming algorithm before implementation. Options:

- Most-connected function's name as cluster label
- Common prefix of symbol names in the cluster
- Manual: just "Cluster 1", "Cluster 2" with the entry point listed

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

### `coupling` — file-pair entanglement analysis

**Question answered:** "Which files are tangled together in ways that make them hard to change
independently?"

**How it differs from `deps`:** `deps` shows dependencies for a single file. `coupling` analyzes all
file pairs and identifies the ones with the highest bidirectional edge count.

**Algorithm:**

1. Query `symbol_edges_v2` joined with `symbols_v2` on both sides
2. Group by (source_file, target_file), count edges per direction
3. Classify: asymmetric (clean dependency) vs symmetric (entanglement)
4. Sort by total bidirectional count descending

**Flags:** `--threshold N` (default 3), `--symmetric-only`, `--top N` (default 20), `--json`,
`--repo`

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

### `dead` — find unreferenced symbols

**Question answered:** "What code is never used and can be safely removed?"

**Algorithm:**

1. LEFT JOIN `symbols_v2` against `ast_references` and `symbol_edges_v2` (incoming edges)
2. Filter for NULL (no references)
3. Assign confidence: High (private, no refs), Medium (public, no refs), Low (text refs only)

**Flags:** `--confidence <high|medium|low>` (default high), `--include-public`, `--include-tests`,
`--json`, `--repo`

**Design review note — scope tightly:** Dead code detection generates false positives that erode
trust fast. Known blind spots:

- `pub` for testing vs `pub` for external use (Rust)
- Conditional compilation (`#[cfg(...)]`) hides usages
- Macros reference symbols invisibly to tree-sitter
- Trait implementations have zero direct callers but are not dead

**Recommendation:** Ship v1 as **high confidence only, private symbols only**. Expand to Medium/Low
tiers only after validating false positive rate on real codebases. Do not ship `--include-public` in
v1.

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

## Phase 3: Test Intelligence — `test-gaps`, `test-quality`

**Depends on:** Phase 1 (schema v4, line_count) **Independent of:** Phase 2

### `test-gaps` — pre-refactoring safety gate

**Question answered:** "Is this file/symbol tested well enough that I can refactor it safely?"

The single most important pre-refactoring check. Makes risk explicit before the agent gambles on
untested code.

**Algorithm:**

1. Query all symbols in target file from `symbols_v2`
2. For each, use existing `tests-for` logic — does any test file reference it?
3. Classify: covered vs uncovered
4. Prioritize uncovered by risk: public + many callers + large = high risk
5. Report coverage ratio and prioritized gap list

**Flags:** `<file>` (positional), `--symbol <name>`, `--min-risk <high|medium|low>`, `--json`,
`--repo`

**Note:** Risk prioritization requires `visibility` column to be populated. If not yet populated,
degrade gracefully (skip visibility-based risk classification, use size + caller count only).

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

### `test-quality` — are existing tests healthy?

**Question answered:** "Can I trust the tests that do exist?"

**Algorithm:**

1. Find all test functions (kind=function in files matching test patterns)
2. Flag: very long tests (>100 lines), tests not referencing their likely target, symbols with only
   one test, test files with very few `assert` occurrences
3. Report flagged tests with reasons

**Flags:** `<file>` (positional), `--json`, `--repo`

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

## Phase 4: Refactoring Intelligence — `boundary`, `suggest`

**Depends on:** Phase 1 (schema v4), benefits from Phases 2 + 3

### `boundary` — public API surface of a file

**Question answered:** "What's the public interface of this file, and how much can I safely
restructure internally?"

**Algorithm:**

1. Query `symbols_v2` for target file
2. Classify by visibility (from `visibility` column, or inferred from AST patterns)
3. For public symbols, count external references from other files
4. Report: public API surface, internal symbols, safe-to-restructure percentage

**Requires:** `visibility` column populated. This is the forcing function for implementing
visibility detection in language adapters.

**Design review note — visibility spec needed:** Before implementing, define per-language visibility
values:

- **Rust:** `pub`, `pub(crate)`, `pub(super)`, `private`
- **TypeScript:** `export`, `export_default`, `internal` — what about re-exports via barrel files?
- **Python:** `public` (no underscore), `private` (leading underscore) — `__all__` handling?
- **Go:** `public` (capitalized), `private` (lowercase) — internal packages?

**Flags:** `<file>` (positional), `--public-only`, `--json`, `--repo`

**Files:** `src/query/planning.rs` (NEW), `src/cli.rs`, `src/output.rs`, `src/main.rs`, language
adapters for visibility population

---

### `suggest` — prioritized refactoring targets

**Question answered:** "What should I refactor first?"

**Design review note — reframe as sort, not score:** Rather than computing a weighted
"refactoring_value" score (which is hard to interpret), present as a ranked list: "large functions
sorted by risk." Show the signals that contributed (size, fan-in, test coverage) but don't combine
them into a single number.

**Algorithm:**

1. For each function above minimum size (20 lines): gather size, fan-in, test coverage status
2. If anatomy/coupling data available, include those signals
3. Sort by size \* fan-in (simple, interpretable)
4. Classify: SAFE (has tests) vs RISKY (no tests)
5. Report with contributing signals shown per entry

**Flags:** `--top N` (default 10), `--safe-only`, `--json`, `--repo`

**Files:** `src/query/diagnostics.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

## Phase 5: Refactoring Pre-flight — `extract-check`, `move-check`, `rename-check`, `split-check`

**Depends on:** Phase 4 (`boundary`), Phase 2 (`anatomy` for split-check)

### `extract-check` — pre-flight for function extraction

**Question answered:** "If I extract lines X-Y into a new function, what will the signature look
like?"

**This requires variable-flow analysis** — the most technically ambitious feature in the entire
plan. Each language adapter gains `analyze_extraction()` method.

**Design review note — ship Rust-only first:** Build for Rust using `syn` (excellent for this), ship
as v1, add other languages incrementally. Don't gate other Phase 5 commands on all-language
extract-check.

**Variable-flow analysis per language:**

- **Rust:** Walk function body via `syn`. Track `let` bindings, parameters, pattern bindings.
- **TypeScript:** Walk via tree-sitter. Track `const`/`let`/`var`, destructuring, parameters.
- **Python:** Walk via tree-sitter. Track assignments, parameters, `for`/`with` targets.
- **Go:** Walk via tree-sitter. Track `:=`, `var`, parameters, range variables.

**Documented limitations:** Cannot track closure captures crossing boundary, cannot resolve type
aliases, best-effort type annotations, does not handle mutations.

**Flags:** `<symbol>` (positional), `--lines <start>-<end>` (required), `--json`, `--repo`

**Files:** `src/query/planning.rs`, `src/cli.rs`, `src/output.rs`, `src/main.rs`,
`src/indexer/languages/mod.rs` (trait extension), all 4 language adapters

---

### `move-check` — pre-flight for moving a symbol

**Question answered:** "If I move this symbol to a different file, what breaks?"

**Composes:** existing `refs` + `impact` + `tests-for` + `boundary`

**Output:** Dependencies that must come with it, call sites needing updates (file:line), tests that
reference it, whether the move changes the public API boundary.

**Flags:** `<symbol>` (positional), `--to <path>`, `--json`, `--repo`

---

### `rename-check` — preview all rename impacts

**Question answered:** "If I rename this, what needs to change beyond what the compiler catches?"

More comprehensive than `refs`: includes AST references, text occurrences in strings/comments/docs,
re-exports, and derived names (test names containing the symbol name).

**Flags:** `<symbol>` (positional), `--to <new_name>`, `--json`, `--repo`

---

### `split-check` — pre-flight for file splitting

**Question answered:** "If I split this file into these groups, what cross-references would I
create?"

**Composes:** `anatomy` clusters + `boundary` + `circular`

**Algorithm:**

1. Accept proposed groupings (or `--auto` to use `anatomy` cluster suggestions)
2. For each group: compute needed imports, cross-group references, new circular deps
3. Score: does it improve cohesion?

**Flags:** `<file>` (positional), `--groups "<g1>:<g2>"`, `--auto`, `--json`, `--repo`

**Files (for move/rename/split-check):** `src/query/planning.rs`, `src/cli.rs`, `src/output.rs`,
`src/main.rs`

---

## Phase 6: Refactoring Support — `test-scaffold`, `safe-steps`

**Depends on:** Phase 5

**Design review note:** These commands generate structured advice (step-by-step plans, suggested
test cases). Consider whether the underlying data from extract-check/move-check is rich enough that
the AI agent can derive these steps itself. If so, these commands may be unnecessary — the agent's
job is judgment and planning; the tool's job is data.

**Decision: revisit after Phase 5 dogfooding.** If agents consistently produce good plans from Phase
5 output alone, skip these commands.

### `test-scaffold` — structured test setup information

**Question answered:** "I need to write a test for this symbol. What do I need to know?"

Not a test generator. Provides context: signature, dependencies, existing test conventions,
suggested test file location, suggested test cases.

**Flags:** `<symbol>` (positional), `--json`, `--repo`

### `safe-steps` — decompose refactoring into safe increments

**Question answered:** "What's the safest order of operations for this refactoring?"

Applies refactoring-type-specific templates populated with data from planning commands.

**Flags:** `<symbol>` (positional), `--action <extract|move|rename|split>`, `--lines <range>`,
`--to <path|name>`, `--json`, `--repo`

---

## Phase 7: Verification — `verify-refactor`, `health --diff`

**Depends on:** Phase 1 only (independent of Phases 2-6) **Note:** Can be built any time after
Phase 1. The most differentiated feature in the plan.

### `verify-refactor` — post-refactoring completeness check

**Question answered:** "I just finished refactoring. Did I miss anything?"

**Algorithm:**

1. Accept `--before <commit>` and `--after <commit>` (or working tree)
2. Index at both points (or use saved baseline)
3. Compare: disappeared symbols with remaining references (incomplete move/delete), new unresolved
   references (renamed but not all refs updated), edge changes, lost test associations, new circular
   dependencies
4. Report discrepancies as warnings

**Design review note — consider building earlier:** This only depends on Phase 1. A simplified
version (diff two index states, flag orphaned references) could ship as early as Phase 2. Full
version adds cycle detection comparison and test association checking.

**Flags:** `--before <commit|baseline>`, `--after <commit>` (default working tree), `--strict`,
`--json`, `--repo`

**Files:** `src/query/verification.rs` (NEW), `src/cli.rs`, `src/output.rs`, `src/main.rs`

---

### `health --diff` — baseline comparison

**Question answered:** "Did my refactoring actually improve the codebase?"

Extends Phase 1's `health` command:

- `health --save-baseline` saves metrics
- `health --diff` compares current against baseline
- Shows improvements and regressions

**Design review note — store baselines in the database:** Use a `health_baselines` table in the
index DB rather than `.repo-scout/health-baseline.json`. Avoids file management complexity, keeps
everything atomic.

**Files:** Extend `HealthArgs`, `src/query/diagnostics.rs`, `src/output.rs`

---

## Phase 8: Enhance Existing Commands

**Depends on:** Phases 2-7 (uses infrastructure from earlier phases)

### `hotspots` enhancement

- Add `line_count` annotation to each hotspot entry
- Add `--annotate` flag for risk scoring (fan-in \* line_count = change risk)
- No breaking changes to existing output

### `orient` enhancement

- Include `suggest` top-3 in recommendations (if suggest data available)
- Phase 1 already includes circular in orient

### `deps` enhancement

- Add `--coupling` flag that includes directionality scoring (reuses `coupling` logic)
- Default behavior unchanged

---

## Dependency Graph

```
Phase 1 (DONE — in implementation plan)
  schema v4, tree, health, circular, orient
    │
    ├──→ Phase 2: anatomy, coupling, dead
    │
    ├──→ Phase 3: test-gaps, test-quality
    │
    ├──→ Phase 7: verify-refactor, health --diff  ← CAN BUILD EARLY
    │
    ├──→ Phase 4: boundary, suggest  (benefits from 2+3)
    │       │
    │       └──→ Phase 5: extract-check, move-check, rename-check, split-check
    │               │
    │               └──→ Phase 6: test-scaffold, safe-steps  (may be unnecessary)
    │
    └──→ Phase 8: hotspots/orient/deps enhancements  (after relevant phases)
```

**Key insight:** Phases 2, 3, and 7 are all independent and can be built in any order after Phase 1.
The critical path to the most ambitious features is: Phase 1 → Phase 4 (boundary) → Phase 5
(extract-check).

---

## Features That May Not Survive Dogfooding

These features are included for completeness but have been flagged during design review as
potentially unnecessary:

| Feature                 | Concern                                                                                                 | Revisit Criteria                                                              |
| ----------------------- | ------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `suggest`               | Combining signals into a ranking may not be more useful than just reading `health` + `test-gaps` output | After Phase 3: do agents derive priorities naturally from health + test-gaps? |
| `test-scaffold`         | Agents already know how to derive test cases from signatures and existing conventions                   | After Phase 5: do agents struggle to write tests given Phase 5 data?          |
| `safe-steps`            | Agents already know how to plan refactoring steps given the right data                                  | After Phase 5: do agents produce bad step ordering given Phase 5 data?        |
| `dead` Medium/Low tiers | False positive rate on public/text-only symbols may be too high to trust                                | After Phase 2: what's the false positive rate on real codebases?              |

---

## Schema Considerations for Future Phases

All schema changes were front-loaded into the v4 migration (Phase 1). Future phases should NOT
require additional migrations unless:

- A new table is needed (e.g., `health_baselines` for Phase 7)
- The `visibility` population strategy requires schema adjustments after per-language spec review

If a new migration is needed, bump to v5 and follow the existing pattern.

---

## Visibility Specification (Blocking Phase 4)

Before implementing `boundary`, define the exact visibility values per language:

| Language   | Values                                     | Notes                                          |
| ---------- | ------------------------------------------ | ---------------------------------------------- |
| Rust       | `pub`, `pub_crate`, `pub_super`, `private` | Straightforward from AST                       |
| TypeScript | `export`, `export_default`, `internal`     | Re-exports via barrel files?                   |
| Python     | `public`, `private`                        | Leading underscore convention only? `__all__`? |
| Go         | `public`, `private`                        | Capitalization-based. Internal packages?       |

This spec must be written before Phase 4 implementation begins.

---

## Performance Considerations

Some future queries could be expensive on large repos. Before implementing, establish:

- **Query budget:** Maximum acceptable latency per command (suggestion: 5s for single-file commands,
  30s for whole-repo analysis)
- **Caching strategy:** Consider materialized views or cached results for expensive cross-joins
  (coupling, anatomy clustering)
- **Incremental analysis:** Most commands assume full index. Consider whether changed-files-only
  analysis is needed for large repos

---

## Future Considerations (Not in Any Phase)

These ideas from v3 are captured here but have no timeline:

- **Complexity metrics population:** Populate `param_count`, `nesting_depth`, `branch_count`,
  `complexity_score` columns (schema already prepared, no migration needed)
- **Duplication detection:** Structural duplication (similar AST shapes). High value, high cost.
- **Watch mode:** `repo-scout watch` — re-index on file changes
- **IDE/editor integration:** LSP-like integration for editor UI
