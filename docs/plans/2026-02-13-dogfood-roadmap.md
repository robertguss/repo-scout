# Dogfood Roadmap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan
> task-by-task.

**Goal:** Implement all 20 items from the dogfood report — bugs, quick wins, new commands, scoring
improvements, and stretch goals.

**Architecture:** Each item is a self-contained task following strict TDD. Changes touch
`src/cli.rs` (arg definitions), `src/query/mod.rs` (query logic), `src/output.rs` (rendering),
`src/main.rs` (command dispatch), and `src/indexer/mod.rs` (indexer output). New commands reuse
existing `symbols_v2` and `symbol_edges_v2` tables — no schema changes needed.

**Tech Stack:** Rust 2024, clap derive, rusqlite, serde_json, tree-sitter (existing), git2 (new
dependency for `--since`)

---

## Phase 1: Bug Fix

### Task 1: Fix `--include-snippets` in terminal mode for `explain`

**Files:**

- Modify: `src/output.rs:471-505` (add snippet printing)
- Test: `tests/milestone_dogfood_explain_snippets.rs` (new)

**Step 1: Write the failing test**

Create `tests/milestone_dogfood_explain_snippets.rs`:

```rust
mod common;

use serde_json::Value;

#[test]
fn explain_include_snippets_terminal_shows_snippet() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n",
    );
    let _index_out = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let terminal_out = common::run_stdout(&[
        "explain", "greet", "--repo", repo.path().to_str().unwrap(), "--include-snippets",
    ]);
    assert!(
        terminal_out.contains("snippet:"),
        "terminal output should contain 'snippet:' label, got:\n{terminal_out}"
    );
    assert!(
        terminal_out.contains("pub fn greet"),
        "terminal output should contain the function source, got:\n{terminal_out}"
    );
}

#[test]
fn explain_include_snippets_terminal_matches_json_snippet() {
    let repo = common::temp_repo();
    common::write_file(
        repo.path(),
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    );
    let _index_out = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let terminal_out = common::run_stdout(&[
        "explain", "add", "--repo", repo.path().to_str().unwrap(), "--include-snippets",
    ]);
    let json_out = common::run_stdout(&[
        "explain", "add", "--repo", repo.path().to_str().unwrap(), "--include-snippets", "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    let json_snippet = payload["results"][0]["snippet"].as_str().expect("snippet in json");

    // Terminal should contain the same snippet content
    assert!(
        terminal_out.contains(json_snippet.lines().next().unwrap()),
        "terminal should contain first line of JSON snippet"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test milestone_dogfood_explain_snippets -- --nocapture` Expected: FAIL — terminal
output missing `snippet:` label

**Step 3: Write minimal implementation**

In `src/output.rs`, add snippet printing after the outbound line (after line 503):

```rust
// In print_explain(), after the outbound println!, add:
        if let Some(snippet) = &result.snippet {
            println!("snippet:");
            for line in snippet.lines() {
                println!("  {line}");
            }
        }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test milestone_dogfood_explain_snippets -- --nocapture` Expected: PASS

**Step 5: Run full suite**

Run: `cargo test` Expected: All existing tests pass

**Step 6: Commit**

```bash
git add tests/milestone_dogfood_explain_snippets.rs src/output.rs
git commit -m "RED+GREEN: fix --include-snippets not rendering in terminal explain output"
```

---

## Phase 2: Quick Wins

### Task 2: Add descriptions to `--help` subcommand list

**Files:**

- Modify: `src/cli.rs:14-25` (add `#[command(about = "...")]` to each variant)
- Test: `tests/milestone_dogfood_help_text.rs` (new)

**Step 1: Write the failing test**

Create `tests/milestone_dogfood_help_text.rs`:

```rust
mod common;

#[test]
fn help_text_shows_subcommand_descriptions() {
    let output = common::run_stdout(&["--help"]);
    assert!(output.contains("Index a repository"), "missing Index description:\n{output}");
    assert!(output.contains("Show index status"), "missing Status description:\n{output}");
    assert!(output.contains("Find symbol definitions"), "missing Find description:\n{output}");
    assert!(output.contains("Find all references"), "missing Refs description:\n{output}");
    assert!(output.contains("Show what depends on"), "missing Impact description:\n{output}");
    assert!(output.contains("Find code relevant to"), "missing Context description:\n{output}");
    assert!(output.contains("Find test files"), "missing TestsFor description:\n{output}");
    assert!(output.contains("Suggest test commands"), "missing VerifyPlan description:\n{output}");
    assert!(output.contains("Analyze blast radius"), "missing DiffImpact description:\n{output}");
    assert!(output.contains("Show symbol details"), "missing Explain description:\n{output}");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test milestone_dogfood_help_text -- --nocapture` Expected: FAIL — no descriptions
in help output

**Step 3: Write minimal implementation**

In `src/cli.rs`, replace the `Command` enum (lines 13-25):

```rust
#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Index a repository into the local SQLite database")]
    Index(RepoArgs),
    #[command(about = "Show index status and health")]
    Status(RepoArgs),
    #[command(about = "Find symbol definitions by name")]
    Find(FindArgs),
    #[command(about = "Find all references to a symbol")]
    Refs(RefsArgs),
    #[command(about = "Show what depends on a symbol (callers, importers)")]
    Impact(QueryArgs),
    #[command(about = "Find code relevant to a task description")]
    Context(ContextArgs),
    #[command(about = "Find test files that cover a symbol")]
    TestsFor(TestsForArgs),
    #[command(about = "Suggest test commands after changing files")]
    VerifyPlan(VerifyPlanArgs),
    #[command(about = "Analyze blast radius of file changes")]
    DiffImpact(DiffImpactArgs),
    #[command(about = "Show symbol details: signature, call graph, source")]
    Explain(ExplainArgs),
}
```

**Step 4: Run tests**

Run: `cargo test --test milestone_dogfood_help_text -- --nocapture` Expected: PASS

**Step 5: Run full suite + commit**

Run: `cargo test`

```bash
git add src/cli.rs tests/milestone_dogfood_help_text.rs
git commit -m "GREEN: add descriptions to --help subcommand list"
```

---

### Task 3: Rename `skipped_files` to `non_source_files` in index output

**Files:**

- Modify: `src/indexer/mod.rs:17-21` (rename field)
- Modify: `src/output.rs:84-94` (update label)
- Modify: `src/main.rs:74-75` (update field reference)
- Test: `tests/milestone_dogfood_index_labels.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn index_output_uses_non_source_files_label() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::write_file(repo.path(), "README.md", "# Hello\n");
    let output = common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    assert!(
        output.contains("non_source_files:"),
        "should use 'non_source_files' label, got:\n{output}"
    );
    assert!(
        !output.contains("skipped_files:"),
        "should NOT use 'skipped_files' label, got:\n{output}"
    );
}
```

**Step 2: Run test — expected FAIL**

Run: `cargo test --test milestone_dogfood_index_labels -- --nocapture`

**Step 3: Implement**

In `src/indexer/mod.rs`, rename `skipped_files` → `non_source_files` in `IndexSummary` and all
usages.

In `src/output.rs:93`, change `println!("skipped_files: {skipped_files}")` to
`println!("non_source_files: {non_source_files}")`.

Update `src/main.rs:75` to use `summary.non_source_files`.

**Step 4: Run tests — expected PASS**

Run: `cargo test --test milestone_dogfood_index_labels -- --nocapture`

**Step 5: Fix any existing tests that assert `skipped_files`**

Search: `cargo test 2>&1 | grep -i "skipped_files\|FAIL"` — fix any broken assertions.

**Step 6: Full suite + commit**

Run: `cargo test`

```bash
git add src/indexer/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_index_labels.rs
git commit -m "GREEN: rename skipped_files to non_source_files in index output"
```

---

### Task 4: Enrich `status` with file counts, languages, staleness

**Files:**

- Modify: `src/query/mod.rs` (add `status_summary` function)
- Modify: `src/output.rs:96-99` (expand `print_status`)
- Modify: `src/main.rs:80-84` (pass more data to print_status)
- Test: `tests/milestone_dogfood_status.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn status_shows_enriched_output() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\npub fn world() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["status", "--repo", repo.path().to_str().unwrap()]);

    assert!(output.contains("source_files:"), "missing source_files count:\n{output}");
    assert!(output.contains("definitions:"), "missing definitions count:\n{output}");
    assert!(output.contains("references:"), "missing references count:\n{output}");
    assert!(output.contains("edges:"), "missing edges count:\n{output}");
}
```

**Step 2: Run test — expected FAIL**

Run: `cargo test --test milestone_dogfood_status -- --nocapture`

**Step 3: Implement**

Add to `src/query/mod.rs`:

```rust
pub struct StatusSummary {
    pub source_files: usize,
    pub definitions: usize,
    pub references: usize,
    pub text_occurrences: usize,
    pub edges: usize,
    pub languages: Vec<(String, usize)>,  // (language, file_count)
}

pub fn status_summary(db_path: &Path) -> anyhow::Result<StatusSummary> {
    let connection = Connection::open(db_path)?;
    let source_files: usize = connection.query_row(
        "SELECT COUNT(*) FROM indexed_files", [], |row| row.get(0)
    )?;
    let definitions: usize = connection.query_row(
        "SELECT COUNT(*) FROM symbols_v2", [], |row| row.get(0)
    )?;
    let references: usize = connection.query_row(
        "SELECT COUNT(*) FROM ast_references", [], |row| row.get(0)
    )?;
    let text_occurrences: usize = connection.query_row(
        "SELECT COUNT(*) FROM text_occurrences", [], |row| row.get(0)
    )?;
    let edges: usize = connection.query_row(
        "SELECT COUNT(*) FROM symbol_edges_v2", [], |row| row.get(0)
    )?;
    let mut lang_stmt = connection.prepare(
        "SELECT language, COUNT(DISTINCT file_path) FROM symbols_v2 GROUP BY language ORDER BY language"
    )?;
    let languages: Vec<(String, usize)> = lang_stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
    })?.filter_map(|r| r.ok()).collect();

    Ok(StatusSummary { source_files, definitions, references, text_occurrences, edges, languages })
}
```

Update `src/output.rs` `print_status` to accept and print `StatusSummary`.

Update `src/main.rs` `run_status` to call `status_summary` and pass to `print_status`.

**Step 4: Run tests — expected PASS**

**Step 5: Full suite + commit**

```bash
git add src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_status.rs
git commit -m "GREEN: enrich status command with file counts, languages, and statistics"
```

---

### Task 5: Add `--code-only` flag to `refs` (already exists)

**Files:**

- Verify: `src/cli.rs:64-65` — `code_only` already exists on `RefsArgs`
- Test: `tests/milestone_dogfood_refs_code_only.rs`

**Step 1: Write the test to confirm it works**

```rust
mod common;

#[test]
fn refs_code_only_excludes_non_source_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn helper() {}\npub fn caller() { helper(); }\n");
    common::write_file(repo.path(), "docs/notes.md", "We use helper() in the codebase.\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let all = common::run_stdout(&["refs", "helper", "--repo", repo.path().to_str().unwrap()]);
    let code_only = common::run_stdout(&[
        "refs", "helper", "--repo", repo.path().to_str().unwrap(), "--code-only",
    ]);

    // code_only should have fewer or equal results (no .md files)
    let all_count: usize = all.lines().filter(|l| l.contains("helper")).count();
    let code_count: usize = code_only.lines().filter(|l| l.contains("helper")).count();
    assert!(code_count <= all_count, "code-only should not have more results than all");
    // Verify no .md files in code_only output
    for line in code_only.lines() {
        assert!(!line.ends_with(".md") && !line.contains(".md:"),
            "code-only should not contain .md files: {line}");
    }
}
```

**Step 2: Run test — expected PASS (already implemented)**

If it passes, this task is just verification. If it fails, investigate and fix.

**Step 3: Commit test**

```bash
git add tests/milestone_dogfood_refs_code_only.rs
git commit -m "TEST: verify --code-only flag works for refs command"
```

---

### Task 6: Set default `--max-results 30` for `diff-impact`

**Files:**

- Modify: `src/cli.rs:125-126` (add default_value)
- Test: `tests/milestone_dogfood_diff_impact_default_limit.rs`

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn diff_impact_defaults_to_30_max_results() {
    let repo = common::temp_repo();
    // Create a file with many symbols to generate many impact results
    let mut source = String::new();
    for i in 0..50 {
        source.push_str(&format!("pub fn func_{i}() {{}}\n"));
    }
    source.push_str("pub fn hub() {\n");
    for i in 0..50 {
        source.push_str(&format!("    func_{i}();\n"));
    }
    source.push_str("}\n");
    common::write_file(repo.path(), "src/lib.rs", &source);
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = common::run_stdout(&[
        "diff-impact",
        "--changed-file", "src/lib.rs",
        "--repo", repo.path().to_str().unwrap(),
    ]);

    // Count result lines (lines with file paths, not header lines)
    let result_count: usize = output.lines()
        .filter(|l| l.starts_with("src/"))
        .count();
    assert!(
        result_count <= 30,
        "default diff-impact should cap at 30 results, got {result_count}"
    );
}
```

**Step 2: Run test — expected FAIL**

Run: `cargo test --test milestone_dogfood_diff_impact_default_limit -- --nocapture`

**Step 3: Implement**

In `src/cli.rs:125-126`, change:

```rust
    #[arg(long = "max-results")]
    pub max_results: Option<u32>,
```

to:

```rust
    #[arg(long = "max-results", default_value_t = 30)]
    pub max_results: u32,
```

Update `src/main.rs` `run_diff_impact` to pass `Some(args.max_results)` instead of
`args.max_results`.

Add a `--no-limit` flag if users want unlimited:
`#[arg(long, default_value_t = false)] pub no_limit: bool` and pass `None` when `no_limit` is true.

**Step 4: Run tests — expected PASS**

**Step 5: Full suite + commit**

```bash
git add src/cli.rs src/main.rs tests/milestone_dogfood_diff_impact_default_limit.rs
git commit -m "GREEN: default diff-impact --max-results to 30"
```

---

## Phase 3: New Commands (Medium Effort)

### Task 7: `snippet <symbol>` — targeted source extraction

**Files:**

- Modify: `src/cli.rs` (add `Snippet` variant + `SnippetArgs`)
- Modify: `src/query/mod.rs` (add `snippet_for_symbol` function)
- Modify: `src/output.rs` (add `print_snippet` + `print_snippet_json`)
- Modify: `src/main.rs` (add `run_snippet` handler + dispatch)
- Test: `tests/milestone_dogfood_snippet.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

use serde_json::Value;

#[test]
fn snippet_returns_function_source() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n\npub fn farewell() -> &'static str {\n    \"bye\"\n}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "snippet", "greet", "--repo", repo.path().to_str().unwrap(),
    ]);

    assert!(output.contains("pub fn greet"), "should contain function signature:\n{output}");
    assert!(output.contains("Hello, {name}!"), "should contain function body:\n{output}");
    assert!(!output.contains("farewell"), "should NOT contain other functions:\n{output}");
}

#[test]
fn snippet_json_output() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "snippet", "add", "--repo", repo.path().to_str().unwrap(), "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    assert_eq!(payload["command"], "snippet");
    assert!(payload["results"][0]["snippet"].as_str().unwrap().contains("a + b"));
}

#[test]
fn snippet_with_context_lines() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "// This is a greeting function\npub fn greet() -> String {\n    \"hi\".to_string()\n}\n// End of greet\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "snippet", "greet", "--repo", repo.path().to_str().unwrap(), "--context", "1",
    ]);
    assert!(output.contains("greeting function"), "should contain 1 context line above:\n{output}");
}
```

**Step 2: Run test — expected FAIL (command doesn't exist)**

Run: `cargo test --test milestone_dogfood_snippet -- --nocapture`

**Step 3: Implement**

Add to `src/cli.rs`:

```rust
#[derive(Debug, Args)]
pub struct SnippetArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 0)]
    pub context: u32,
}
```

Add `Snippet(SnippetArgs)` to `Command` enum with
`#[command(about = "Extract source code for a symbol")]`.

Add to `src/query/mod.rs`:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SnippetMatch {
    pub symbol: String,
    pub kind: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub snippet: String,
    pub signature: Option<String>,
}

pub fn snippet_for_symbol(
    db_path: &Path,
    symbol: &str,
    context_lines: u32,
) -> anyhow::Result<Vec<SnippetMatch>> {
    let connection = Connection::open(db_path)?;
    // Query symbols_v2 for matching symbols
    let mut stmt = connection.prepare(
        "SELECT file_path, symbol, kind, start_line, end_line, signature
         FROM symbols_v2
         WHERE symbol = ?1
         ORDER BY file_path, start_line"
    )?;
    let rows = stmt.query_map(params![symbol], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, u32>(3)?,
            row.get::<_, u32>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        let (file_path, sym, kind, start_line, end_line, signature) = row?;
        let adj_start = start_line.saturating_sub(context_lines);
        let adj_end = end_line.saturating_add(context_lines);
        if let Some(snippet) = extract_symbol_snippet(db_path, &file_path, adj_start, adj_end) {
            results.push(SnippetMatch {
                symbol: sym, kind, file_path, start_line, end_line, snippet, signature,
            });
        }
    }
    Ok(results)
}
```

Add `print_snippet` and `print_snippet_json` to `src/output.rs`.

Add `run_snippet` to `src/main.rs` and wire into the match dispatch.

**Step 4: Run tests — expected PASS**

**Step 5: Full suite + commit**

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_snippet.rs
git commit -m "GREEN: add snippet command for targeted source extraction"
```

---

### Task 8: `outline <file>` — file structure without bodies

**Files:**

- Modify: `src/cli.rs` (add `Outline` variant + `OutlineArgs`)
- Modify: `src/query/mod.rs` (add `outline_file` function)
- Modify: `src/output.rs` (add `print_outline` + `print_outline_json`)
- Modify: `src/main.rs` (add `run_outline` + dispatch)
- Test: `tests/milestone_dogfood_outline.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

use serde_json::Value;

#[test]
fn outline_shows_signatures_without_bodies() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub struct Foo {\n    pub x: i32,\n}\n\npub fn bar(a: i32) -> i32 {\n    a * 2\n}\n\nfn baz() {\n    println!(\"hello\");\n}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "outline", "src/lib.rs", "--repo", repo.path().to_str().unwrap(),
    ]);

    assert!(output.contains("Foo"), "should list struct Foo:\n{output}");
    assert!(output.contains("bar"), "should list function bar:\n{output}");
    assert!(output.contains("baz"), "should list function baz:\n{output}");
    // Should NOT contain implementation details
    assert!(!output.contains("a * 2"), "should not contain function bodies:\n{output}");
    assert!(!output.contains("println"), "should not contain function bodies:\n{output}");
}

#[test]
fn outline_json_output() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "outline", "src/lib.rs", "--repo", repo.path().to_str().unwrap(), "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    assert_eq!(payload["command"], "outline");
    assert!(payload["results"].as_array().unwrap().len() > 0);
}
```

**Step 2: Run test — expected FAIL**

**Step 3: Implement**

Add to `src/cli.rs`:

```rust
#[derive(Debug, Args)]
pub struct OutlineArgs {
    pub file: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}
```

Add
`#[command(about = "Show file structure: signatures and definitions without bodies")] Outline(OutlineArgs)`
to `Command`.

Add to `src/query/mod.rs`:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct OutlineEntry {
    pub symbol: String,
    pub kind: String,
    pub line: u32,
    pub signature: Option<String>,
    pub visibility: String,  // "pub" or ""
}

pub fn outline_file(db_path: &Path, file_path: &str) -> anyhow::Result<Vec<OutlineEntry>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT symbol, kind, start_line, signature
         FROM symbols_v2
         WHERE file_path = ?1
         ORDER BY start_line"
    )?;
    let rows = stmt.query_map(params![file_path], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut entries = Vec::new();
    for row in rows {
        let (symbol, kind, line, signature) = row?;
        let visibility = signature.as_deref()
            .map(|s| if s.starts_with("pub") { "pub" } else { "" })
            .unwrap_or("")
            .to_string();
        entries.push(OutlineEntry { symbol, kind, line, signature, visibility });
    }
    Ok(entries)
}
```

Wire into `output.rs` and `main.rs` similarly to other commands.

**Step 4-6: Run tests, full suite, commit**

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_outline.rs
git commit -m "GREEN: add outline command for file structure view"
```

---

### Task 9: `summary` — whole-repo structural overview

**Files:**

- Modify: `src/cli.rs` (add `Summary` variant)
- Modify: `src/query/mod.rs` (add `repo_summary` function)
- Modify: `src/output.rs` (add `print_summary` + JSON variant)
- Modify: `src/main.rs` (add `run_summary` + dispatch)
- Test: `tests/milestone_dogfood_summary.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn summary_shows_repo_overview() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\npub struct Foo {}\n");
    common::write_file(repo.path(), "src/main.rs", "fn main() { hello(); }\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["summary", "--repo", repo.path().to_str().unwrap()]);

    assert!(output.contains("source_files:"), "missing file count:\n{output}");
    assert!(output.contains("definitions:"), "missing definition count:\n{output}");
    assert!(output.contains("edges:"), "missing edge count:\n{output}");
    assert!(output.contains("rust"), "missing language breakdown:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Implementation reuses `status_summary` from Task 4, adding module structure and entry point
detection. Query `symbols_v2` grouped by file_path prefix for module structure. Identify `main`
functions for entry points.

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_summary.rs
git commit -m "GREEN: add summary command for whole-repo structural overview"
```

---

## Phase 4: New Commands (Higher Effort)

### Task 10: `--since <commit>` for `diff-impact` and `verify-plan`

**Files:**

- Modify: `Cargo.toml` (add `git2` dependency)
- Modify: `src/cli.rs` (add `--since` and `--unstaged` to `DiffImpactArgs` and `VerifyPlanArgs`)
- Add: `src/git_utils.rs` (git diff file extraction)
- Modify: `src/main.rs` (use git_utils in `run_diff_impact` and `run_verify_plan`)
- Test: `tests/milestone_dogfood_since.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn diff_impact_since_head_detects_changed_files() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\n");

    // Initialize git, make initial commit
    std::process::Command::new("git").args(["init"]).current_dir(repo.path()).output().unwrap();
    std::process::Command::new("git").args(["add", "."]).current_dir(repo.path()).output().unwrap();
    std::process::Command::new("git").args(["commit", "-m", "initial"]).current_dir(repo.path()).output().unwrap();

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    // Make a change and commit
    common::write_file(repo.path(), "src/lib.rs", "pub fn original() {}\npub fn added() {}\n");
    std::process::Command::new("git").args(["add", "."]).current_dir(repo.path()).output().unwrap();
    std::process::Command::new("git").args(["commit", "-m", "add function"]).current_dir(repo.path()).output().unwrap();

    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);

    let output = common::run_stdout(&[
        "diff-impact", "--since", "HEAD~1", "--repo", repo.path().to_str().unwrap(),
    ]);
    assert!(output.contains("src/lib.rs"), "should detect changed file:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Create `src/git_utils.rs`:

```rust
use std::path::Path;
use std::process::Command;

pub fn changed_files_since(repo: &Path, since: &str) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", since, "HEAD"])
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let files = String::from_utf8(output.stdout)?
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    Ok(files)
}

pub fn unstaged_files(repo: &Path) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    let files = String::from_utf8(output.stdout)?
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    Ok(files)
}
```

Make `--changed-file` no longer `required = true` — instead require at least one of
`--changed-file`, `--since`, or `--unstaged`.

```bash
git add Cargo.toml src/git_utils.rs src/cli.rs src/main.rs tests/milestone_dogfood_since.rs
git commit -m "GREEN: add --since and --unstaged flags to diff-impact and verify-plan"
```

---

### Task 11: `callers <symbol>` and `callees <symbol>`

**Files:**

- Modify: `src/cli.rs` (add `Callers` + `Callees` variants)
- Modify: `src/query/mod.rs` (add `callers_of` and `callees_of` functions)
- Modify: `src/output.rs` (add rendering)
- Modify: `src/main.rs` (add handlers + dispatch)
- Test: `tests/milestone_dogfood_callers_callees.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn callers_shows_who_calls_a_function() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn leaf() {}\npub fn mid() { leaf(); }\npub fn top() { mid(); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["callers", "leaf", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("mid"), "mid should call leaf:\n{output}");
}

#[test]
fn callees_shows_what_a_function_calls() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn a() {}\npub fn b() {}\npub fn hub() { a(); b(); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["callees", "hub", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("a"), "hub should call a:\n{output}");
    assert!(output.contains("b"), "hub should call b:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Query `symbol_edges_v2` where `edge_kind = 'calls'`:

- `callers_of`: join where `to_symbol_id` matches the target symbol
- `callees_of`: join where `from_symbol_id` matches the target symbol

```rust
pub fn callers_of(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT s_from.file_path, s_from.symbol, s_from.kind, s_from.start_line, s_from.start_column, e.confidence
         FROM symbol_edges_v2 e
         JOIN symbols_v2 s_from ON e.from_symbol_id = s_from.symbol_id
         JOIN symbols_v2 s_to ON e.to_symbol_id = s_to.symbol_id
         WHERE s_to.symbol = ?1 AND e.edge_kind = 'calls'
         ORDER BY s_from.file_path, s_from.start_line"
    )?;
    // ... map to QueryMatch
}
```

`callees_of` is the inverse (swap `from` and `to`).

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_callers_callees.rs
git commit -m "GREEN: add callers and callees commands for directed graph navigation"
```

---

### Task 12: `deps <file>` — file-level dependency graph

**Files:**

- Modify: `src/cli.rs` (add `Deps` variant + `DepsArgs`)
- Modify: `src/query/mod.rs` (add `file_deps` function)
- Modify: `src/output.rs` (add `print_deps`)
- Modify: `src/main.rs` (add `run_deps` + dispatch)
- Test: `tests/milestone_dogfood_deps.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn deps_shows_file_dependencies() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/helper.rs", "pub fn help() {}\n");
    common::write_file(repo.path(), "src/lib.rs", "mod helper;\npub fn main_fn() { helper::help(); }\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["deps", "src/lib.rs", "--repo", repo.path().to_str().unwrap()]);
    assert!(output.contains("depends_on:") || output.contains("src/helper.rs"),
        "should show dependency on helper.rs:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Query: aggregate `symbol_edges_v2` by file_path pairs. For a given file, find all edges where `from`
is in that file pointing to symbols in other files (depends_on), and edges from other files pointing
into this file (depended_on_by).

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_deps.rs
git commit -m "GREEN: add deps command for file-level dependency graph"
```

---

### Task 13: `hotspots` — most-connected symbols

**Files:**

- Modify: `src/cli.rs` (add `Hotspots` variant + `HotspotsArgs`)
- Modify: `src/query/mod.rs` (add `hotspots` function)
- Modify: `src/output.rs` (add `print_hotspots`)
- Modify: `src/main.rs` (add `run_hotspots` + dispatch)
- Test: `tests/milestone_dogfood_hotspots.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn hotspots_returns_most_connected_symbols() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn core_fn() {}\npub fn a() { core_fn(); }\npub fn b() { core_fn(); }\npub fn c() { core_fn(); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "hotspots", "--repo", repo.path().to_str().unwrap(), "--limit", "5",
    ]);
    // core_fn should be #1 — it has the most inbound edges
    let lines: Vec<&str> = output.lines().filter(|l| l.contains("core_fn")).collect();
    assert!(!lines.is_empty(), "core_fn should appear in hotspots:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Query:

```sql
SELECT s.symbol, s.file_path, s.kind,
       COUNT(DISTINCT e_in.from_symbol_id) as fan_in,
       COUNT(DISTINCT e_out.to_symbol_id) as fan_out
FROM symbols_v2 s
LEFT JOIN symbol_edges_v2 e_in ON e_in.to_symbol_id = s.symbol_id
LEFT JOIN symbol_edges_v2 e_out ON e_out.from_symbol_id = s.symbol_id
GROUP BY s.symbol_id
ORDER BY (COUNT(DISTINCT e_in.from_symbol_id) + COUNT(DISTINCT e_out.to_symbol_id)) DESC
LIMIT ?1
```

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_hotspots.rs
git commit -m "GREEN: add hotspots command for most-connected symbols"
```

---

## Phase 5: Scoring & Quality Improvements

### Task 14: Differentiate `context` relevance scoring

**Files:**

- Modify: `src/query/mod.rs:2218-2229` (replace `context_direct_score`)
- Test: `tests/milestone_dogfood_context_scoring.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

use serde_json::Value;

#[test]
fn context_scoring_differentiates_direct_vs_tangential() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn index_file(path: &str) -> bool { true }\n\
         pub fn validate_schema() { /* mentions index in comment */ }\n\
         pub fn index_repository(path: &str) { index_file(path); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let json_out = common::run_stdout(&[
        "context", "--task", "understand how indexing works",
        "--repo", repo.path().to_str().unwrap(), "--json",
    ]);
    let payload: Value = serde_json::from_str(&json_out).expect("valid json");
    let results = payload["results"].as_array().unwrap();

    if results.len() >= 2 {
        let scores: Vec<f64> = results.iter()
            .map(|r| r["score"].as_f64().unwrap())
            .collect();
        let unique_scores: std::collections::HashSet<u64> = scores.iter()
            .map(|s| (s * 1000.0) as u64)
            .collect();
        assert!(
            unique_scores.len() > 1,
            "scores should be differentiated, got: {scores:?}"
        );
    }
}
```

**Step 2-6: Red-green-refactor-commit**

Replace `context_direct_score` with a tf-idf-inspired approach:

```rust
fn context_direct_score(
    overlap_count: usize,
    exact_symbol_match: bool,
    symbol_token_count: usize,
    task_token_count: usize,
    total_symbols_with_overlap: usize,
) -> f64 {
    // IDF-like: rarer matches score higher
    let idf = if total_symbols_with_overlap > 0 {
        (1.0 + (total_symbols_with_overlap as f64).ln()).recip()
    } else {
        1.0
    };
    let tf = overlap_count as f64 / task_token_count.max(1) as f64;
    let base = 0.50 + (tf * idf * 0.40);

    // Name match bonus: symbol name contains task keywords
    let name_bonus = if exact_symbol_match { 0.15 } else { 0.0 };

    // Specificity: longer symbol names are more specific
    let specificity = (std::cmp::min(symbol_token_count, 5) as f64 * 0.02).min(0.10);

    (base + name_bonus + specificity).min(0.98)
}
```

This requires threading `total_symbols_with_overlap` and `task_token_count` through the caller.
Update `context_matches` to compute these and pass them.

```bash
git add src/query/mod.rs tests/milestone_dogfood_context_scoring.rs
git commit -m "GREEN: differentiate context scoring with tf-idf-inspired approach"
```

---

### Task 15: Improve `tests-for` recall with CLI-command mapping

**Files:**

- Modify: `src/query/mod.rs` (enhance `test_targets_for_symbol`)
- Test: `tests/milestone_dogfood_tests_for_recall.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn tests_for_finds_cli_integration_tests() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/main.rs",
        "fn main() {}\nfn run_index() { index_repository(); }\nfn index_repository() {}\n"
    );
    common::write_file(repo.path(), "tests/test_index.rs",
        "use assert_cmd::Command;\n#[test]\nfn test_indexing() {\n    Command::cargo_bin(\"myapp\").unwrap().arg(\"index\").assert().success();\n}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "tests-for", "index_repository", "--repo", repo.path().to_str().unwrap(),
    ]);
    // The test file invokes "index" CLI command, which maps to run_index -> index_repository
    // This is a stretch goal — if the basic heuristic picks it up via text matching, that's fine
    assert!(
        output.contains("test_index") || output.contains("tests/"),
        "should find test that exercises index_repository via CLI:\n{output}"
    );
}
```

**Step 2-6: Red-green-refactor-commit**

Add a secondary heuristic: if a test file contains `.arg("index")` or similar CLI subcommand
strings, map them to their handler functions (`run_index` → `index_repository`). Build a simple
mapping table in the query.

```bash
git add src/query/mod.rs tests/milestone_dogfood_tests_for_recall.rs
git commit -m "GREEN: improve tests-for recall with CLI-command mapping heuristic"
```

---

### Task 16: Group `refs` output by category

**Files:**

- Modify: `src/output.rs` (new `print_refs_grouped` function)
- Modify: `src/main.rs` (use grouped output for refs terminal)
- Test: `tests/milestone_dogfood_refs_grouped.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn refs_terminal_groups_by_category() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn helper() {}\npub fn caller() { helper(); }\n"
    );
    common::write_file(repo.path(), "tests/test_it.rs",
        "fn test_helper() { /* helper */ }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&["refs", "helper", "--repo", repo.path().to_str().unwrap()]);

    // Output should have section headers
    let has_sections = output.contains("Source") || output.contains("Test") || output.contains("Definitions");
    assert!(has_sections, "refs output should group results by category:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

In `print_refs_grouped`, partition `QueryMatch` results into:

- Definitions (where `why_matched` contains `ast_definition`)
- Source references (file_path starts with `src/`)
- Test references (file_path starts with `tests/` or contains `_test`)
- Documentation (file_path ends with `.md`)

Print with section headers.

```bash
git add src/output.rs src/main.rs tests/milestone_dogfood_refs_grouped.rs
git commit -m "GREEN: group refs terminal output by category"
```

---

## Phase 6: Stretch Goals

### Task 17: `path <from> <to>` — call path discovery

**Files:**

- Modify: `src/cli.rs` (add `Path` variant + `PathArgs`)
- Modify: `src/query/mod.rs` (add `find_call_path` using BFS on `symbol_edges_v2`)
- Modify: `src/output.rs` (add `print_path`)
- Modify: `src/main.rs` (add dispatch)
- Test: `tests/milestone_dogfood_path.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn path_finds_call_chain() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn a() { b(); }\npub fn b() { c(); }\npub fn c() {}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "call-path", "a", "c", "--repo", repo.path().to_str().unwrap(),
    ]);
    assert!(output.contains("a"), "path should include start:\n{output}");
    assert!(output.contains("b"), "path should include intermediate:\n{output}");
    assert!(output.contains("c"), "path should include end:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Use BFS over `symbol_edges_v2` where `edge_kind = 'calls'`. Start from `from` symbol, expand
outbound edges until `to` is found or max depth (default 10) exceeded.

Note: Use `CallPath` as the command name in the CLI to avoid conflict with Rust's `Path` type.

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_path.rs
git commit -m "GREEN: add call-path command for call chain discovery"
```

---

### Task 18: `related <symbol>` — structural neighborhood

**Files:**

- Modify: `src/cli.rs` (add `Related` variant + `RelatedArgs`)
- Modify: `src/query/mod.rs` (add `related_symbols` function)
- Modify: `src/output.rs` (add `print_related`)
- Modify: `src/main.rs` (add dispatch)
- Test: `tests/milestone_dogfood_related.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn related_shows_siblings_and_shared_callers() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn sibling_a() {}\npub fn sibling_b() {}\npub fn user() { sibling_a(); sibling_b(); }\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "related", "sibling_a", "--repo", repo.path().to_str().unwrap(),
    ]);
    assert!(output.contains("sibling_b"), "sibling_b should be related to sibling_a:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Three relationship types:

1. **Siblings**: same file, same container (or top-level)
2. **Shares callers**: symbols called by the same function
3. **Shares callees**: symbols that call the same functions

Query siblings from `symbols_v2` where `file_path` matches. Query shared callers/callees via two-hop
joins on `symbol_edges_v2`.

```bash
git add src/cli.rs src/query/mod.rs src/output.rs src/main.rs tests/milestone_dogfood_related.rs
git commit -m "GREEN: add related command for structural neighborhood discovery"
```

---

### Task 19: `--compact` output mode

**Files:**

- Modify: `src/cli.rs` (add `--compact` flag to `FindArgs`, `RefsArgs`, `ExplainArgs`)
- Modify: `src/output.rs` (add compact printing variants)
- Modify: `src/main.rs` (pass compact flag)
- Test: `tests/milestone_dogfood_compact.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn find_compact_output_is_minimal() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs", "pub fn hello() {}\n");
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    let output = common::run_stdout(&[
        "find", "hello", "--repo", repo.path().to_str().unwrap(), "--compact",
    ]);
    // Compact: just file:line symbol, no headers, no metadata
    assert!(!output.contains("command:"), "compact should not have command header:\n{output}");
    assert!(!output.contains("results:"), "compact should not have results header:\n{output}");
    assert!(output.contains("src/lib.rs:"), "should still show file:line:\n{output}");
}
```

**Step 2-6: Red-green-refactor-commit**

Compact format: one line per result, `file_path:line symbol` only. No headers, no metadata.

```bash
git add src/cli.rs src/output.rs src/main.rs tests/milestone_dogfood_compact.rs
git commit -m "GREEN: add --compact output mode for context-constrained agents"
```

---

### Task 20: Fuzzy matching / "did you mean?" for `find`

**Files:**

- Modify: `src/query/mod.rs` (add fuzzy fallback to `find_matches_scoped`)
- Modify: `src/output.rs` (add "did you mean?" suggestion rendering)
- Test: `tests/milestone_dogfood_fuzzy.rs` (new)

**Step 1: Write the failing test**

```rust
mod common;

#[test]
fn find_suggests_similar_when_no_exact_match() {
    let repo = common::temp_repo();
    common::write_file(repo.path(), "src/lib.rs",
        "pub fn index_repository() {}\npub fn index_file() {}\n"
    );
    common::run_stdout(&["index", "--repo", repo.path().to_str().unwrap()]);
    // Misspelled: "index_repo" instead of "index_repository"
    let output = common::run_stdout(&[
        "find", "index_repo", "--repo", repo.path().to_str().unwrap(),
    ]);
    assert!(
        output.contains("did you mean") || output.contains("index_repository"),
        "should suggest similar symbol:\n{output}"
    );
}
```

**Step 2-6: Red-green-refactor-commit**

When `find_matches_scoped` returns 0 results, query `symbols_v2` for symbols containing the search
term as a substring. Use Levenshtein distance (implement a simple version or use prefix/substring
matching as a simpler alternative). Show top 5 suggestions.

Simple approach — no external dependency:

```rust
fn suggest_similar_symbols(connection: &Connection, symbol: &str) -> anyhow::Result<Vec<String>> {
    let pattern = format!("%{symbol}%");
    let mut stmt = connection.prepare(
        "SELECT DISTINCT symbol FROM symbols_v2
         WHERE symbol LIKE ?1
         ORDER BY LENGTH(symbol) ASC
         LIMIT 5"
    )?;
    let rows = stmt.query_map(params![pattern], |row| row.get::<_, String>(0))?;
    rows.filter_map(|r| r.ok()).collect::<Vec<_>>().pipe(Ok)
}
```

```bash
git add src/query/mod.rs src/output.rs tests/milestone_dogfood_fuzzy.rs
git commit -m "GREEN: add fuzzy matching and 'did you mean?' suggestions to find"
```

---

## Execution Order Summary

| Phase                  | Tasks | Effort       | Dependencies                                 |
| ---------------------- | ----- | ------------ | -------------------------------------------- |
| 1: Bug Fix             | 1     | Small        | None                                         |
| 2: Quick Wins          | 2-6   | Small each   | None                                         |
| 3: New Commands (Med)  | 7-9   | Medium each  | None                                         |
| 4: New Commands (High) | 10-13 | Medium-Large | Task 10 needs `git2` dep                     |
| 5: Scoring             | 14-16 | Medium each  | None                                         |
| 6: Stretch             | 17-20 | Medium each  | Tasks 11 (callers/callees) helpful for 17-18 |

Tasks within each phase are independent and can be parallelized. Phases should be done in order for
incremental value delivery.

## Testing Strategy

Every task follows strict Red-Green-Refactor:

1. Write failing integration test in `tests/milestone_dogfood_*.rs`
2. Run it to confirm failure
3. Implement minimal code
4. Run it to confirm pass
5. Run `cargo test` full suite
6. Run `cargo clippy --all-targets --all-features -- -D warnings`
7. Commit with appropriate prefix

## Post-Implementation Verification

After all tasks:

```bash
just check                        # fmt + clippy + test
cargo run -- index --repo .       # dogfood: re-index
cargo run -- snippet find_matches_scoped --repo .  # dogfood: new command
cargo run -- outline src/query/mod.rs --repo .     # dogfood: new command
cargo run -- summary --repo .                      # dogfood: new command
cargo run -- hotspots --repo . --limit 10          # dogfood: new command
cargo run -- callers ensure_store --repo .          # dogfood: new command
cargo run -- callees index_repository --repo .      # dogfood: new command
```
