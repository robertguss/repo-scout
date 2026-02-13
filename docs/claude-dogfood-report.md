# repo-scout Dogfood Report: Claude's Honest Assessment

> Written by Claude (Opus 4.6) after building, indexing, and running every command against the
> repo-scout codebase itself. This is my genuine perspective on what works, what doesn't, and what I
> actually need to be maximally effective as a coding agent.

---

## Part 1: What Exists Today (Command-by-Command Review)

### `index` — Fast and Correct, Unclear Messaging

Indexing the repo took ~2 seconds in release mode. The incremental hashing works — re-indexing is
nearly instant. The output is clean:

```
indexed_files: 7
skipped_files: 291
```

**Problem:** "skipped_files: 291" is misleading. On a fresh index, nothing was "skipped" — those 291
files are markdown, shell scripts, fixtures, and other non-source files that the indexer correctly
ignores. "Skipped" implies something went wrong or was deferred. Better labels:

- `source_files: 7` / `non_source_files: 291`
- Or: `indexed: 7` / `ignored: 291` (with a note about why)

**Impact:** Low severity but high frequency — every user sees this on first run and wonders "why
were 291 files skipped?"

---

### `find <symbol>` — Solid Foundation, Works as Expected

Found `index_repository` at its definition and import locations. The `[ast_definition ast_exact]`
labels are helpful — I can immediately tell this is a real definition, not a text match.

**What works well:**

- Exact matches return fast with precise file:line:column
- AST-backed results are clearly labeled
- JSON output is clean and parseable

**What's missing:**

- No fuzzy/partial matching. If I misremember a name (`index_repo` instead of `index_repository`), I
  get 0 results with no suggestions. A "did you mean?" or fuzzy fallback would save round-trips.
- No way to filter by kind (function, struct, trait, module). When I search for `text`, I get module
  definitions mixed with functions. A `--kind function` filter would help.

---

### `refs <symbol>` — The Most Immediately Useful Command

For `LanguageAdapter`, returned 62 results across source, tests, and docs. The substring matching
(finding `GoLanguageAdapter` when searching `LanguageAdapter`) is genuinely smart.

**What makes this valuable for me:**

- Complete usage map in a single call — I don't need multiple rounds of grep
- Provenance labels (`ast_exact` vs `text_fallback`) let me gauge confidence
- Cross-file, cross-language results

**What could improve:**

- Results include markdown docs and planning files (`agents/plans/...`, `docs/dogfood-log.md`).
  These are noise when I'm trying to understand code structure. A `--code-only` filter (like
  `context` has) would help. Or `--exclude-docs`.
- 62 results is a lot of output. The results aren't grouped — source files, test files, and docs are
  interleaved. Grouping by category (definitions / source references / test references / docs) would
  make the output scannable.

---

### `impact <symbol>` — Useful but Narrow

For `index_repository` → found 1 caller (`run_index`). Correct but thin. For `LanguageAdapter` →
found 0 results. Technically correct (a trait isn't "called"), but unhelpful.

**The gap:** `impact` only shows `called_by` relationships. For traits, I want to see implementors.
For structs, I want to see where they're constructed. For modules, I want to see who imports them.
The command is useful for functions but doesn't generalize to other symbol kinds.

**Suggestion:** Either expand `impact` to handle different relationship types based on symbol kind,
or document clearly that it's function-centric.

---

### `context --task "..."` — Great Concept, Flat Scoring

I tested three tasks:

- "understand how indexing works" → 6 results, all scored 0.87
- "add a new CLI command" → 6 results, all scored 0.87
- "understand error handling patterns" → 10 results, 9 at 0.87, 1 at 0.83

**The problem:** The scoring is too flat. When I ask about "indexing", a function literally named
`index_file_indexes_changed_source_file` scores the same (0.87) as
`milestone48_audit_index_reflects_current_artifacts`. The former is directly about indexing; the
latter tangentially mentions it. Without score differentiation, I can't prioritize what to read
first.

**What I actually want from `context`:** "Given this task description, show me the files and
functions I should read, in priority order." The current token-overlap approach treats all keyword
matches equally. A tf-idf or BM25 scoring model would differentiate between "this function IS the
thing" vs "this function MENTIONS the thing."

---

### `tests-for <symbol>` — Under-Discovers

For `index_repository`, found only 1 test file (`milestone42_src_structure.rs`). But
`milestone2_indexing.rs` and `milestone5_e2e.rs` clearly exercise indexing. They just don't contain
the literal string "index_repository" — they invoke it through the CLI binary.

**The gap:** `tests-for` appears to rely on direct symbol mentions in test files. But in integration
test suites (like this one, using `assert_cmd`), tests invoke functionality through the binary, not
by calling functions directly. The test file says
`Command::cargo_bin("repo-scout")... .arg("index")`, not `index_repository()`.

**Suggestion:** Consider a secondary heuristic: if a test file invokes CLI commands that map to
functions (e.g., `"index"` → `run_index` → `index_repository`), count that as coverage. This is a
hard problem, but even a simple CLI-arg-to-handler mapping would improve recall significantly.

---

### `verify-plan --changed-file` — The Most Actionable Command

Given `src/indexer/mod.rs`, returned 9 targeted test commands with copy-pasteable `cargo test`
invocations. This is exactly what I need after making a change.

**What makes this excellent:**

- Directly actionable output — I can copy-paste the test commands
- Explains WHY each test is relevant (e.g., "references changed symbol 'LanguageAdapter'")
- Always includes `cargo test (full_suite)` as a safety net

**What could improve:**

- The targeted tests are identified by file-level symbol overlap, not by actual coverage. Some
  targets are false positives (e.g., `milestone49_rust_hardening` testing `HashSet` — the fact that
  `src/indexer/mod.rs` imports `HashSet` doesn't mean that test is relevant to my change).
- Accepting `--since <commit>` to automatically derive changed files from git would eliminate manual
  file listing.

---

### `diff-impact --changed-file` — Comprehensive but Overwhelming

104 results for a single file change. The distance-based traversal (0, 1, 2) is the right model, and
the results are technically correct. But 104 results is too many to be actionable.

**What works:**

- Distance metric clearly shows blast radius layers
- Confidence scores decay appropriately with distance
- Test targets are identified separately from impacted symbols

**What needs work:**

- Without `--max-results`, the output floods. Default should probably be 20-30 with explicit opt-in
  for full output.
- All distance-0 symbols are included even though many are internal helpers. A `--public-only` or
  minimum-importance filter would help.
- The `--changed-line` option exists but I didn't test it — if it narrows to only symbols on those
  lines, that would be far more precise than whole-file analysis.

---

### `explain <symbol>` — Good Orientation, Missing Terminal Snippets

Signature + call graph edges. This is my "what is this thing?" command.

**Bug found:** `--include-snippets` works in `--json` mode but NOT in terminal mode. The terminal
renderer silently drops the snippet. This is the most impactful bug I found — as an agent, I
primarily use terminal output, and the snippet is the most valuable part of `explain`.

**What the JSON output gives me (correctly):**

```json
{
  "signature": "fn extract_with_adapter(",
  "snippet": "fn extract_with_adapter(\n    file_path: &str, ..."
}
```

**What terminal output shows (missing snippet):**

```
src/indexer/mod.rs:734:4 extract_with_adapter (function) [ast_definition graph_exact 1.00]
signature: fn extract_with_adapter(
inbound: called_by=1 ...
outbound: calls=2 ...
```

No snippet. This needs fixing.

---

### `status` — Too Bare

Only shows DB path and schema version. Doesn't tell me:

- How many files are indexed
- Which languages were detected
- When the index was last updated
- Whether the index is stale (files changed since last index)

For an agent, `status` should be my "is the index fresh?" check before running queries.

---

### Help Text — No Descriptions

```
Commands:
  index
  status
  find
  refs
  impact
  context
  tests-for
  verify-plan
  diff-impact
  explain
```

No descriptions. Every subcommand should have a one-liner. An agent encountering this tool for the
first time has to guess what each command does or `--help` each one individually.

---

## Part 2: What I Genuinely Need (New Capabilities)

These are ordered by how much they would change my daily effectiveness. I've thought hard about each
one — these aren't wishes, they're pain points I hit in virtually every session.

### Priority 1: `snippet <symbol>` — Targeted Source Extraction

**The pain:** My biggest context-window cost is reading entire files to find one function.
`query/mod.rs` is 3,611 lines. If I need to understand `add_changed_file_target_step`, I read 3,611
lines to get ~30. That's a 99% waste ratio.

**What I want:**

```
repo-scout snippet add_changed_file_target_step --repo .
```

Returns ONLY the source code of that function — its signature, body, and nothing else. Maybe 20-40
lines. Maybe with a `--context N` flag to include N lines above/below for surrounding context.

**Why this matters:** In a typical session, I read 10-20 functions across a codebase. If each file
averages 500 lines but each function averages 30 lines, I'm consuming 10,000 lines when I only
need 600. `snippet` would reduce my context consumption by ~94%.

**The data already exists.** The AST extraction captures start_line and end_line for every
definition. The `explain --json --include-snippets` command already retrieves snippets. This is just
exposing it as a first-class command.

**Variations that would help:**

- `snippet <symbol> --with-callers` — show the function + inline its direct callers' signatures
- `snippet <symbol> --with-types` — include the definitions of types used in the signature

---

### Priority 2: `outline <file>` — File Structure Without Implementation

**The pain:** When I first open a file, I need to understand its structure — what functions exist,
what types are defined, what's public vs private. Currently I read the whole file, burning context
on implementation details I don't yet care about.

**What I want:**

```
repo-scout outline src/query/mod.rs --repo .
```

Returns a condensed view: every function signature, struct/enum definition, trait impl header, and
module declaration. NO function bodies. Just the skeleton.

For a 3,611-line file, this might produce ~150 lines. That's a 24x reduction while giving me
everything I need to orient.

**Example output:**

```
src/query/mod.rs (3611 lines, 47 functions, 8 structs, 2 enums)

  pub struct FindResult { ... }           line 15
  pub struct RefsResult { ... }           line 28
  pub fn run_find(...) -> ...             line 52
  pub fn run_refs(...) -> ...             line 134
  fn rank_find_results(...) -> ...        line 201
  ...
```

**Why this matters more than you might think:** Outline isn't just for orientation — it's for
PATTERN DISCOVERY. When I see all function signatures at once, I can identify naming conventions,
parameter patterns, return type patterns, and module organization. This is how I learn "how does
this codebase do things?" so I can write code that matches.

---

### Priority 3: `summary` — Whole-Repo Structural Overview

**The pain:** At the start of every session on an unfamiliar codebase, I spend 5-10 turns just
orienting: reading README, exploring directories, sampling files. This "cold start" problem costs
significant context and time.

**What I want:**

```
repo-scout summary --repo .
```

Returns a machine-readable project overview:

- Language breakdown (7 Rust source files, 15 TypeScript fixtures, etc.)
- Module structure (src/indexer/, src/query/, src/store/)
- Entry points (main.rs, lib.rs)
- Public API surface (exported functions/types)
- Key statistics (total definitions, total edges, most-connected symbols)
- Test structure (where tests live, how many, what patterns)

**This replaces:** My current multi-step orientation: `ls`, `Read README`, `Glob **/*.rs`,
`Read Cargo.toml`, `Read src/main.rs` — all condensed into one command.

---

### Priority 4: `--since <commit>` for diff-impact and verify-plan

**The pain:** Currently I have to manually list changed files:

```
diff-impact --changed-file src/a.rs --changed-file src/b.rs --changed-file src/c.rs --repo .
```

In practice, I already know what changed because I just made the changes (or I can ask git). Having
to manually enumerate files is error-prone and tedious.

**What I want:**

```
repo-scout diff-impact --since HEAD~1 --repo .
repo-scout verify-plan --since HEAD~1 --repo .
repo-scout diff-impact --unstaged --repo .
```

Reads changed files directly from `git diff`. This is the natural workflow: change code → ask "what
did I break?" → get answer. No manual file enumeration.

---

### Priority 5: `callers <symbol>` and `callees <symbol>` — Directed Graph Navigation

**The pain:** `impact` gives me callers. `explain` gives me call counts but not the actual symbols.
There's no way to ask "what does this function call?" without reading its source.

**What I want:**

```
repo-scout callers index_repository --repo .
# → run_index (src/main.rs:70)

repo-scout callees index_repository --repo .
# → index_file, prune_stale_file_rows, replay_deferred_edges, ...
```

**Why this matters:** When I'm tracing execution flow ("what happens when the user runs `index`?"),
I need to walk the call graph in both directions. Currently I can walk UP (via `impact`) but not
DOWN. Walking down requires reading source, which burns context.

`callees` is particularly valuable for understanding unfamiliar functions without reading their
implementation. If I see a function calls `validate_schema`, `insert_rows`, and
`commit_transaction`, I already understand its purpose without reading a single line of its body.

---

### Priority 6: `deps <file>` — File-Level Dependency Graph

**What I want:**

```
repo-scout deps src/indexer/mod.rs --repo .

depends_on:
  src/indexer/files.rs (discover_source_files)
  src/indexer/languages/mod.rs (LanguageAdapter, ExtractionUnit)
  src/indexer/text.rs (extract_text_occurrences)
  src/store/mod.rs (ensure_store)

depended_on_by:
  src/main.rs (index_repository)
```

**Why:** When I'm about to modify a file, I need to know its dependency neighborhood. What modules
does it use? What modules use it? This tells me both "what do I need to understand before changing
this?" and "what might break when I change this?" — at the file level rather than the symbol level.

---

### Priority 7: `hotspots` — Most-Connected Symbols

**What I want:**

```
repo-scout hotspots --repo . --limit 10

  1. ensure_store        fan_in=12  fan_out=3  file=src/store/mod.rs
  2. index_repository    fan_in=8   fan_out=5  file=src/indexer/mod.rs
  3. run_find            fan_in=3   fan_out=7  file=src/query/mod.rs
  ...
```

**Why:** Hotspots are the "load-bearing walls" of a codebase. When I'm new to a repo, knowing the 10
most-connected symbols tells me where to focus my understanding. These are the functions that, if I
modify them, have the highest blast radius. They're also the functions most likely to be relevant to
whatever task I'm working on.

---

### Priority 8: `path <from> <to>` — Call Path Discovery

**What I want:**

```
repo-scout path run_index ensure_store --repo .

  run_index → index_repository → ensure_store
  (2 hops)
```

**Why:** When someone says "does changing `ensure_store` affect the `explain` command?", I currently
have to manually trace through multiple `impact` and `refs` calls. A direct path query would answer
this in one call.

---

### Priority 9: `related <symbol>` — Neighborhood Discovery

**What I want:**

```
repo-scout related index_file --repo .

  siblings (same module):
    index_repository, file_is_unchanged, prepare_file_data, clear_file_rows
  shares_callers_with:
    prune_stale_file_rows, replay_deferred_edges
  shares_callees_with:
    insert_symbols, insert_references
```

**Why:** When I find one relevant function, I usually need its "neighborhood" — the functions that
work alongside it. Currently I discover this by reading the whole file (expensive) or by doing
multiple `refs`/`impact` queries (slow). `related` would give me the structural neighborhood in one
call.

---

## Part 3: Improvements to Existing Commands

### 3.1 Fix `--include-snippets` in Terminal Mode

This is a bug. The snippet data exists (proven by JSON output) but the terminal renderer doesn't
display it. This is my highest-priority bug fix because `explain` with snippets is the closest thing
to `snippet` that exists today.

### 3.2 Differentiate `context` Scoring

The 0.87-flat problem. Every result scores nearly identically regardless of actual relevance. This
makes `context` unreliable for prioritization.

**Possible approaches:**

- tf-idf weighting: rare terms in the task description should boost matches more than common terms
- Name-match bonus: if the task says "indexing" and the function is named `index_file`, that should
  score higher than a function whose BODY mentions "index"
- Kind-based boost: function definitions should generally rank above test functions for
  understanding tasks

### 3.3 Improve `tests-for` Recall

Two concrete improvements:

1. **CLI-command mapping:** If a test file invokes `repo-scout index`, and `index` maps to
   `run_index` which calls `index_repository`, that test covers `index_repository`.
2. **Transitive coverage:** If `index_repository` calls `index_file`, tests for `index_file` also
   partially cover `index_repository`.

### 3.4 Enrich `status` Output

```
index_path: ./.repo-scout/index.db
schema_version: 3
last_indexed: 2025-02-13T14:23:00Z
source_files: 7 (rust: 7)
definitions: 1230
references: 3450
text_occurrences: 281978
edges: 847
stale_files: 0
```

### 3.5 Add Descriptions to Help Text

```
Commands:
  index        Index a repository into the local SQLite database
  status       Show index status and health
  find         Find symbol definitions by name
  refs         Find all references to a symbol
  impact       Show what depends on a symbol (callers, importers)
  context      Find code relevant to a task description
  tests-for    Find test files that cover a symbol
  verify-plan  Suggest test commands after changing files
  diff-impact  Analyze blast radius of file changes
  explain      Show symbol details: signature, call graph, source
```

### 3.6 Add `--code-only` to `refs`

Currently, `refs LanguageAdapter` returns 62 results including hits in markdown planning docs. When
I'm doing code navigation, I want `--code-only` to filter to source/test files only.

### 3.7 Group `refs` Output by Category

Instead of a flat list of 62 results, group them:

```
Definitions (2):
  src/indexer/languages/mod.rs:51:11 LanguageAdapter [ast_definition]
  src/indexer/mod.rs:6:1 LanguageAdapter [import]

Source references (12):
  src/indexer/languages/go.rs:7:73 ...
  ...

Test references (2):
  tests/milestone14_adapter.rs:114:44 ...
  ...

Documentation mentions (4):
  agents/plans/repo-scout-phase3-execplan.md:792:15 ...
  ...
```

### 3.8 Default `diff-impact` to a Reasonable Limit

104 results for one file is too much. Default `--max-results` to 30 and require explicit opt-in for
unlimited output. Or add a `--brief` mode that shows only distance-0 and distance-1.

---

## Part 4: Output Format Considerations

### 4.1 Every Token Costs Me Context Window

This is the fundamental constraint that should drive all output design. When I invoke repo-scout via
Bash, every character of output goes into my context window. Verbose output directly reduces my
ability to do other work in the same session.

**Principle:** Default output should be information-dense. No decorative separators, no redundant
labels, no padding. Every line should carry signal.

### 4.2 The `--json` vs Terminal Divide

Currently `--json` has more information than terminal (e.g., snippets). This should be flipped —
terminal is what agents use via Bash, and it should be the RICHER format. JSON is for programmatic
consumption where the consumer can select what it needs.

At minimum, feature parity: if `--json` includes snippets, terminal should too.

### 4.3 Consider a `--compact` Mode

For agents that are tight on context, a `--compact` mode that strips everything except
file:line:symbol would be useful:

```
repo-scout refs LanguageAdapter --repo . --compact
src/indexer/languages/mod.rs:51 LanguageAdapter
src/indexer/mod.rs:6 LanguageAdapter
src/indexer/languages/go.rs:7 LanguageAdapter
...
```

---

## Part 5: The Bigger Picture — What I'm Actually Doing When I Work

To understand why these features matter, here's what my work pattern actually looks like:

### Phase 1: Orient (10-20% of session)

**Current:** Read README → `ls` → `Glob` for key files → Read 3-5 files → build mental model **With
repo-scout:** `summary` → `outline` on 2-3 key files → ready to work

### Phase 2: Locate (5-10% of session)

**Current:** `Grep` for symbol → Read file containing it → Read related files **With repo-scout:**
`find` + `explain` + `snippet` → precise understanding without full-file reads

### Phase 3: Understand Dependencies (10-15% of session)

**Current:** Multiple rounds of `Grep` → Read each file → manually trace call chains **With
repo-scout:** `callees` → `callers` → `deps` → complete picture in 3 commands

### Phase 4: Make Changes (30-40% of session)

This is where I actually write code. repo-scout doesn't help here (nor should it).

### Phase 5: Verify (15-20% of session)

**Current:** Guess which tests to run → run full suite as safety net → wait **With repo-scout:**
`verify-plan --since HEAD` → run only targeted tests → full suite only if targeted tests pass

### Phase 6: Review Impact (5-10% of session)

**Current:** Manually review changed files → grep for potential breakage → hope I didn't miss
anything **With repo-scout:** `diff-impact --since HEAD` → comprehensive blast radius → known
unknowns

**The opportunity:** Phases 1-3 and 5-6 are where repo-scout saves time. That's potentially 40-55%
of a session's work being done more efficiently, with less context consumption, and higher
confidence.

---

## Part 6: Single Most Impactful Change

If I could only have one thing from this entire document, it would be **`snippet`**.

The reason is simple math. In a typical session:

- I read ~15 functions across ~8 files
- Average file length: ~600 lines
- Average function length: ~30 lines
- Total lines read: 15 × 600 = 9,000
- Total lines needed: 15 × 30 = 450
- Waste ratio: 95%

`snippet` would make me **20x more context-efficient** for targeted code understanding. Nothing else
comes close to that multiplier.

---

## Appendix: Raw Command Outputs

<details>
<summary>Index output</summary>

```
$ repo-scout index --repo .
index_path: ./.repo-scout/index.db
schema_version: 3
indexed_files: 7
skipped_files: 291
```

</details>

<details>
<summary>find index_repository</summary>

```
$ repo-scout find index_repository --repo .
command: find
query: index_repository
results: 2
src/indexer/mod.rs:68:8 index_repository [ast_definition ast_exact]
src/main.rs:10:1 index_repository [ast_definition ast_exact]
```

</details>

<details>
<summary>refs LanguageAdapter (62 results)</summary>

```
$ repo-scout refs LanguageAdapter --repo .
command: refs
query: LanguageAdapter
results: 62
src/indexer/languages/go.rs:7:73 LanguageAdapter [exact_symbol_name text_fallback]
src/indexer/languages/go.rs:37:6 LanguageAdapter [exact_symbol_name text_fallback]
src/indexer/languages/mod.rs:51:11 LanguageAdapter [exact_symbol_name text_fallback]
... (truncated for brevity)
```

</details>

<details>
<summary>explain with --include-snippets --json (working)</summary>

```json
{
  "snippet": "fn extract_with_adapter(\n    file_path: &str,\n    source: &str,\n) -> ..."
}
```

</details>

<details>
<summary>explain with --include-snippets terminal (bug: no snippet shown)</summary>

```
$ repo-scout explain extract_with_adapter --repo . --include-snippets
command: explain
query: extract_with_adapter
results: 1
src/indexer/mod.rs:734:4 extract_with_adapter (function) [ast_definition graph_exact 1.00]
signature: fn extract_with_adapter(
inbound: called_by=1 imported_by=0 implemented_by=0 contained_by=0
outbound: calls=2 imports=0 implements=0 contains=0
```

Note: no snippet in output despite --include-snippets flag.

</details>

<details>
<summary>verify-plan for indexer changes</summary>

```
$ repo-scout verify-plan --changed-file src/indexer/mod.rs --repo .
command: verify-plan
changed_files: 1
results: 9
cargo test --test milestone14_adapter (targeted) ...
cargo test --test milestone32_semantic_contracts (targeted) ...
cargo test --test milestone41_process_contracts (targeted) ...
cargo test --test milestone42_src_structure (targeted) ...
cargo test --test milestone47_process_policy (targeted) ...
cargo test --test milestone48_audit_closure (targeted) ...
cargo test --test milestone49_rust_hardening (targeted) ...
cargo test --test milestone6_schema_migration (targeted) ...
cargo test (full_suite) ...
```

</details>

---

## Summary: Prioritized Action Items

### Bugs (fix first)

1. `--include-snippets` not rendering in terminal mode (`explain` command)

### Quick Wins (high value, low effort)

2. Add descriptions to `--help` subcommand list
3. Change "skipped_files" to "non_source_files" or "ignored" in index output
4. Enrich `status` with file counts, languages, staleness check
5. Add `--code-only` flag to `refs`
6. Set a default `--max-results` for `diff-impact` (suggest 30)

### New Commands (high value, medium effort)

7. **`snippet <symbol>`** — extract just a function/type's source code
8. **`outline <file>`** — file structure without implementation bodies
9. **`summary`** — whole-repo structural overview

### New Commands (high value, higher effort)

10. **`--since <commit>`** for `diff-impact` and `verify-plan`
11. **`callers` / `callees`** — directed call graph navigation
12. **`deps <file>`** — file-level dependency graph
13. **`hotspots`** — most-connected symbols

### Scoring/Quality Improvements

14. Differentiate `context` relevance scoring (replace flat 0.87)
15. Improve `tests-for` recall (CLI-command mapping, transitive coverage)
16. Group `refs` output by category (source / test / docs)

### Stretch Goals

17. `path <from> <to>` — call path discovery
18. `related <symbol>` — structural neighborhood
19. `--compact` output mode for context-constrained agents
20. Fuzzy matching / "did you mean?" for `find`
