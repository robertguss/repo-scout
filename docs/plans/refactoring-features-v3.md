# Repo-Scout Refactoring & Code Intelligence — Implementation Plan v3

## Preamble

This plan was written by Claude (Opus 4.6) at Robert's request. Unlike v2, which was a feature
specification, this plan is designed from the question: **"What does an AI coding agent actually
need to safely and confidently navigate, understand, and refactor a codebase?"**

Every feature earns its place by answering a question I (Claude) genuinely face when working on
code. If a feature doesn't reduce uncertainty or prevent mistakes, it doesn't belong here.

**Supersedes:** `docs/plans/refactoring-features-v2.md` (preserved as-is)

**Design principles:**

- Features exist to reduce uncertainty. Every command should answer a question the agent can't
  currently answer, or answer an existing question faster/better.
- Compose existing commands before building new ones. Repo-scout already has 19 commands — new
  features should build on that foundation, not ignore it.
- Line count first, complexity later. Ship with the simplest useful signal, design schema to accept
  richer metrics without migration pain.
- Full variable-flow analysis for extract-check. Do it properly across all four language adapters.
- Determinism is non-negotiable. All output must be identical given identical input.

---

## What Already Exists (and what it can't do)

Before designing new features, here's what repo-scout already provides and where the gaps are:

| Command       | What it does                        | What it can't do                              |
| ------------- | ----------------------------------- | --------------------------------------------- |
| `summary`     | File count, definitions, languages  | No structural map, no health signals          |
| `outline`     | Symbols in a single file            | No cross-file relationships, no annotations   |
| `find`        | Symbol lookup (AST + text)          | No filtering by health/complexity             |
| `refs`        | All references to a symbol          | No directionality analysis                    |
| `impact`      | What depends on a symbol            | No distance/depth tracking                    |
| `callers`     | Incoming call edges                 | No transitive analysis                        |
| `callees`     | Outgoing call edges                 | No transitive analysis                        |
| `deps`        | File-level dependency graph         | No coupling scoring, no directionality        |
| `hotspots`    | Most-connected symbols (fan-in/out) | No complexity annotation, no risk scoring     |
| `explain`     | Symbol details + call graph         | No cohesion context, no health context        |
| `tests-for`   | Test files covering a symbol        | No gap analysis, no risk prioritization       |
| `diff-impact` | Blast radius of file changes        | No pre/post comparison, no completeness check |
| `call-path`   | Path between two symbols            | No cycle detection across the whole graph     |
| `related`     | Structurally related symbols        | No clustering, no split recommendations       |
| `context`     | Task-relevant code discovery        | No refactoring-specific context               |
| `snippet`     | Source code extraction              | No variable-flow analysis within snippets     |

**Key insight:** The existing commands are excellent at answering point queries ("what is X?", "who
calls X?"). What's missing is aggregate intelligence ("what's wrong with this codebase?", "is it
safe to refactor X?", "did I miss anything?").

---

## The Two Workflows

### Workflow 1: Exploration (dropped into an unknown codebase)

```
orient               → one-command orientation (composite)
  ├── tree           → structural map with dependency flow
  ├── health         → code health dashboard
  ├── hotspots       → (existing, enhanced) load-bearing symbols
  └── circular       → cycle detection

anatomy <file>       → deep dive into a specific file
test-gaps <file>     → is this file safe to change?
```

### Workflow 2: Active Refactoring (mid-task safety checks)

```
Pre-refactoring:
  boundary <file>    → what's public vs internal?
  test-gaps <symbol> → is this symbol tested?
  extract-check      → what happens if I extract this?
  move-check         → what breaks if I move this?
  rename-check       → what needs updating if I rename this?

During refactoring:
  safe-steps         → decompose into safe increments
  test-scaffold      → what tests should I write?

Post-refactoring:
  verify-refactor    → did I miss anything?
  health --diff      → did health improve?
```

---

## Architecture: Query Module Split

The existing `query/mod.rs` (4000+ lines, 183 symbols) will be split as part of this work. New query
functions go into new modules; existing functions stay in `mod.rs` until a natural seam emerges.

```
src/query/
├── mod.rs              # Existing queries (find, refs, impact, etc.) + re-exports
├── diagnostics.rs      # health, anatomy, dead, circular, coupling, test-gaps, test-quality
├── planning.rs         # boundary, move-check, extract-check, rename-check, split-check,
│                       # test-scaffold, safe-steps
├── verification.rs     # verify-refactor
└── orientation.rs      # tree, orient (composite command)
```

Shared types (like `QueryScope`, scoring helpers) stay in `mod.rs`. Each new module imports from
`mod.rs` as needed. The `output.rs` file follows the same pattern — new print functions can go in
`output.rs` or be split if it grows unwieldy.

**Migration strategy:** Don't move existing functions in Phase 1. Create new modules for new code.
Optionally refactor existing functions into modules in a later phase once `anatomy` can dogfood the
split decisions.

---

## Schema Changes

### Single migration: v3 → v4

All schema changes happen in one migration to minimize churn:

```sql
-- Add line_count to indexed_files (populated during indexing)
ALTER TABLE indexed_files ADD COLUMN line_count INTEGER;

-- Add metrics columns to symbols_v2 (nullable, populated incrementally)
ALTER TABLE symbols_v2 ADD COLUMN line_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN visibility TEXT;

-- Reserve columns for future complexity metrics (not populated in v3→v4)
-- These are added now so we don't need another migration later:
ALTER TABLE symbols_v2 ADD COLUMN param_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN nesting_depth INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN branch_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN complexity_score INTEGER;
```

**Why add complexity columns now if we're not populating them?** Because schema migrations require
re-indexing. Adding nullable columns once is cheap. Adding them later means another migration cycle.
The columns exist but are NULL until a future phase populates them. All queries that use these
columns must handle NULL gracefully.

**`line_count` on `symbols_v2`:** Precomputed as `end_line - start_line + 1`. This is the primary
health signal for v3.

**`visibility` on `symbols_v2`:** "public", "private", "crate" (Rust), "export" (TS), etc. Needed by
`boundary`, `dead`, and `test-gaps` for risk classification. Populated by language adapters.

**`line_count` on `indexed_files`:** Total lines per file. Populated during indexing by counting
newlines. Needed by `tree`, `health`, and `anatomy`.

---

## Variable-Flow Analysis (for `extract-check`)

This is the most technically ambitious addition. Each language adapter gains a new method:

```rust
pub trait LanguageAdapter {
    // ... existing methods ...

    /// Analyze variable flow within a function body for a given line range.
    /// Returns variables that cross the extraction boundary.
    fn analyze_extraction(
        &self,
        source: &str,
        function_start_line: u32,
        function_end_line: u32,
        extract_start_line: u32,
        extract_end_line: u32,
    ) -> Result<ExtractionAnalysis>;
}

pub struct ExtractionAnalysis {
    /// Variables declared outside the range but used inside → become parameters
    pub params_needed: Vec<VariableCrossing>,
    /// Variables declared inside the range but used after it → become return values
    pub returns_needed: Vec<VariableCrossing>,
    /// Variables declared and used only within the range → local to extracted fn
    pub locals: Vec<String>,
    /// Function calls made within the range → dependencies of extracted fn
    pub internal_calls: Vec<String>,
    /// Types referenced within the range
    pub referenced_types: Vec<String>,
}

pub struct VariableCrossing {
    pub name: String,
    pub inferred_type: Option<String>,  // best-effort from AST
    pub declared_line: u32,
    pub used_lines: Vec<u32>,
}
```

**Implementation per language:**

- **Rust:** Walk function body AST via `syn`. Track `let` bindings, function parameters, pattern
  bindings. For each identifier usage, check if its declaration is inside or outside the range. Type
  inference from `let x: Type` patterns and function signatures.

- **TypeScript:** Walk function body via tree-sitter. Track `const`/`let`/`var` declarations,
  function parameters, destructuring bindings. Identifier resolution via tree-sitter scope queries.
  Type inference from explicit annotations.

- **Python:** Walk function body via tree-sitter. Track assignments, function parameters, `for`
  targets, `with` targets, comprehension variables. Simpler scoping model (function-level by
  default).

- **Go:** Walk function body via tree-sitter. Track `:=` short declarations, `var` declarations,
  function parameters, range variables. Type inference from explicit declarations.

**Limitations (documented in output):**

- Cannot track variables captured by closures that cross the boundary
- Cannot resolve type aliases or trait-based type inference
- Best-effort type annotation — may show `?` for complex inferred types
- Does not handle mutations (mutable borrows in Rust, pointer reassignment in Go)

---

## Phase 1: Foundation — `tree`, `health`, `orient`

### `tree` — structural map with dependency flow

**The question it answers:** "What is this codebase's structure, and how do the pieces connect?"

**Why existing commands don't suffice:** `summary` gives counts. `outline` shows one file. Neither
shows the directory structure annotated with intelligence from the index. An AI agent currently
needs to run `summary` + `outline` on every file + `deps` on every file to build the same mental
model.

**Default behavior includes dependency flow.** The plain directory listing is the less useful mode —
an agent can already do `find . -name '*.rs'`. The value is in the annotations.

**Output (default — with dependency annotations):**

```
src/                               [4 modules, 156 symbols]
├── main.rs                        [1085 lines │ 15 fns, 3 structs]
│   → imports: cli, indexer, output, query, store
├── cli.rs                         [200 lines  │ 1 enum, 12 structs]
│   ← used by: main.rs
├── output.rs                      [945 lines  │ 20 fns]
│   ← used by: main.rs
│   → imports: query
├── store/                         [2 files, 10 symbols]
│   ├── mod.rs                     [300 lines  │ 8 fns]
│   └── schema.rs                  [100 lines  │ 2 fns, 5 consts]
├── indexer/                       [6 files, 52 symbols]
│   ├── mod.rs                     [500 lines  │ 12 fns]
│   ├── files.rs                   [100 lines  │ 3 fns]
│   ├── text.rs                    [100 lines  │ 2 fns]
│   └── languages/                 [4 files, 35 symbols]
│       ├── rust.rs                [300 lines  │ 8 fns]
│       ├── typescript.rs          [800 lines  │ 12 fns]
│       ├── python.rs              [700 lines  │ 10 fns]
│       └── go.rs                  [600 lines  │ 9 fns]
└── query/                         [1 file, 183 symbols]
    └── mod.rs                     [4000 lines │ 45 fns, 20 structs │ ⚠ largest file]
        → imports: store
        ← used by: main.rs, output.rs
```

**Flags:**

- `--depth N` — control tree depth (default: 3)
- `--no-deps` — plain tree without dependency arrows
- `--focus <path>` — zoom into a subtree
- `--symbols` — expand to show individual symbols per file
- `--json`, `--repo`

**Data sources:** `indexed_files` for file list + `line_count`, `symbols_v2` for symbol counts,
`symbol_edges_v2` joined with `symbols_v2` for inter-file dependency arrows, filesystem for
directory structure.

**Files to modify:**

- `src/cli.rs` — add `Tree(TreeArgs)`
- `src/query/orientation.rs` — new file: `tree_report()`, `TreeNode` struct
- `src/output.rs` — add `print_tree()` and `print_tree_json()`
- `src/main.rs` — add `run_tree()` handler
- `src/store/schema.rs` — migration adding `line_count` to `indexed_files`
- `src/indexer/mod.rs` — populate `line_count` during indexing

---

### `health` — code health dashboard

**The question it answers:** "What parts of this codebase are unhealthy and should I be careful
around?"

**Output:**

```
Code health for repo-scout:

  LARGEST FILES:
    #1  src/query/mod.rs              4199 lines   183 symbols
    #2  src/main.rs                   1164 lines    52 symbols
    #3  src/output.rs                  945 lines    42 symbols

  LARGEST FUNCTIONS:
    #1  command_handlers_cover_...     350 lines    src/main.rs:734
    #2  diff_impact_for_changed_...    267 lines    src/query/mod.rs:266
    #3  explain_symbol                  85 lines    src/query/mod.rs:680

  DENSEST FILES (most symbols per file):
    #1  src/query/mod.rs              183 symbols   (0.044 per line)
    #2  src/main.rs                    52 symbols   (0.045 per line)

  CODE MARKERS:
    TODO: 3    FIXME: 0    HACK: 1    SAFETY: 0

  SUMMARY:
    Total: 325 source files, 1348 definitions, 1552 edges
    Warning: 1 file over 2000 lines (src/query/mod.rs)
```

**Flags:**

- `--top N` — results per category (default: 20)
- `--threshold N` — minimum line count to report
- `--large-files` — only show large files
- `--large-functions` — only show large functions
- `--save-baseline` — save metrics to `.repo-scout/health-baseline.json`
- `--diff` — compare against saved baseline (see Phase 5)
- `--json`, `--repo`

**Schema dependency:** Requires `line_count` on `indexed_files` and `symbols_v2` (Phase 1
migration).

**Files to modify:**

- `src/cli.rs` — add `Health(HealthArgs)`
- `src/query/diagnostics.rs` — new file: `health_report()`, `HealthReport` struct
- `src/output.rs` — add `print_health()` and `print_health_json()`
- `src/main.rs` — add `run_health()` handler

---

### `orient` — composite orientation command

**The question it answers:** "I just opened this codebase. What do I need to know?"

This is the single most valuable command for AI agents. Instead of running tree + health + hotspots
separately (3 round-trips, 3 outputs to synthesize), `orient` produces a unified report.

**Output:**

```
Orientation report for repo-scout:

═══ STRUCTURE ═══
[abbreviated tree output — depth 2, with dependency arrows]

═══ HEALTH ═══
[abbreviated health output — top 5 per category]

═══ HOTSPOTS ═══
[top 10 from existing hotspots command]

═══ RISKS ═══
  ⚠ 1 file over 2000 lines: src/query/mod.rs (4199 lines)
  ⚠ 1 function over 200 lines: command_handlers_cover_... (350 lines)
  ℹ No circular dependencies detected [requires `circular` — Phase 2]

═══ RECOMMENDATIONS ═══
  Start exploring: src/main.rs (entry point, 15 functions)
  Careful around: src/query/mod.rs (largest file, 183 symbols, high fan-in)
  Well-tested: tests/ (89 milestone test files)
```

**Implementation:** Calls `tree_report()`, `health_report()`, `hotspots()` internally. Adds a
synthesis layer that computes recommendations from the combined data. The `--json` output includes
all sub-reports as nested objects.

**Flags:**

- `--depth N` — tree depth (default: 2 for orient, vs 3 for standalone tree)
- `--top N` — items per section (default: 5 for orient, vs 20 for standalone)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Orient(OrientArgs)`
- `src/query/orientation.rs` — add `orient_report()`, `OrientReport` struct
- `src/output.rs` — add `print_orient()` and `print_orient_json()`
- `src/main.rs` — add `run_orient()` handler

---

## Phase 2: Diagnosis — `anatomy`, `circular`, `coupling`, `dead`

### `anatomy` — single-file structural analysis with cohesion scoring

**The question it answers:** "What's inside this big file, and where are the natural seams if I
wanted to split it?"

This is the feature I (Claude) would use the most. When I see a 4000-line file, my first question is
not "what symbols are in it?" (that's `outline`) but "which symbols are related to each other, and
what are the independent clusters?"

**Output for `anatomy src/query/mod.rs`:**

```
Anatomy of src/query/mod.rs:

  SYMBOLS (183 total — 45 functions, 20 structs, 4 enums, 114 other):

    Cluster 1: "diff-impact" (12 functions, 4 structs — 580 lines)
      Cohesion: 0.78 (high)
      diff_impact_for_changed_files    267 lines  L266    ← entry point
      collect_changed_symbol_matches    56 lines  L333
      changed_symbol_seeds              29 lines  L389
      expand_changed_symbol_neighbors   27 lines  L434
      expand_neighbors_for_symbol       32 lines  L461
      incoming_neighbors                30 lines  L493
      ...
      Shared types: DiffImpactMatch, DiffImpactState, ChangedSymbolSeed

    Cluster 2: "context" (8 functions, 3 structs — 200 lines)
      Cohesion: 0.65 (moderate)
      context_matches_scoped            36 lines  L1225   ← entry point
      context_seed_symbols              19 lines  L1278
      push_direct_context_match         32 lines  L1316
      push_neighbor_context_matches     48 lines  L1348
      ...

    Cluster 3: "verification" (12 functions, 4 structs — 350 lines)
      Cohesion: 0.72 (high)
      ...

    Unclustered (15 functions — low cohesion with any cluster):
      extract_keywords                  27 lines  L2148
      is_test_like_path                  9 lines  L2059
      ...

  SUGGESTED SPLITS:
    1. Cluster "diff-impact" → query/diff_impact.rs (580 lines, 0 external deps within file)
    2. Cluster "context" → query/context.rs (200 lines, shares extract_keywords with cluster 4)
    3. Cluster "verification" → query/verification.rs (350 lines, 0 external deps)

  FILE COHESION SCORE: 0.34 (low — file contains multiple independent concerns)
```

**Cohesion scoring algorithm:**

1. Build intra-file call graph from `symbol_edges_v2` where both source and target are in the same
   file
2. Find connected components (clusters) using union-find
3. For each cluster, compute cohesion = actual_edges / possible_edges (density), weighted by shared
   type usage
4. File-level cohesion = weighted average of cluster scores, penalized by number of unclustered
   symbols

**Flags:**

- `<file>` — target file (positional, required)
- `--clusters` — only show cluster analysis
- `--cohesion` — only show cohesion scores
- `--suggest-split` — only show split recommendations
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Anatomy(AnatomyArgs)`
- `src/query/diagnostics.rs` — add `file_anatomy()`, `AnatomyReport`, `Cluster`, `CohesionScore`
- `src/output.rs` — add `print_anatomy()` and `print_anatomy_json()`
- `src/main.rs` — add `run_anatomy()` handler

---

### `circular` — cycle detection in the dependency graph

**The question it answers:** "Are there circular dependencies, and if so, which specific symbols
create them?"

**Algorithm:**

1. Build directed file-level graph from `symbol_edges_v2` joined with `symbols_v2`
2. Run Tarjan's SCC algorithm to find strongly connected components
3. For SCCs with >1 file, extract the specific symbol edges creating the cycle
4. Sort by cycle length (shortest first — most actionable)

**Output:**

```
Circular dependencies:

  Cycle 1 (2 files):
    src/output.rs → src/query/mod.rs
      via: output::print_explain() calls query::explain_symbol()
    src/query/mod.rs → src/output.rs
      via: query::format_result() calls output::render_line()

  Cycle 2 (3 files):
    src/a.rs → src/b.rs → src/c.rs → src/a.rs
    ...

  Summary: 2 cycles found (1 two-file, 1 three-file)
```

**Flags:**

- `--max-length N` — only report cycles up to N files (default: 10)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Circular(CircularArgs)`
- `src/query/diagnostics.rs` — add `detect_circular_deps()`, `CycleDep` struct
- `src/output.rs` — add `print_circular()` and `print_circular_json()`
- `src/main.rs` — add `run_circular()` handler

---

### `coupling` — extends `deps` with directionality and scoring

**The question it answers:** "Which files are tangled together in ways that make them hard to change
independently?"

**How it differs from existing `deps`:** The current `deps` command shows depends*on/depended_on_by
for a single file. `coupling` analyzes \_all* file pairs and identifies the ones with the highest
bidirectional edge count — the smell of entanglement.

**Algorithm:**

1. Query `symbol_edges_v2` joined with `symbols_v2` on both sides
2. Group by (source_file, target_file), count edges per direction
3. Classify: asymmetric (clean dependency) vs symmetric (entanglement)
4. Sort by total bidirectional count descending

**Output:**

```
File coupling analysis:

  SYMMETRIC (bidirectional — potential entanglement):
    src/query/mod.rs ↔ src/main.rs       A→B: 5, B→A: 12   total: 17
    src/indexer/mod.rs ↔ src/store/mod.rs A→B: 8, B→A: 3    total: 11

  ASYMMETRIC (clean dependencies — top 5):
    src/main.rs → src/query/mod.rs                           edges: 12
    src/main.rs → src/output.rs                              edges: 8
    ...
```

**Flags:**

- `--threshold N` — minimum edge count (default: 3)
- `--symmetric-only` — only show bidirectional coupling
- `--top N` — number of pairs (default: 20)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Coupling(CouplingArgs)`
- `src/query/diagnostics.rs` — add `coupling_analysis()`, `CouplingPair` struct
- `src/output.rs` — add `print_coupling()` and `print_coupling_json()`
- `src/main.rs` — add `run_coupling()` handler

---

### `dead` — find unreferenced symbols

**The question it answers:** "What code in this codebase is never used and can be safely removed?"

**Algorithm:**

1. Query all `symbols_v2` definitions
2. For each, check `ast_references`, `text_occurrences`, and `symbol_edges_v2` (incoming)
3. Assign confidence:
   - **High**: no AST refs, no text refs, no incoming edges, not public, not a test, not `main`
   - **Medium**: no AST refs, no incoming edges, but IS public (could have external consumers)
   - **Low**: has text occurrences but no AST references (might be string-referenced)

**SQL approach:** LEFT JOIN `symbols_v2` against `ast_references` and `symbol_edges_v2` (incoming
edges), filter for NULL. This is a single efficient query, not per-symbol iteration.

**Flags:**

- `--confidence <high|medium|low>` — minimum confidence (default: high)
- `--include-public` — include public symbols
- `--include-tests` — include test functions
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Dead(DeadArgs)`
- `src/query/diagnostics.rs` — add `find_dead_symbols()`, `DeadSymbol` struct
- `src/output.rs` — add `print_dead()` and `print_dead_json()`
- `src/main.rs` — add `run_dead()` handler

---

## Phase 3: Test Intelligence — `test-gaps`, `test-quality`

### `test-gaps` — the pre-refactoring safety gate

**The question it answers:** "Is this file/symbol tested well enough that I can refactor it safely?"

This is the single most important pre-refactoring check. An AI agent that refactors untested code is
gambling. This command makes the risk explicit.

**Algorithm:**

1. Query all symbols in the target file from `symbols_v2`
2. For each symbol, use existing `tests-for` logic — does any test file reference it?
3. Classify: covered vs uncovered
4. Prioritize uncovered symbols by risk: public + many callers + large = high risk
5. Report coverage ratio and prioritized gap list

**Output:**

```
Test gap analysis for src/query/mod.rs:

  COVERED (32 of 45 functions):
    find_matches_scoped         → tests/milestone50_find.rs
    refs_matches_scoped         → tests/milestone51_refs.rs
    diff_impact_for_changed_... → tests/milestone12_diff_impact.rs
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

- `<file>` — analyze a file (positional)
- `--symbol <name>` — analyze a single symbol and its callers
- `--min-risk <high|medium|low>` — filter by risk level
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `TestGaps(TestGapsArgs)`
- `src/query/diagnostics.rs` — add `test_gap_analysis()`, `TestGapReport` struct
- `src/output.rs` — add `print_test_gaps()` and `print_test_gaps_json()`
- `src/main.rs` — add `run_test_gaps()` handler

---

### `test-quality` — are existing tests healthy?

**The question it answers:** "Can I trust the tests that do exist?"

**Algorithm:**

1. Find all test functions (kind = function in files matching test patterns)
2. Flag potential issues:
   - Very long test functions (>100 lines) — doing too much
   - Test functions that don't reference their likely target (stale/misnamed)
   - Symbols with only one associated test (single-path coverage)
   - Test files with very few `assert` text occurrences (weak assertions)
3. Report flagged tests with reasons

**Flags:**

- `<file>` — analyze tests for a specific source file (positional)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `TestQuality(TestQualityArgs)`
- `src/query/diagnostics.rs` — add `test_quality_analysis()`, `TestQualityReport` struct
- `src/output.rs` — add `print_test_quality()` and `print_test_quality_json()`
- `src/main.rs` — add `run_test_quality()` handler

---

## Phase 4: Refactoring Intelligence — `suggest`, `boundary`

### `suggest` — prioritized refactoring recommendations

**The question it answers:** "What should I refactor first, and is it safe to do so?"

**Algorithm:**

1. For each function above a minimum size (20 lines):
   - Size signal: `line_count` (from `symbols_v2`)
   - Hotspot signal: incoming edge count (fan-in)
   - Coupling signal: how coupled is its file? (from `coupling` logic)
   - Cohesion signal: how cohesive is its cluster? (from `anatomy` logic)
   - Test readiness: does it have associated tests? (from `test-gaps` logic)
2. Compute weighted `refactoring_value` score using available signals
3. Classify test readiness: SAFE (has tests) vs RISKY (no tests)
4. Sort by refactoring value, report with recommended action

**Graceful degradation:** `suggest` works with whatever data IS available. If only `line_count` and
`hotspots` exist, it scores on those two signals. If `anatomy` and `test-gaps` have been computed
for the relevant files, those signals are included. The output transparently shows which signals
contributed to each score.

**Flags:**

- `--top N` — number of suggestions (default: 10)
- `--safe-only` — only suggest targets with test coverage
- `--min-score N` — minimum score threshold
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Suggest(SuggestArgs)`
- `src/query/diagnostics.rs` — add `suggest_refactorings()`, `Suggestion` struct
- `src/output.rs` — add `print_suggest()` and `print_suggest_json()`
- `src/main.rs` — add `run_suggest()` handler

---

### `boundary` — public API surface of a file or module

**The question it answers:** "What's the public interface of this file, and how much can I safely
restructure internally?"

**Algorithm:**

1. Query `symbols_v2` for the target file
2. Classify by visibility (from `visibility` column, or inferred from AST patterns if not yet
   populated)
3. For public symbols, count external references from other files
4. Report: public API surface, internal symbols, safe-to-restructure percentage

**Flags:**

- `<file>` — target file (positional)
- `--public-only` — only show public API
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `Boundary(BoundaryArgs)`
- `src/query/planning.rs` — new file: `boundary_analysis()`, `BoundaryReport` struct
- `src/output.rs` — add `print_boundary()` and `print_boundary_json()`
- `src/main.rs` — add `run_boundary()` handler

---

## Phase 5: Refactoring Pre-flight — `extract-check`, `move-check`, `rename-check`, `split-check`

These four commands compose existing queries with new analysis to produce refactoring-specific
checklists.

### `extract-check` — pre-flight for function extraction

**The question it answers:** "If I extract lines X-Y from this function into a new function, what
will the signature look like and what might break?"

**This is the command that requires full variable-flow analysis** (see Architecture section above).

**Algorithm:**

1. Find the target function in `symbols_v2`
2. Read the source file
3. Call the language adapter's `analyze_extraction()` method for the specified line range
4. Combine with graph data: what functions are called within the range (from `symbol_edges_v2`),
   what types are referenced
5. Produce the extraction report

**Output:**

```
Extract analysis for query_find lines 200-350:

  Proposed function: extracted_query_find_core

  Parameters needed (variables crossing in):
    conn: &Connection          (declared line 195, used lines 210, 230, 280)
    symbol_name: &str          (declared line 198, used lines 215, 240)
    opts: &FindOptions         (declared line 199, used lines 205, 320)

  Return values needed (variables crossing out):
    results: Vec<FindResult>   (declared line 210, used at line 360)

  Internal calls (within extracted range):
    resolve_symbol_key()       (called at line 220)
    normalize_path()           (called at line 245)

  Referenced types:
    Connection, FindOptions, FindResult, SymbolKey

  Estimated extracted size: 150 lines
```

**Flags:**

- `<symbol>` — target function (positional)
- `--lines <start>-<end>` — line range to extract (required)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `ExtractCheck(ExtractCheckArgs)`
- `src/query/planning.rs` — add `extract_check()`, `ExtractionReport` struct
- `src/output.rs` — add `print_extract_check()` and `print_extract_check_json()`
- `src/main.rs` — add `run_extract_check()` handler
- `src/indexer/languages/rust.rs` — add `analyze_extraction()`
- `src/indexer/languages/typescript.rs` — add `analyze_extraction()`
- `src/indexer/languages/python.rs` — add `analyze_extraction()`
- `src/indexer/languages/go.rs` — add `analyze_extraction()`
- `src/indexer/languages/mod.rs` — add method to `LanguageAdapter` trait

---

### `move-check` — pre-flight for moving a symbol

**The question it answers:** "If I move this symbol to a different file, what breaks?"

**Composes:** existing `refs` + `impact` + `tests-for` + new `boundary`

**Output:**

- Dependencies that must come with it or be imported at the new location
- Call sites that need updating (with file:line)
- Tests that reference it
- Whether the move changes the public API boundary

**Flags:**

- `<symbol>` — target symbol (positional)
- `--to <path>` — destination file
- `--json`, `--repo`

---

### `rename-check` — preview all rename impacts

**The question it answers:** "If I rename this symbol, what needs to change beyond what the compiler
catches?"

More comprehensive than `refs`: includes AST references, text occurrences in strings/comments/docs,
re-exports, and derived names (impl blocks, test names containing the symbol name).

**Output includes:**

- AST references (compiler will catch these)
- Text occurrences in comments/docs/strings (must be updated manually)
- Derived names (test functions, impl blocks referencing the name)

**Flags:**

- `<symbol>` — current name (positional)
- `--to <new_name>` — proposed new name
- `--json`, `--repo`

---

### `split-check` — pre-flight for file splitting

**The question it answers:** "If I split this file into these groups, what cross-references would I
create?"

**Composes:** `anatomy` clusters + `boundary` + `circular` check

**Algorithm:**

1. Accept proposed groupings (symbol lists per group)
2. For each group, compute: needed imports, cross-group references, new circular deps
3. Score the split: does it improve cohesion?

**Flags:**

- `<file>` — target file (positional)
- `--groups "<group1>:<group2>"` — proposed groupings
- `--auto` — use `anatomy` cluster suggestions as groups
- `--json`, `--repo`

---

**Files to modify (for move-check, rename-check, split-check):**

- `src/cli.rs` — add all three commands with args
- `src/query/planning.rs` — add query functions for each
- `src/output.rs` — add print functions for each
- `src/main.rs` — add run handlers for each

---

## Phase 6: Refactoring Support — `test-scaffold`, `safe-steps`

### `test-scaffold` — structured test setup information

**The question it answers:** "I need to write a test for this symbol. What do I need to know?"

**Not a test generator.** The AI agent writes the test. This command provides the context needed to
write it well: signature, dependencies, existing test conventions, suggested test file location,
suggested test cases derived from the function's parameters and branching.

**Output (JSON optimized for AI consumption):**

```json
{
  "symbol": "health_report",
  "file": "src/query/diagnostics.rs",
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

**Flags:**

- `<symbol>` — target symbol (positional)
- `--json`, `--repo`

---

### `safe-steps` — decompose refactoring into safe increments

**The question it answers:** "What's the safest order of operations for this refactoring?"

**Algorithm:** Applies refactoring-type-specific templates (move, extract, rename, split) populated
with data from the planning commands. Each step includes a verification command.

**Output for `safe-steps query_find --action extract --lines 200-350`:**

```
Safe refactoring steps for extracting query_find lines 200-350:

  Step 1: Create the new function
    - Add extracted_query_find_core() below query_find in src/query/mod.rs
    - Signature: fn extracted_query_find_core(conn: &Connection, ...) -> Vec<FindResult>
    - Copy lines 200-350 into the new function body
    Verify: cargo check

  Step 2: Update query_find to delegate
    - Replace lines 200-350 with: let results = extracted_query_find_core(conn, ...);
    Verify: cargo check

  Step 3: Run existing tests
    - cargo test milestone50 — should still pass (behavior unchanged)
    Verify: cargo test

  Step 4: (Optional) Move to separate file
    - Run: repo-scout move-check extracted_query_find_core
```

**Flags:**

- `<symbol>` — target symbol (positional)
- `--action <extract|move|rename|split>` — refactoring type
- `--lines <range>` — for extract
- `--to <path|name>` — for move/rename
- `--json`, `--repo`

**Files to modify (for both):**

- `src/cli.rs` — add both commands
- `src/query/planning.rs` — add query functions
- `src/output.rs` — add print functions
- `src/main.rs` — add run handlers

---

## Phase 7: Verification — `verify-refactor`, `health --diff`

### `verify-refactor` — post-refactoring completeness check

**The question it answers:** "I just finished refactoring. Did I miss anything?"

This is the most differentiated feature in the entire plan. No existing tool does this well.

**Algorithm:**

1. Accept `--before <commit>` and `--after <commit>` (or working tree)
2. Index at both points (or use saved baseline)
3. Compare:
   - Symbols that disappeared but still have references → incomplete move/delete
   - New unresolved references → renamed/moved but not all refs updated
   - Edge changes → dependency graph shifted, verify intentional
   - Test associations → did any tested symbol lose its tests?
   - New circular dependencies introduced
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
```

**Flags:**

- `--before <commit|baseline>` — reference point
- `--after <commit>` — comparison point (default: working tree)
- `--strict` — treat warnings as errors (exit code 1)
- `--json`, `--repo`

**Files to modify:**

- `src/cli.rs` — add `VerifyRefactor(VerifyRefactorArgs)`
- `src/query/verification.rs` — new file: `verify_refactor()`, `VerificationReport` struct
- `src/output.rs` — add `print_verify_refactor()` and `print_verify_refactor_json()`
- `src/main.rs` — add `run_verify_refactor()` handler

---

### `health --diff` — baseline comparison

**The question it answers:** "Did my refactoring actually improve the codebase?"

**Extends Phase 1's `health` command:**

- `health --save-baseline` saves metrics to `.repo-scout/health-baseline.json`
- `health --diff` compares current metrics against baseline
- Shows improvements (green) and regressions (red)

**Output:**

```
Health comparison (baseline: 2025-06-15 → current):

  Largest file:     query/mod.rs  4199 → 3200 lines  ✓ improved (-24%)
  Avg fn size:      45 → 38 lines                    ✓ improved (-16%)
  Dead symbols:     12 → 8                            ✓ improved (-33%)
  Circular deps:    2 → 1                             ✓ improved (-50%)
  Max coupling:     A↔B: 17 → A↔B: 12                ✓ improved (-29%)
```

**Files to modify:**

- Extend `HealthArgs` with `--save-baseline` and `--diff`
- Add baseline save/load in `src/query/diagnostics.rs`
- Add diff rendering in `src/output.rs`

---

## Phase 8: Enhance Existing Commands

With the new infrastructure in place, enhance existing commands to surface new intelligence:

### `hotspots` enhancement

- Add `line_count` annotation to each hotspot entry
- Add `--annotate` flag to show risk scoring (fan-in \* line_count = change risk)
- No breaking changes to existing output format

### `orient` enhancement

- Now that `circular` exists, include cycle count in orient report
- Include `suggest` top-3 if the command has been run before (from cached data)

### `deps` enhancement

- Add `--coupling` flag that includes directionality scoring (reuses `coupling` logic)
- Default behavior unchanged

---

## Implementation Order & Dependencies

```
PHASE 1 — Foundation (no dependencies):
  Schema migration v3 → v4 (line_count, visibility, reserved complexity columns)
  tree              ← orientation, annotated structure
  health            ← dashboard, line-count-based
  orient            ← composite of tree + health + existing hotspots

PHASE 2 — Diagnosis (depends on Phase 1 schema):
  anatomy           ← single-file deep dive, cohesion scoring
  circular          ← cycle detection (independent)
  coupling          ← extends deps with directionality (independent)
  dead              ← unreferenced symbols (independent)

PHASE 3 — Test Intelligence (depends on Phase 1 schema):
  test-gaps         ← pre-refactoring safety gate
  test-quality      ← test health assessment

PHASE 4 — Refactoring Intelligence (depends on Phases 2 + 3):
  suggest           ← aggregates Phase 2 + 3 signals
  boundary          ← public API surface analysis

PHASE 5 — Refactoring Pre-flight (depends on Phase 4):
  extract-check     ← variable-flow analysis (most ambitious)
  move-check        ← composes refs + impact + tests-for + boundary
  rename-check      ← comprehensive rename preview
  split-check       ← composes anatomy + boundary + circular

PHASE 6 — Refactoring Support (depends on Phase 5):
  test-scaffold     ← test setup information
  safe-steps        ← safe refactoring decomposition

PHASE 7 — Verification (depends on Phase 1, independent of 2-6):
  verify-refactor   ← post-refactoring completeness check
  health --diff     ← baseline comparison

PHASE 8 — Enhancements (depends on Phases 2-7):
  hotspots enhancement
  orient enhancement
  deps enhancement
```

**Parallelization opportunities:**

- Phase 1: `tree`, `health`, and schema migration can proceed in parallel
- Phase 2: All four commands (`anatomy`, `circular`, `coupling`, `dead`) are independent
- Phase 3: `test-gaps` and `test-quality` are independent
- Phase 5: All four check commands are independent (given Phase 4)
- Phase 7: Can be built any time after Phase 1 — doesn't depend on Phases 2-6
- Phase 8: Each enhancement is independent

---

## Schema Changes Summary

| Phase   | Change                                                 | Version |
| ------- | ------------------------------------------------------ | ------- |
| Phase 1 | `line_count INTEGER` on `indexed_files`                | v4      |
| Phase 1 | `line_count`, `visibility` on `symbols_v2` (populated) | v4      |
| Phase 1 | `param_count`, `nesting_depth`, `branch_count`,        | v4      |
|         | `complexity_score` on `symbols_v2` (reserved, NULL)    |         |

**One migration.** All columns added in v4. Only `line_count` and `visibility` are populated
initially. Complexity columns exist but remain NULL until a future enhancement populates them.

---

## JSON Output Contract

All `--json` output follows these rules (consistent with existing repo-scout conventions):

1. **Deterministic**: identical input → identical output. Explicit SQL ordering, stable
   tie-breakers.
2. **Versioned**: include `"schema_version"` in top-level JSON.
3. **Cross-referenceable**: symbols identified by `file_path + symbol_name + line` consistently
   across all commands.
4. **Actionable**: include enough context for an AI agent to act without re-querying.
5. **NULL-safe**: commands handle missing optional columns gracefully. JSON output omits NULL fields
   rather than emitting `null`.

---

## New Files Summary

```
src/query/
├── mod.rs              # existing (unchanged initially)
├── diagnostics.rs      # NEW: health, anatomy, dead, circular, coupling,
│                       #      test-gaps, test-quality, suggest
├── planning.rs         # NEW: boundary, move-check, extract-check,
│                       #      rename-check, split-check, test-scaffold, safe-steps
├── verification.rs     # NEW: verify-refactor
└── orientation.rs      # NEW: tree, orient
```

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

### Complexity Metrics Population

When the diagnostic commands prove the need for finer-grained metrics, populate the reserved
`param_count`, `nesting_depth`, `branch_count`, and `complexity_score` columns. The schema is
already prepared — this requires only language adapter changes and indexer updates, no migration.

### Duplication Detection

Structural duplication (functions with similar AST shapes). High value but high implementation cost.

### Watch Mode

`repo-scout watch` — re-index on file changes, continuously update health metrics.

### IDE/Editor Integration

LSP-like integration where commands feed directly into editor UI.
