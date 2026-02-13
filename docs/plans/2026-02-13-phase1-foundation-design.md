# Phase 1: Foundation — Implementation Plan

**Date:** 2026-02-13 **Status:** Approved **Parent:** `docs/plans/refactoring-features-v3.md`
**Scope:** Schema migration v3→v4, `tree`, `health`, `circular`, `orient`

---

## Decisions

These decisions were made during design review and are final:

1. **`circular` is in Phase 1** — fills the gap in `orient`'s output, independent, algorithmically
   clean
2. **`health` v1 is lean** — largest files + largest functions only (no density, no code markers, no
   summary stats)
3. **Start the query module split now** — new code goes in `query/orientation.rs` and
   `query/diagnostics.rs`, re-exported from `query/mod.rs`
4. **Add all v4 schema columns now** — including `visibility` and reserved complexity columns
   (nullable, unpopulated), to avoid a future migration

---

## Schema Migration: v3 → v4

### Columns Added

```sql
-- Populated in Phase 1:
ALTER TABLE indexed_files ADD COLUMN line_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN line_count INTEGER;

-- Populated in future phases (nullable, reserved):
ALTER TABLE symbols_v2 ADD COLUMN visibility TEXT;
ALTER TABLE symbols_v2 ADD COLUMN param_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN nesting_depth INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN branch_count INTEGER;
ALTER TABLE symbols_v2 ADD COLUMN complexity_score INTEGER;
```

### Changes

- `src/store/schema.rs`: Bump `SCHEMA_VERSION` from 3 → 4
- `src/store/schema.rs`: Add migration function for v3 → v4 (ALTER TABLE statements)
- Migration triggers re-index (existing pattern — stale schema version causes full re-index)

### Population

- `indexed_files.line_count`: Count newlines in file source during `index_file()`. Value =
  `source.lines().count()` (or `source.as_bytes().iter().filter(|&&b| b == b'\n').count() + 1` for
  consistency with editors).
- `symbols_v2.line_count`: Computed as `end_line - start_line + 1` during symbol insertion. Uses
  existing `ExtractedSymbol.start_line` and `end_line` fields — no language adapter changes needed.

---

## Indexer Changes

**File:** `src/indexer/mod.rs`

In `index_file()`, after reading source content:

1. Compute file line count from source bytes
2. Store in `indexed_files.line_count` via updated INSERT statement
3. For each extracted symbol, compute `end_line - start_line + 1` and store in
   `symbols_v2.line_count` via updated INSERT statement

No changes to language adapters or the `LanguageAdapter` trait.

---

## Module Structure

New files created in Phase 1:

```
src/query/
├── mod.rs              # Existing — add `pub mod orientation;` and `pub mod diagnostics;`
├── orientation.rs      # NEW: tree_report(), orient_report(), supporting types
└── diagnostics.rs      # NEW: health_report(), detect_circular_deps(), supporting types
```

New modules import from `query/mod.rs` as needed (for `QueryScope`, shared helpers). `mod.rs`
re-exports new public types so callers can use `query::HealthReport` etc.

---

## Command: `tree`

### Purpose

"What is this codebase's structure, and how do the pieces connect?"

An agent currently needs `summary` + `outline` on every file + `deps` on every file to build the
same mental model. `tree` does it in one command.

### CLI Definition

```
repo-scout tree --repo <path> [--depth N] [--no-deps] [--focus <path>] [--symbols] [--json]
```

| Flag        | Type           | Default  | Description                                |
| ----------- | -------------- | -------- | ------------------------------------------ |
| `--repo`    | PathBuf        | required | Repository path                            |
| `--depth`   | u32            | 3        | Max directory depth to display             |
| `--no-deps` | bool           | false    | Plain tree without dependency arrows       |
| `--focus`   | Option<String> | None     | Zoom into a subtree (relative path)        |
| `--symbols` | bool           | false    | Expand to show individual symbols per file |
| `--json`    | bool           | false    | JSON output                                |

### Query Function

**Location:** `src/query/orientation.rs`

```rust
pub fn tree_report(db_path: &Path, args: &TreeReportArgs) -> Result<TreeReport>
```

**Data sources:**

1. `indexed_files` — file list + `line_count`
2. `symbols_v2` — symbol counts per file (`SELECT file_path, COUNT(*) ... GROUP BY file_path`)
3. `symbol_edges_v2` JOIN `symbols_v2` (both sides) — inter-file dependency arrows
   (`SELECT DISTINCT source.file_path, target.file_path FROM ...`)
4. Filesystem — directory structure (for grouping files into tree nodes)

**Algorithm:**

1. Query all indexed files with line counts and symbol counts
2. Build in-memory tree from file paths (split on `/`, construct nested `TreeNode`s)
3. Aggregate directory-level stats (total files, total symbols, total lines)
4. If `--no-deps` is false, query file-level dependency edges and attach as annotations
5. Apply `--depth` truncation, `--focus` filtering
6. If `--symbols`, query individual symbols per file and attach to leaf nodes

**Output types:**

```rust
pub struct TreeReport {
    pub root: TreeNode,
}

pub struct TreeNode {
    pub name: String,
    pub kind: TreeNodeKind,  // File or Directory
    pub line_count: Option<u32>,
    pub symbol_count: u32,
    pub children: Vec<TreeNode>,
    pub imports: Vec<String>,      // files this file depends on
    pub used_by: Vec<String>,      // files that depend on this file
    // Directory-level aggregates:
    pub total_files: u32,
    pub total_symbols: u32,
}
```

### Human-Readable Output

```
src/                               [4 modules, 156 symbols]
├── main.rs                        [1085 lines │ 15 fns, 3 structs]
│   → imports: cli, indexer, output, query, store
├── cli.rs                         [200 lines  │ 1 enum, 12 structs]
│   ← used by: main.rs
├── query/                         [1 file, 183 symbols]
│   └── mod.rs                     [4000 lines │ 45 fns, 20 structs │ ⚠ largest file]
│       → imports: store
│       ← used by: main.rs, output.rs
└── ...
```

### JSON Output

```json
{
  "schema_version": 2,
  "command": "tree",
  "tree": { ... recursive TreeNode ... }
}
```

---

## Command: `health`

### Purpose

"What parts of this codebase are largest/riskiest?"

Lean v1: two sections only — largest files and largest functions.

### CLI Definition

```
repo-scout health --repo <path> [--top N] [--threshold N] [--large-files] [--large-functions] [--json]
```

| Flag                | Type    | Default  | Description                       |
| ------------------- | ------- | -------- | --------------------------------- |
| `--repo`            | PathBuf | required | Repository path                   |
| `--top`             | u32     | 20       | Results per category              |
| `--threshold`       | u32     | 0        | Minimum line count to report      |
| `--large-files`     | bool    | false    | Only show large files section     |
| `--large-functions` | bool    | false    | Only show large functions section |
| `--json`            | bool    | false    | JSON output                       |

### Query Function

**Location:** `src/query/diagnostics.rs`

```rust
pub fn health_report(db_path: &Path, top_n: u32, threshold: u32) -> Result<HealthReport>
```

**Queries:**

1. Largest files:

   ```sql
   SELECT file_path, line_count
   FROM indexed_files
   WHERE line_count >= ?
   ORDER BY line_count DESC, file_path ASC
   LIMIT ?
   ```

2. Largest functions:
   ```sql
   SELECT file_path, symbol, line_count, start_line
   FROM symbols_v2
   WHERE kind = 'function' AND line_count >= ?
   ORDER BY line_count DESC, file_path ASC, symbol ASC
   LIMIT ?
   ```

**Output types:**

```rust
pub struct HealthReport {
    pub largest_files: Vec<FileHealth>,
    pub largest_functions: Vec<FunctionHealth>,
}

pub struct FileHealth {
    pub file_path: String,
    pub line_count: u32,
    pub symbol_count: u32,
}

pub struct FunctionHealth {
    pub file_path: String,
    pub symbol: String,
    pub line_count: u32,
    pub start_line: u32,
}
```

### Human-Readable Output

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
```

---

## Command: `circular`

### Purpose

"Are there circular dependencies, and which symbols create them?"

### CLI Definition

```
repo-scout circular --repo <path> [--max-length N] [--json]
```

| Flag           | Type    | Default  | Description                      |
| -------------- | ------- | -------- | -------------------------------- |
| `--repo`       | PathBuf | required | Repository path                  |
| `--max-length` | u32     | 10       | Only report cycles up to N files |
| `--json`       | bool    | false    | JSON output                      |

### Query Function

**Location:** `src/query/diagnostics.rs`

```rust
pub fn detect_circular_deps(db_path: &Path, max_length: u32) -> Result<CircularReport>
```

**Algorithm:**

1. Build directed file-level graph:
   ```sql
   SELECT DISTINCT src_sym.file_path AS from_file, tgt_sym.file_path AS to_file
   FROM symbol_edges_v2 e
   JOIN symbols_v2 src_sym ON e.from_symbol_id = src_sym.symbol_id
   JOIN symbols_v2 tgt_sym ON e.to_symbol_id = tgt_sym.symbol_id
   WHERE src_sym.file_path != tgt_sym.file_path
   ```
2. Run Tarjan's SCC algorithm on the file-level graph
3. Filter SCCs with >1 file and length ≤ `max_length`
4. For each SCC, extract the specific symbol edges creating the cycle:
   ```sql
   SELECT src_sym.file_path, src_sym.symbol, tgt_sym.file_path, tgt_sym.symbol, e.edge_kind
   FROM symbol_edges_v2 e
   JOIN symbols_v2 src_sym ON e.from_symbol_id = src_sym.symbol_id
   JOIN symbols_v2 tgt_sym ON e.to_symbol_id = tgt_sym.symbol_id
   WHERE src_sym.file_path IN (?...) AND tgt_sym.file_path IN (?...)
     AND src_sym.file_path != tgt_sym.file_path
   ```
5. Sort cycles by length ascending (shortest = most actionable)

**Output types:**

```rust
pub struct CircularReport {
    pub cycles: Vec<CycleDep>,
    pub total_cycles: usize,
}

pub struct CycleDep {
    pub files: Vec<String>,
    pub edges: Vec<CycleEdge>,
}

pub struct CycleEdge {
    pub from_file: String,
    pub from_symbol: String,
    pub to_file: String,
    pub to_symbol: String,
    pub edge_kind: String,
}
```

### Human-Readable Output

```
Circular dependencies:

  Cycle 1 (2 files):
    src/output.rs → src/query/mod.rs
      via: output::print_explain() calls query::explain_symbol()
    src/query/mod.rs → src/output.rs
      via: query::format_result() calls output::render_line()

  Summary: 1 cycle found
```

Or if none:

```
No circular dependencies found.
```

---

## Command: `orient`

### Purpose

"I just opened this codebase. What do I need to know?"

Single most valuable command for AI agents. Produces unified report instead of requiring 4 separate
commands.

### CLI Definition

```
repo-scout orient --repo <path> [--depth N] [--top N] [--json]
```

| Flag      | Type    | Default  | Description                                       |
| --------- | ------- | -------- | ------------------------------------------------- |
| `--repo`  | PathBuf | required | Repository path                                   |
| `--depth` | u32     | 2        | Tree depth (lower default than standalone tree)   |
| `--top`   | u32     | 5        | Items per section (lower default than standalone) |
| `--json`  | bool    | false    | JSON output                                       |

### Query Function

**Location:** `src/query/orientation.rs`

```rust
pub fn orient_report(db_path: &Path, args: &OrientReportArgs) -> Result<OrientReport>
```

**Composes internally:**

1. `tree_report()` with depth=args.depth, no-deps=false
2. `health_report()` with top=args.top
3. `hotspots()` (existing) with limit=10
4. `detect_circular_deps()` with max_length=10
5. Synthesis: compute recommendations from combined data

**Recommendations logic (simple rules, not ML):**

- Entry point suggestion: file with most outbound edges and fewest inbound (likely `main.rs`)
- "Careful around" warning: files flagged by both health (large) and hotspots (high fan-in)
- Cycle warnings: count from circular report

**Output types:**

```rust
pub struct OrientReport {
    pub tree: TreeReport,
    pub health: HealthReport,
    pub hotspots: Vec<HotspotEntry>,  // reuse existing type
    pub circular: CircularReport,
    pub recommendations: Vec<Recommendation>,
}

pub struct Recommendation {
    pub kind: RecommendationKind,  // StartExploring, CarefulAround, WellTested, CycleWarning
    pub message: String,
    pub file_path: Option<String>,
}
```

### Human-Readable Output

```
Orientation report for repo-scout:

═══ STRUCTURE ═══
[abbreviated tree output — depth 2]

═══ HEALTH ═══
[abbreviated health output — top 5]

═══ HOTSPOTS ═══
[top 10 from existing hotspots]

═══ CIRCULAR ═══
[cycle summary or "No circular dependencies"]

═══ RECOMMENDATIONS ═══
  Start exploring: src/main.rs (entry point, 15 functions)
  Careful around: src/query/mod.rs (largest file, 183 symbols, high fan-in)
```

---

## Files Modified

| File                       | Change                                                                                     |
| -------------------------- | ------------------------------------------------------------------------------------------ |
| `src/store/schema.rs`      | Migration v3→v4, bump SCHEMA_VERSION to 4                                                  |
| `src/indexer/mod.rs`       | Populate `line_count` on `indexed_files` and `symbols_v2`                                  |
| `src/cli.rs`               | Add `Tree(TreeArgs)`, `Health(HealthArgs)`, `Circular(CircularArgs)`, `Orient(OrientArgs)` |
| `src/query/mod.rs`         | Add `pub mod orientation;` and `pub mod diagnostics;`, re-exports                          |
| `src/query/orientation.rs` | **NEW**: `tree_report()`, `orient_report()`, types                                         |
| `src/query/diagnostics.rs` | **NEW**: `health_report()`, `detect_circular_deps()`, types                                |
| `src/output.rs`            | Add `print_tree()`, `print_health()`, `print_circular()`, `print_orient()` + JSON variants |
| `src/main.rs`              | Add dispatch arms + `run_tree()`, `run_health()`, `run_circular()`, `run_orient()`         |

---

## Test Plan

Following strict TDD (red → green → refactor). Each test file uses `assert_cmd` and
`common::temp_repo()`.

| Test File                        | Covers                                                                                                                                                                              |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/milestone90_schema_v4.rs` | Migration succeeds, line_count populated on both tables, re-index populates correctly                                                                                               |
| `tests/milestone91_tree.rs`      | Tree structure correct, annotations present, --depth respected, --no-deps hides arrows, --focus filters, --symbols expands, --json deterministic                                    |
| `tests/milestone92_health.rs`    | Largest files sorted correctly, largest functions sorted correctly, --top limits output, --threshold filters, --large-files/--large-functions filter sections, --json deterministic |
| `tests/milestone93_circular.rs`  | Detects known cycle, reports no cycles when clean, edge details correct, --max-length filters, --json deterministic                                                                 |
| `tests/milestone94_orient.rs`    | All sections present, recommendations generated, --depth/--top respected, --json contains all sub-reports                                                                           |

### Test Fixtures

Create `tests/fixtures/phase1/` with small multi-file repos designed to exercise:

- A repo with files of varying sizes (for health)
- A repo with known circular dependencies (for circular)
- A repo with clear directory structure and cross-file deps (for tree)
- A clean repo with no cycles (for circular negative case)

---

## Implementation Order

```
1. Schema migration v3 → v4          (foundation — everything depends on this)
   └── Tests: milestone90

2. Indexer: line_count population     (depends on schema)
   └── Tests: milestone90 (extended)

3. health                             (simplest query, validates schema + line_count work)
   └── Tests: milestone92

4. circular                           (independent algorithm, no dependency on health/tree)
   └── Tests: milestone93

5. tree                               (more complex, uses same data as health)
   └── Tests: milestone91

6. orient                             (composite — depends on health, tree, circular, hotspots)
   └── Tests: milestone94
```

### Parallelization

Steps 3, 4, and 5 are independent once the schema + indexer work is done. They can be built in any
order. `orient` must come last.

---

## Dogfooding

After each command is implemented:

```bash
cargo run -- index --repo .
cargo run -- <new-command> --repo .
cargo run -- <new-command> --repo . --json
```

After `orient` is complete, run `orient` on repo-scout itself and verify the output is useful and
accurate. Include transcript in PR notes.

---

## What This Plan Does NOT Include

- `visibility` population (column added but NULL — future phase)
- Complexity metrics population (columns added but NULL — future phase)
- `health --save-baseline` / `health --diff` (Phase 7 in v3)
- Code markers (TODO/FIXME scanning — deferred from health v1)
- Symbol density section in health (deferred)
- Any Phase 2+ features from v3
