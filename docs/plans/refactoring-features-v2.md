# Repo-Scout Refactoring & Code Intelligence — Implementation Plan v2

## Context

Repo-scout is a Rust CLI that indexes codebases into SQLite and answers code-navigation queries. It
already has `find`, `refs`, `impact`, `tests-for`, `diff-impact`, etc. This plan adds a
comprehensive suite of **refactoring-focused features** organized into three layers: diagnosis,
planning, and verification.

**Companion to:** `docs/plans/refactoring-features.md` (v1). This document supersedes v1 with
expanded scope based on analysis of what AI agents and developers actually need for safe, confident
refactoring.

**Two audiences:**

1. **AI agents (Claude)** — structured `--json` output for programmatic refactoring decisions
2. **Developers** — human-readable terminal output, code health dashboard

**Design principles:**

- Diagnosis without verification is academic. Verification without tests is reckless.
- Every command should reduce uncertainty for the next command in the workflow.
- Opinionated defaults with transparent scoring. Don't make the consumer do mental synthesis.
- Composability: commands should build on each other, and `--json` output should cross-reference
  using consistent symbol identifiers.

---

## The Refactoring Workflow

This is the end-to-end loop these commands enable:

```
tree              → understand the codebase structure
health            → get the overall health picture
suggest           → identify highest-value refactoring targets
test-gaps         → check if the target is safe to refactor
test-scaffold     → fill coverage gaps efficiently
anatomy           → understand the target's internal structure
extract-check     → plan the specific refactoring
safe-steps        → decompose into safe increments
                  → execute the refactoring
verify-refactor   → confirm completeness
health --diff     → prove improvement
```

---

## Layer 1: Diagnosis (what's wrong?)

### Phase 1: `tree` command (no schema changes)

**What:** Structural map of the entire codebase, annotated with data from the index.

**Why first:** Orientation is the prerequisite for everything else. For AI agents, this is the
single most valuable "first command" — it replaces 15 minutes of exploratory file reading with one
call.

**Output:**

```
src/                           [4 modules, 156 symbols, 162k lines total]
├── main.rs                    [28k lines │ 15 fns, 3 structs │ command dispatch]
├── cli.rs                     [2k lines  │ 1 enum, 12 structs │ clap definitions]
├── output.rs                  [4k lines  │ 20 fns │ terminal + JSON rendering]
├── store/                     [2 files, 18 symbols]
│   ├── mod.rs                 [3k lines  │ 8 fns │ DB bootstrap, recovery]
│   └── schema.rs              [1k lines  │ 2 fns, 5 consts │ DDL, migrations]
├── indexer/                   [6 files, 52 symbols]
│   ├── mod.rs                 [5k lines  │ 12 fns │ coordinator, edge resolution]
│   ├── files.rs               [1k lines  │ 3 fns │ ignore-aware discovery]
│   └── languages/             [4 files, 35 symbols]
│       ├── rust.rs            [3k lines  │ 8 fns]
│       └── ...
└── query/                     [1 file, 48 symbols]
    └── mod.rs                 [121k lines │ 45 fns, 20 structs │ ⚠ largest file]
```

**Flags:**

- `--depth N` — control tree depth (default: 3)
- `--annotate` — symbol counts and line counts (on by default)
- `--deps` — show dependency arrows between modules
- `--focus <path>` — zoom into a subtree with full detail
- `--symbols` — expand to show individual symbols within each file
- `--json` — structured output for AI agents
- `--repo` — repository path

**Files to modify:**

- `src/cli.rs` — add `Tree(TreeArgs)` command
- `src/query/mod.rs` — add `tree_report()` query, `TreeNode` struct
- `src/output.rs` — add `print_tree()` and `print_tree_json()`
- `src/main.rs` — add `run_tree()` handler

**Data sources:** `indexed_files` for file list, `symbols_v2` for symbol counts/line ranges,
filesystem for directory structure. All already available.

---

### Phase 2: `health` command (small schema change)

**What:** Whole-repo code health dashboard using existing indexed data.

**Output includes:**

- Largest files by line count
- Largest functions by line count (`end_line - start_line + 1` from `symbols_v2`)
- Files with the most symbols (symbol density)
- TODO/FIXME/HACK counts (from `text_occurrences`)

**Flags:**

- `--top N` — number of results per category (default: 20)
- `--threshold N` — minimum line count to report
- `--large-files` — only show large files section
- `--large-functions` — only show large functions section
- `--save-baseline` — save current metrics to `.repo-scout/health-baseline.json`
- `--diff` — compare against saved baseline (Phase 12)
- `--json`, `--repo`

**Schema change (minimal):**

- Add `line_count INTEGER` column to `indexed_files` table (populated during indexing)
- Schema version bump to v4

**Files to modify:**

- `src/cli.rs` — add `Health(HealthArgs)`
- `src/query/mod.rs` — add `health_report()`, `HealthReport` struct
- `src/output.rs` — add `print_health()` and `print_health_json()`
- `src/main.rs` — add `run_health()` handler
- `src/store/schema.rs` — migration adding `line_count` to `indexed_files`

---

### Phase 3: `anatomy` command + `cohesion` scoring (no additional schema changes)

**What:** Structural map of a single file with cohesion analysis.

**`anatomy` output includes:**

- All functions/structs/types in the file with line counts, sorted by size
- Intra-file call graph (from `symbol_edges_v2` where both symbols are in the same file)
- Cluster detection: connected components in the intra-file call graph
- Cohesion score per cluster and for the file overall
- Suggested split points based on clusters

**Cohesion scoring algorithm:**

- High cohesion: functions in a cluster call each other frequently and share types
- Low cohesion: functions have few or no intra-file edges, operate on different types
- Score: ratio of actual intra-cluster edges to possible edges (density), weighted by shared type
  usage
- File-level score: weighted average of cluster cohesion scores

**Flags:**

- `--file <path>` — target file (required)
- `--clusters` — only show cluster analysis
- `--cohesion` — only show cohesion scores
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Anatomy(AnatomyArgs)`
- `src/query/mod.rs` — add `file_anatomy()`, `AnatomyReport` struct with `FileSymbol`, `Cluster`,
  and `CohesionScore` types
- `src/output.rs` — add `print_anatomy()` and `print_anatomy_json()`
- `src/main.rs` — add `run_anatomy()` handler

---

### Phase 4: Complexity metrics (schema extension)

**What:** Add structural complexity data to the index, computed during AST extraction.

**New columns on `symbols_v2`:**

- `line_count INTEGER` — `end_line - start_line + 1` (precomputed)
- `param_count INTEGER` — number of parameters
- `nesting_depth INTEGER` — max nesting depth within function body
- `branch_count INTEGER` — number of if/match/else/for/while nodes in body
- `visibility TEXT` — "public", "private", "crate" (language-dependent)
- `complexity_score INTEGER` — composite 0-100 score (see formula below)

**Complexity score formula:**

```
raw = (line_count_norm * 0.3) + (branch_count_norm * 0.3) +
      (nesting_depth_norm * 0.25) + (param_count_norm * 0.15)
complexity_score = clamp(raw * 100, 0, 100)
```

Where each `_norm` is the value divided by a reasonable maximum for that metric (e.g., line_count /
200, branch_count / 30, nesting_depth / 8, param_count / 10, each clamped to 1.0).

Transparent: the formula and weights are documented. Users can disagree with the weights but the raw
metrics are also available.

**Schema migration:** v4 → v5 (or combined with Phase 2's v4)

**Files to modify:**

- `src/store/schema.rs` — migration adding columns, schema version bump
- `src/indexer/languages/rust.rs` — extract metrics
- `src/indexer/languages/typescript.rs` — extract metrics
- `src/indexer/languages/python.rs` — extract metrics
- `src/indexer/languages/go.rs` — extract metrics
- `src/indexer/mod.rs` — pass new fields through to symbol insertion, compute complexity_score
- `ExtractedSymbol` struct — add new optional fields

**Complexity computation (per language adapter):**

- **param_count**: count parameter children of the parameters node
- **nesting_depth**: walk function body, track depth for branching/looping nodes, record max
- **branch_count**: count all branching nodes within function body
- **visibility**: `visibility_modifier` (Rust), `export` keyword (TS), leading `_` (Python),
  uppercase first letter (Go)

**Enhance `health` and `anatomy`** to include complexity metrics in output once available.

---

### Phase 5: `dead`, `hotspots`, `circular`, `coupling` (no schema changes)

Four independent diagnostic commands that can be built in any order or in parallel.

#### `dead` — find unreferenced symbols

**Algorithm:**

1. Query all `symbols_v2` definitions
2. For each, check `ast_references`, `text_occurrences`, and `symbol_edges_v2` (incoming)
3. Assign confidence levels:
   - **High**: no AST refs, no text refs, no incoming edges, not public, not a test, not `main`
   - **Medium**: no AST refs, no incoming edges, but IS public (could have external consumers)
   - **Low**: has text occurrences but no AST references (might be string-referenced)
4. Report by confidence level, sorted by file then line

**Flags:**

- `--confidence <high|medium|low>` — minimum confidence to report (default: high)
- `--include-public` — include public symbols (moves them from medium to reportable)
- `--include-tests` — include test functions
- `--json`, `--repo`

**SQL approach:** LEFT JOIN `symbols_v2` against `ast_references` and `symbol_edges_v2` (incoming),
filter for NULL.

#### `hotspots` — most-depended-on symbols

**What:** Symbols with the highest inbound reference count. The load-bearing walls of the codebase.

**Algorithm:**

1. Count incoming edges per symbol from `symbol_edges_v2`
2. Count AST references per symbol from `ast_references`
3. Combine and rank by total inbound count
4. Annotate with file, line count, complexity score (if available)

**Flags:**

- `--top N` — number of results (default: 20)
- `--kind <function|struct|...>` — filter by symbol kind
- `--json`, `--repo`

**Why this matters:** Hotspots are the riskiest to change and the most valuable to refactor well.
Different from `coupling` (pairwise) — this answers "what are the most depended-on things in the
entire codebase?"

#### `circular` — cycle detection in the dependency graph

**What:** Find circular dependencies between files/modules.

**Algorithm:**

1. Build a directed graph: file A → file B if any symbol in A has an outgoing edge to a symbol in B
   (from `symbol_edges_v2` joined with `symbols_v2`)
2. Run cycle detection (Tarjan's or Johnson's algorithm for all elementary circuits)
3. Report cycles sorted by length (shortest first — these are the most actionable)
4. For each cycle, list the specific symbols creating the dependency

**Flags:**

- `--max-length N` — only report cycles up to N files (default: 10)
- `--json`, `--repo`

#### `coupling` — cross-file reference density with directionality

**What:** Show which files are most tangled together, with directional counts.

**Algorithm:**

1. Query `symbol_edges_v2` joined with `symbols_v2` on both sides
2. Group edges by (source_file, target_file) where source_file != target_file
3. Count edges per direction: A→B and B→A separately
4. Classify:
   - **Asymmetric** (normal): A→B is high, B→A is zero/low. This is a clean dependency.
   - **Symmetric** (smell): both A→B and B→A are significant. This is entanglement.
5. Sort by total bidirectional count descending, flag symmetric pairs

**Flags:**

- `--threshold N` — minimum edge count to report (default: 3)
- `--symmetric-only` — only show bidirectional coupling (the real smells)
- `--top N` — number of pairs (default: 20)
- `--json`, `--repo`

**Files to modify (for all four commands):**

- `src/cli.rs` — add `Dead`, `Hotspots`, `Circular`, `Coupling` commands with respective args
- `src/query/mod.rs` — add query functions and result structs for each
- `src/output.rs` — add print functions for each
- `src/main.rs` — add run handlers for each

---

### Phase 6: `test-gaps` and `test-quality` (no schema changes)

**What:** Assess test coverage and test health using structural analysis (no instrumentation
needed).

#### `test-gaps` — the pre-refactoring gate

**Algorithm:**

1. Query all symbols in the target file/module from `symbols_v2`
2. For each symbol, run `tests-for` logic — does any test file reference it?
3. Classify each symbol:
   - **Covered**: at least one test references this symbol
   - **Uncovered**: no test references found
4. Prioritize uncovered symbols by risk: public + many callers + large = high risk
5. Report coverage ratio and prioritized gap list

**Output:**

```
Test gap analysis for src/query/mod.rs:

  COVERED (32 of 45 functions):
    query_find          → tests/milestone50_find.rs
    query_refs          → tests/milestone51_refs.rs
    ...

  UNCOVERED (13 functions — no associated tests):
    ⚠ HIGH   health_report        [42 lines, 3 callers, public]
    ⚠ HIGH   file_anatomy         [67 lines, 1 caller, public]
      MEDIUM resolve_symbol_key   [28 lines, 8 callers, crate-private]
      LOW    format_helper        [8 lines, 1 caller, private]
    ...

  SUMMARY: 71% coverage (32/45) — 3 high-risk gaps
```

**Flags:**

- `--file <path>` — analyze a single file
- `--symbol <name>` — analyze a single symbol and its neighborhood
- `--min-risk <high|medium|low>` — filter by risk level
- `--json`, `--repo`

#### `test-quality` — are existing tests healthy?

**Algorithm:**

1. Find all test functions (kind = function in files matching test patterns)
2. Flag potential issues:
   - Very long test functions (>100 lines)
   - Test functions that don't reference their likely target symbol (stale/misnamed)
   - Symbols with only one associated test (single-path coverage)
   - Test files with very few assertions (check for `assert` text occurrences)
3. Report flagged tests with reasons

**Flags:**

- `--file <path>` — analyze tests for a specific file
- `--json`, `--repo`

---

### Phase 7: `suggest` command (depends on Phases 2-6)

**What:** Aggregate all diagnostic signals into a prioritized list of refactoring recommendations.

**Algorithm:**

1. For each symbol (functions above a minimum size):
   - Size score: from `line_count`
   - Complexity score: from `complexity_score` (Phase 4)
   - Coupling score: how coupled is its file? (Phase 5)
   - Cohesion score: how cohesive is its cluster? (Phase 3)
   - Hotspot score: how many things depend on it? (Phase 5)
   - Test readiness: does it have tests? (Phase 6)
2. Compute a weighted `refactoring_value` score
3. Classify test readiness:
   - ✓ SAFE: has associated tests
   - ⚠ RISKY: no associated tests (recommend writing tests first)
4. Sort by refactoring value descending
5. For each suggestion, include: why it's recommended, its risk level, and the recommended action

**Output:**

```
Refactoring suggestions for repo-scout:

#1  query_find (src/query/mod.rs:200-550)
    Score: 87/100 — large (350 lines), complex (78), 12 callers
    Tests: ✓ SAFE — 4 associated tests
    Action: extract into sub-functions by cluster

#2  resolve_edges (src/indexer/mod.rs:400-680)
    Score: 72/100 — large (280 lines), complex (65), 8 callers
    Tests: ⚠ RISKY — 0 associated tests
    Action: write tests first, then simplify branching

#3  print_impact (src/output.rs:100-200)
    Score: 61/100 — medium (100 lines), moderate complexity (45)
    Tests: ✓ SAFE — 2 associated tests, low coupling
    Action: extract formatting helpers
```

**Flags:**

- `--top N` — number of suggestions (default: 10)
- `--safe-only` — only suggest targets with test coverage
- `--min-score N` — minimum refactoring value score
- `--json`, `--repo`

**Graceful degradation:** If complexity metrics aren't available (Phase 4 not yet implemented or
index not re-run), `suggest` works with whatever data IS available — line counts, reference counts,
test gaps. The score formula adjusts weights to the available signals.

**Files to modify:**

- `src/cli.rs` — add `Suggest(SuggestArgs)`
- `src/query/mod.rs` — add `suggest_refactorings()`, `Suggestion` struct
- `src/output.rs` — add `print_suggest()` and `print_suggest_json()`
- `src/main.rs` — add `run_suggest()` handler

---

## Layer 2: Planning (what would change?)

### Phase 8: `deps` and `boundary` (no schema changes)

#### `deps` — upstream dependencies of a symbol

**What:** Everything a function depends on.

**Output:**

- Parameters (from signature)
- Functions it calls (outgoing `calls` edges)
- Types it uses (outgoing `imports` edges + referenced types)
- Modules it reaches into (derived from file paths of dependencies)

**Algorithm:**

1. Find target in `symbols_v2`
2. Query `symbol_edges_v2` for outgoing edges
3. Join with `symbols_v2` for dependency metadata
4. Group by category

**Flags:** symbol positional arg, `--json`, `--repo`

#### `boundary` — public API surface of a file or module

**What:** Show what's exported vs internal in a file or module.

**Algorithm:**

1. Query `symbols_v2` for the target file/directory
2. Classify by visibility (from Phase 4, or inferred from AST if visibility column not yet
   available)
3. For public symbols, count external references (from other files)
4. Report: public API surface, internal symbols, and external consumer counts

**Output:**

```
Boundary analysis for src/query/mod.rs:

  PUBLIC API (12 symbols — referenced from other files):
    query_find           [called from main.rs, 3 tests]
    query_refs           [called from main.rs, 2 tests]
    HealthReport         [used in main.rs, output.rs]
    ...

  INTERNAL (33 symbols — not referenced externally):
    resolve_symbol_key   [8 internal callers]
    format_path          [2 internal callers]
    ...

  Safe to restructure: 33/45 symbols (73%) are internal
```

**Flags:**

- `--file <path>` or `--module <path>` — target
- `--public-only` — only show public API
- `--json`, `--repo`

---

### Phase 9: `move-check`, `extract-check`, `rename-check`, `split-check`

Four planning commands that compose existing queries into refactoring-specific checklists.

#### `move-check <symbol> --to <path>` — pre-flight for moving a symbol

Composes: `deps` + `refs` + `tests-for` + `boundary`

**Output:**

- Dependencies that must come with it or be imported at the new location
- Call sites that need updating (with file:line)
- Tests that are affected
- Whether the move changes the public API boundary

#### `extract-check <symbol> --lines <range>` — pre-flight for function extraction

**Algorithm:**

1. Parse the target function's AST within the line range
2. Identify variables used in the range that are defined outside it → these become parameters
3. Identify variables defined in the range that are used after it → these become return values
4. Identify function calls within the range → these become dependencies of the new function
5. Report: proposed parameter list, proposed return type, dependencies

**Output:**

```
Extract analysis for query_find lines 200-350:

  Proposed function: extracted_query_find_core

  Parameters needed (variables crossing in):
    conn: &Connection          (defined line 195)
    symbol_name: &str          (defined line 198)
    opts: &FindOptions         (defined line 199)

  Return values needed (variables crossing out):
    results: Vec<FindResult>   (used at line 360)

  Internal dependencies:
    resolve_symbol_key()       (called at line 220)
    normalize_path()           (called at line 245)

  Estimated extracted size: 150 lines
  Estimated complexity reduction: 45 → 28 (original), 22 (extracted)
```

#### `rename-check <symbol> --to <new_name>` — preview all rename impacts

More comprehensive than `refs`: includes AST references, text occurrences in strings/comments/docs,
re-exports, and derived names (impl blocks, test names containing the symbol name).

**Output:**

```
Rename impact for QueryResult → QueryMatch:

  AST references (17 locations — will be caught by compiler):
    src/query/mod.rs:45     impl QueryResult
    src/query/mod.rs:120    fn find() -> Vec<QueryResult>
    src/main.rs:200         let results: Vec<QueryResult> = ...
    ...

  Text occurrences (4 locations — must be updated manually):
    src/output.rs:50        // Formats a QueryResult for display
    tests/milestone50.rs:10 // Tests QueryResult formatting
    README.md:30            Returns a `QueryResult` struct
    CLAUDE.md:15            `QueryResult` contains...

  Derived names (2 locations — may need updating):
    tests/milestone50.rs:20 fn test_query_result_formatting()
    src/output.rs:55        fn print_query_result()
```

#### `split-check <file> --groups "<group1>:<group2>"`

**Algorithm:**

1. Take proposed groupings (comma-separated symbol names per group, colon-separated groups)
2. For each group, compute:
   - What imports would the new file need?
   - What cross-references between groups would become inter-file dependencies?
   - Would this create any new circular dependencies?
3. Score the split: does it improve cohesion? Does it reduce coupling?

**Flags for all four commands:** `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add all four commands with respective args
- `src/query/mod.rs` — add query functions for each
- `src/output.rs` — add print functions for each
- `src/main.rs` — add run handlers for each

---

### Phase 10: `test-scaffold` and `safe-steps`

#### `test-scaffold <symbol>` — structured test setup information

**What:** Provide all the information needed to write a test for a symbol, without generating the
test code itself (that's the AI agent's job).

**Output:**

```json
{
  "symbol": "health_report",
  "file": "src/query/mod.rs",
  "signature": "pub fn health_report(conn: &Connection, top_n: usize) -> Result<HealthReport>",
  "dependencies": [
    { "symbol": "Connection", "from": "rusqlite", "kind": "type" },
    { "symbol": "symbols_v2", "kind": "table_query" },
    { "symbol": "indexed_files", "kind": "table_query" }
  ],
  "callers": [{ "symbol": "run_health", "file": "src/main.rs", "line": 450 }],
  "existing_test_conventions": {
    "test_dir": "tests/",
    "naming_pattern": "milestone<N>_<feature>.rs",
    "framework": "assert_cmd::Command",
    "fixture_pattern": "tests/fixtures/"
  },
  "suggested_test_file": "tests/milestone90_health.rs",
  "suggested_test_cases": [
    "empty repo (no indexed files)",
    "repo with files of varying sizes",
    "respects --top N limit",
    "JSON output is deterministic"
  ]
}
```

The suggested test cases are derived from the function's parameters (each param suggests edge cases)
and its branching structure (each branch suggests a test path).

#### `safe-steps <symbol> --action <move|extract|rename|split> [--to <path>]`

**What:** Decompose a refactoring into ordered, individually-safe steps where each step leaves the
codebase compilable.

**Output for `safe-steps query_find --action extract --lines 200-350`:**

```
Safe refactoring steps for extracting query_find lines 200-350:

  Step 1: Create the new function
    - Add extracted_query_find_core() below query_find in src/query/mod.rs
    - Signature: fn extracted_query_find_core(conn: &Connection, ...) -> Vec<FindResult>
    - Copy lines 200-350 into the new function body

  Step 2: Update query_find to call the extracted function
    - Replace lines 200-350 with: let results = extracted_query_find_core(conn, ...);
    - Verify: cargo check should pass

  Step 3: Run tests
    - cargo test milestone50 — should still pass (behavior unchanged)

  Step 4: (Optional) Move to separate file if desired
    - Use move-check extracted_query_find_core for impact analysis
```

**Algorithm:** Applies refactoring-specific templates (move, extract, rename, split) with data from
the planning commands. Each template defines a safe step sequence. Steps are annotated with
verification commands.

---

## Layer 3: Verification (did I get everything?)

### Phase 11: `verify-refactor` (no schema changes)

**What:** Compare the index before and after a refactoring to detect incomplete changes.

**Algorithm:**

1. Accept `--before <commit>` and `--after <commit>` (or `--before <baseline-file>`)
2. Index at both points (or use saved baseline)
3. Compare:
   - Symbols that disappeared but still have references pointing to them → incomplete move/delete
   - New unresolved references → something was renamed/moved but not all refs updated
   - Edge changes → dependency graph shifted, verify it's intentional
   - Test associations → did any previously-tested symbol lose its tests?
4. Report discrepancies as warnings

**Output:**

```
Refactoring verification (HEAD~1 → HEAD):

  ✓ No broken references
  ✓ All moved symbols have updated references
  ⚠ 2 warnings:
    - format_helper (src/output.rs:50) lost its test association
      (test_format_helper in milestone50.rs still references old location)
    - New circular dependency introduced: query/mod.rs ↔ output.rs
      (output.rs now calls query_find, which didn't exist before)
```

**Flags:**

- `--before <commit|baseline>` — reference point
- `--after <commit>` — comparison point (default: working tree)
- `--strict` — treat warnings as errors (exit code 1)
- `--json`, `--repo`

---

### Phase 12: `health --diff` (no schema changes, extends Phase 2)

**What:** Before/after comparison of health metrics against a saved baseline.

**Approach:**

- `health --save-baseline` saves metrics to `.repo-scout/health-baseline.json`
- `health --diff` compares current metrics against baseline
- Shows improvements (green) and regressions (red)

**Output:**

```
Health comparison (baseline: 2025-01-15 → current):

  Largest file:     query/mod.rs  121k → 95k lines  ✓ improved (-22%)
  Avg complexity:   45 → 38                         ✓ improved (-16%)
  Dead symbols:     12 → 8                          ✓ improved (-33%)
  Test coverage:    71% → 84%                       ✓ improved (+13%)
  Circular deps:    2 → 2                           — unchanged
  Max coupling:     A↔B: 15 → A↔B: 15              — unchanged
```

**Files to modify:**

- Extend `HealthArgs` with `--save-baseline` and `--diff`
- Add baseline save/load logic in `src/query/mod.rs`
- Add diff rendering in `src/output.rs`

---

## Implementation Order & Dependencies

```
FOUNDATION (immediate value):
  Phase 1:  tree              ← start here, orientation for everything
  Phase 2:  health            ← dashboard, small schema change

DIAGNOSIS (build the intelligence):
  Phase 3:  anatomy + cohesion    ← single-file deep dive
  Phase 4:  complexity metrics    ← schema extension, enhances Phase 2 & 3
  Phase 5:  dead, hotspots,       ← four independent commands, any order
            circular, coupling
  Phase 6:  test-gaps,            ← test readiness assessment
            test-quality
  Phase 7:  suggest               ← depends on Phases 2-6 (gracefully degrades)

PLANNING (refactoring support):
  Phase 8:  deps, boundary        ← building blocks for planning commands
  Phase 9:  move-check,           ← compose Phase 8 queries
            extract-check,
            rename-check,
            split-check
  Phase 10: test-scaffold,        ← depends on Phases 6 + 8
            safe-steps

VERIFICATION (close the loop):
  Phase 11: verify-refactor       ← independent
  Phase 12: health --diff         ← extends Phase 2
```

**Parallelization opportunities:**

- Phases 1 and 2 can be built in parallel (no dependencies)
- Within Phase 5, all four commands are independent
- Phase 6 and Phase 3 can be built in parallel
- Phase 11 can be built any time after Phase 2
- Within Phase 9, all four check commands are independent (given Phase 8)

---

## Schema Changes Summary

| Phase   | Change                                                                                                         | Version |
| ------- | -------------------------------------------------------------------------------------------------------------- | ------- |
| Phase 2 | `line_count INTEGER` on `indexed_files`                                                                        | v4      |
| Phase 4 | `line_count`, `param_count`, `nesting_depth`, `branch_count`, `visibility`, `complexity_score` on `symbols_v2` | v5      |

Only two schema migrations total. All other phases work with existing data.

---

## JSON Output Contract

All `--json` output must follow these rules (consistent with existing repo-scout conventions):

1. **Deterministic**: identical input → identical output. Enforce via explicit SQL ordering and
   stable tie-breakers.
2. **Versioned**: include `"schema_version"` in top-level JSON.
3. **Cross-referenceable**: symbols are identified by `file_path + symbol_name + line` consistently
   across all commands, so output from `suggest` can reference the same symbols as `anatomy`,
   `test-gaps`, etc.
4. **Actionable**: include enough context for an AI agent to act without re-querying. E.g., a
   suggestion includes the file path, line range, and recommended action — not just a symbol name.

---

## Verification (per phase)

For each phase:

1. Write failing integration test first (`tests/milestone<N>_<feature>.rs`)
2. Implement minimal code to pass
3. Run `just check` (fmt + clippy + full test suite)
4. Dogfood: `cargo run -- index --repo . && cargo run -- <new-command> --repo .`
5. Verify `--json` output is deterministic (run twice, diff)
6. Verify human-readable output is scannable and useful

---

## Future Considerations (not in this plan)

### Duplication Detection

Structural duplication (functions with similar AST shapes). Requires AST fingerprinting, similarity
comparison, clustering. High value but high implementation cost. Deferred to a future plan.

### Watch Mode

`repo-scout watch` — re-index on file changes and continuously update health metrics. Useful for IDE
integration and CI pipelines.

### IDE/Editor Integration

LSP-like integration where `tree`, `anatomy`, `suggest`, and `test-gaps` feed directly into editor
UI (VS Code extension, Neovim plugin).

### Multi-Repo Analysis

Cross-repository dependency and coupling analysis for monorepo or microservice architectures.
