use std::path::Path;

use crate::query::{
    ContextMatch, DiffImpactMatch, EdgeMatch, ExplainMatch, FileDeps, HotspotEntry, ImpactMatch,
    OutlineEntry, QueryMatch, RelatedSymbol, SnippetMatch, StatusSummary, TestTarget,
    VerificationStep,
    diagnostics::{
        AnatomyReport, CircularReport, CouplingEntry, DeadSymbol, HealthReport, Suggestion,
        TestGapReport,
    },
    orientation::{OrientReport, TreeNode, TreeNodeKind, TreeReport},
    planning::BoundaryReport,
};
use serde::Serialize;

pub const JSON_SCHEMA_VERSION: u32 = 1;
pub const JSON_SCHEMA_VERSION_V2: u32 = 2;
pub const JSON_SCHEMA_VERSION_V3: u32 = 3;

#[derive(Debug, Serialize)]
struct JsonQueryOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [QueryMatch],
}

#[derive(Debug, Serialize)]
struct JsonImpactOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [ImpactMatch],
}

#[derive(Debug, Serialize)]
struct JsonContextOutput<'a> {
    schema_version: u32,
    command: &'a str,
    task: &'a str,
    budget: u32,
    results: &'a [ContextMatch],
}

#[derive(Debug, Serialize)]
struct JsonTestsForOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [TestTarget],
}

#[derive(Debug, Serialize)]
struct JsonVerifyPlanOutput<'a> {
    schema_version: u32,
    command: &'a str,
    changed_files: &'a [String],
    results: &'a [VerificationStep],
}

#[derive(Debug, Serialize)]
struct JsonDiffImpactOutput<'a> {
    schema_version: u32,
    command: &'a str,
    changed_files: &'a [String],
    max_distance: u32,
    include_tests: bool,
    results: &'a [DiffImpactMatch],
}

#[derive(Debug, Serialize)]
struct JsonExplainOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    include_snippets: bool,
    results: &'a [ExplainMatch],
}

/// Prints index metadata (path, schema version, and file counts) to stdout.
///
/// This writes four lines showing:
/// `index_path`, `schema_version`, `indexed_files`, and `non_source_files`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// print_index(Path::new("index.db"), 1, 42, 3);
/// ```
pub fn print_index(
    index_path: &Path,
    schema_version: i64,
    indexed_files: usize,
    non_source_files: usize,
) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
    println!("indexed_files: {indexed_files}");
    println!("non_source_files: {non_source_files}");
}

pub fn print_status(index_path: &Path, schema_version: i64, summary: &StatusSummary) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
    println!("source_files: {}", summary.source_files);
    println!("definitions: {}", summary.definitions);
    println!("references: {}", summary.references);
    println!("text_occurrences: {}", summary.text_occurrences);
    println!("edges: {}", summary.edges);
    if !summary.languages.is_empty() {
        println!("languages:");
        for (lang, count) in &summary.languages {
            println!("  {lang}: {count}");
        }
    }
}

pub fn print_query(command: &str, symbol: &str, matches: &[QueryMatch]) {
    println!("command: {command}");
    println!("query: {symbol}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}:{} {} [{} {}]",
            result.file_path,
            result.line,
            result.column,
            result.symbol,
            result.why_matched,
            result.confidence
        );
    }
}

pub fn print_did_you_mean(suggestions: &[String]) {
    println!("\ndid you mean:");
    for s in suggestions {
        println!("  {s}");
    }
}

pub fn print_query_compact(matches: &[QueryMatch]) {
    for result in matches {
        println!("{}:{} {}", result.file_path, result.line, result.symbol);
    }
}

/// Emit the query results as pretty-printed JSON to stdout.
///
/// The JSON payload uses the current JSON_SCHEMA_VERSION and includes the
/// original command, the queried symbol, and the provided results slice.
///
/// # Errors
///
/// Returns an error if serialization of the JSON payload fails.
///
/// # Examples
///
/// ```no_run
/// // `matches` is a slice of `QueryMatch`; an empty slice can be passed when
/// // there are no results.
/// let _ = crate::output::print_query_json(
///     "query",
///     "my_symbol",
///     &[] as &[crate::types::QueryMatch],
/// );
/// ```
pub fn print_query_json(command: &str, symbol: &str, matches: &[QueryMatch]) -> anyhow::Result<()> {
    let payload = JsonQueryOutput {
        schema_version: JSON_SCHEMA_VERSION,
        command,
        query: symbol,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

/// Prints a human-readable summary of impact results for a symbol.
///
/// The output includes the command ("impact"), the queried symbol, the number of results,
/// and one line per match containing file path, line, column, symbol, kind, relationship,
/// confidence, and score.
///
/// # Examples
///
/// ```ignore
/// // Print no results
/// print_impact("my::symbol", &[]);
/// ```
pub fn print_impact(symbol: &str, matches: &[ImpactMatch]) {
    println!("command: impact");
    println!("query: {symbol}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}:{} {} ({}) [{} {} {:.2}]",
            result.file_path,
            result.line,
            result.column,
            result.symbol,
            result.kind,
            result.relationship,
            result.confidence,
            result.score
        );
    }
}

/// Serializes impact matches using the v2 JSON schema.
/// Prints the pretty-formatted JSON payload to stdout.
///
/// # Returns
///
/// `Ok(())` if serialization and printing succeed, propagated error otherwise.
///
/// # Examples
///
/// ```no_run
/// let matches: &[ImpactMatch] = &[];
/// print_impact_json("my::symbol", matches).unwrap();
/// ```
pub fn print_impact_json(symbol: &str, matches: &[ImpactMatch]) -> anyhow::Result<()> {
    let payload = JsonImpactOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "impact",
        query: symbol,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

/// Prints a human-readable summary of a context query's results.
///
/// The output includes the command name, task, budget, total result count, and one line per
/// match with file path, start/end lines, symbol, kind, reason for inclusion, confidence,
/// and score.
///
/// # Examples
///
/// ```
/// // Use an empty slice when no matches are available.
/// let matches: &[ContextMatch] = &[];
/// print_context("build", 5, matches);
/// ```
pub fn print_context(task: &str, budget: u32, matches: &[ContextMatch]) {
    println!("command: context");
    println!("task: {task}");
    println!("budget: {budget}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}-{} {} ({}) why: {} [{} {:.2}]",
            result.file_path,
            result.start_line,
            result.end_line,
            result.symbol,
            result.kind,
            result.why_included,
            result.confidence,
            result.score
        );
    }
}

/// Serialize the provided context matches into the v2 JSON schema and print the result to stdout.
///
/// The emitted JSON contains schema version, command (`"context"`), task,
/// budget, and the `results` array.
///
/// # Examples
///
/// ```
/// // prints a JSON object for an empty results list
/// let res = print_context_json("build-docs", 5, &[]);
/// assert!(res.is_ok());
/// ```
pub fn print_context_json(task: &str, budget: u32, matches: &[ContextMatch]) -> anyhow::Result<()> {
    let payload = JsonContextOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "context",
        task,
        budget,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

/// Prints a human-readable summary of test targets for a given symbol to stdout.
///
/// The output includes the command ("tests-for"), the queried symbol, the number of results,
/// and one line per target showing target name, kind, reason for inclusion, confidence, and score.
///
/// # Examples
///
/// ```
/// // An empty slice can be passed when there are no targets.
/// let targets: &[crate::TestTarget] = &[];
/// print_tests_for("my::symbol", targets);
/// ```
pub fn print_tests_for(symbol: &str, targets: &[TestTarget]) {
    println!("command: tests-for");
    println!("query: {symbol}");
    println!("results: {}", targets.len());
    for target in targets {
        println!(
            "{} ({}) why: {} [{} {:.2}]",
            target.target, target.target_kind, target.why_included, target.confidence, target.score
        );
    }
}

/// Serialize the test targets for `symbol` into pretty-printed JSON and write it to stdout.
///
/// # Parameters
///
/// - `symbol`: The query symbol associated with the test targets.
/// - `targets`: Slice of `TestTarget` items to include in the JSON `results` field.
///
/// # Returns
///
/// `Ok(())` on success, `Err` if JSON serialization fails.
///
/// # Examples
///
/// ```
/// // Print an empty results array for symbol "my_crate"
/// let _ = print_tests_for_json("my_crate", &[]).unwrap();
/// ```
pub fn print_tests_for_json(symbol: &str, targets: &[TestTarget]) -> anyhow::Result<()> {
    let payload = JsonTestsForOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "tests-for",
        query: symbol,
        results: targets,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

/// Prints a human-readable verification plan summary and its verification steps.
///
/// The output includes the command name ("verify-plan"), the number of changed files,
/// the number of verification steps, and one line per step with its name, scope,
/// reason for inclusion, confidence, and score.
///
/// # Examples
///
/// ```
/// use crate::output::VerificationStep;
///
/// let changed_files = vec![String::from("src/lib.rs")];
/// let steps = vec![VerificationStep {
///     step: String::from("build"),
///     scope: String::from("repo"),
///     why_included: String::from("changed build files"),
///     confidence: 0.75,
///     score: 1.20,
/// }];
///
/// // Prints a readable verification plan to stdout.
/// crate::output::print_verify_plan(&changed_files, &steps);
/// ```
pub fn print_verify_plan(changed_files: &[String], steps: &[VerificationStep]) {
    println!("command: verify-plan");
    println!("changed_files: {}", changed_files.len());
    println!("results: {}", steps.len());
    for step in steps {
        println!(
            "{} ({}) why: {} [{} {:.2}]",
            step.step, step.scope, step.why_included, step.confidence, step.score
        );
    }
}

/// Serialize a verification plan and list of changed files to pretty-printed JSON and print it.
///
/// The emitted JSON uses the v2 schema and the `"verify-plan"` command label.
///
/// # Returns
///
/// `Ok(())` if serialization and printing succeed, `Err` if JSON serialization fails.
///
/// # Examples
///
/// ```
/// let changed_files: Vec<String> = vec!["src/lib.rs".into()];
/// let steps: Vec<VerificationStep> = Vec::new();
/// // Will print a JSON payload describing the verify-plan to stdout.
/// let _ = print_verify_plan_json(&changed_files, &steps);
/// ```
pub fn print_verify_plan_json(
    changed_files: &[String],
    steps: &[VerificationStep],
) -> anyhow::Result<()> {
    let payload = JsonVerifyPlanOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "verify-plan",
        changed_files,
        results: steps,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_diff_impact(
    changed_files: &[String],
    max_distance: u32,
    include_tests: bool,
    results: &[DiffImpactMatch],
) {
    println!("command: diff-impact");
    println!("changed_files: {}", changed_files.len());
    for changed_file in changed_files {
        println!("changed_file: {changed_file}");
    }
    println!("max_distance: {max_distance}");
    println!("include_tests: {include_tests}");
    println!("results: {}", results.len());
    for result in results {
        match result {
            DiffImpactMatch::ImpactedSymbol {
                symbol,
                kind,
                language,
                file_path,
                line,
                column,
                distance,
                relationship,
                confidence,
                provenance,
                score,
                ..
            } => {
                println!(
                    concat!(
                        "impacted_symbol {}:{}:{} {} ({}, {}) ",
                        "relationship={} distance={} confidence={} provenance={} score={:.2}"
                    ),
                    file_path,
                    line,
                    column,
                    symbol,
                    kind,
                    language,
                    relationship,
                    distance,
                    confidence,
                    provenance,
                    score
                );
            }
            DiffImpactMatch::TestTarget {
                target,
                target_kind,
                language,
                confidence,
                provenance,
                score,
                ..
            } => {
                println!(
                    concat!(
                        "test_target {} ({}, {}) ",
                        "confidence={} provenance={} score={:.2}"
                    ),
                    target, target_kind, language, confidence, provenance, score
                );
            }
        }
    }
}

pub fn print_diff_impact_json(
    changed_files: &[String],
    max_distance: u32,
    include_tests: bool,
    results: &[DiffImpactMatch],
) -> anyhow::Result<()> {
    let payload = JsonDiffImpactOutput {
        schema_version: JSON_SCHEMA_VERSION_V3,
        command: "diff-impact",
        changed_files,
        max_distance,
        include_tests,
        results,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_explain(symbol: &str, matches: &[ExplainMatch]) {
    println!("command: explain");
    println!("query: {symbol}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}:{} {} ({}) [{} {} {:.2}]",
            result.file_path,
            result.start_line,
            result.start_column,
            result.symbol,
            result.kind,
            result.provenance,
            result.confidence,
            result.score
        );
        if let Some(signature) = &result.signature {
            println!("signature: {signature}");
        }
        println!(
            "inbound: called_by={} imported_by={} implemented_by={} contained_by={}",
            result.inbound.called_by,
            result.inbound.imported_by,
            result.inbound.implemented_by,
            result.inbound.contained_by
        );
        println!(
            "outbound: calls={} imports={} implements={} contains={}",
            result.outbound.calls,
            result.outbound.imports,
            result.outbound.implements,
            result.outbound.contains
        );
        if let Some(snippet) = &result.snippet {
            println!("snippet:");
            for line in snippet.lines() {
                println!("  {line}");
            }
        }
    }
}

pub fn print_explain_compact(matches: &[ExplainMatch]) {
    for result in matches {
        println!(
            "{}:{} {}",
            result.file_path, result.start_line, result.symbol
        );
    }
}

pub fn print_explain_json(
    symbol: &str,
    include_snippets: bool,
    matches: &[ExplainMatch],
) -> anyhow::Result<()> {
    let payload = JsonExplainOutput {
        schema_version: JSON_SCHEMA_VERSION_V3,
        command: "explain",
        query: symbol,
        include_snippets,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct JsonSnippetOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [SnippetMatch],
}

pub fn print_snippet(symbol: &str, matches: &[SnippetMatch]) {
    println!("command: snippet");
    println!("query: {symbol}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}-{} {} ({})",
            result.file_path, result.start_line, result.end_line, result.symbol, result.kind,
        );
        for line in result.snippet.lines() {
            println!("  {line}");
        }
    }
}

pub fn print_snippet_json(symbol: &str, matches: &[SnippetMatch]) -> anyhow::Result<()> {
    let payload = JsonSnippetOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "snippet",
        query: symbol,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct JsonOutlineOutput<'a> {
    schema_version: u32,
    command: &'a str,
    file: &'a str,
    results: &'a [OutlineEntry],
}

pub fn print_outline(file: &str, entries: &[OutlineEntry]) {
    println!("command: outline");
    println!("file: {file}");
    println!("results: {}", entries.len());
    for entry in entries {
        let vis = if entry.visibility.is_empty() {
            String::new()
        } else {
            format!("{} ", entry.visibility)
        };
        let sig = entry.signature.as_deref().unwrap_or(&entry.symbol);
        println!("  L{} {}{} ({})", entry.line, vis, sig, entry.kind);
    }
}

pub fn print_outline_json(file: &str, entries: &[OutlineEntry]) -> anyhow::Result<()> {
    let payload = JsonOutlineOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "outline",
        file,
        results: entries,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_summary(summary: &StatusSummary, entry_points: &[String]) {
    println!("command: summary");
    println!("source_files: {}", summary.source_files);
    println!("definitions: {}", summary.definitions);
    println!("references: {}", summary.references);
    println!("text_occurrences: {}", summary.text_occurrences);
    println!("edges: {}", summary.edges);
    if !summary.languages.is_empty() {
        println!("languages:");
        for (lang, count) in &summary.languages {
            println!("  {lang}: {count}");
        }
    }
    if !entry_points.is_empty() {
        println!("entry_points:");
        for ep in entry_points {
            println!("  {ep}");
        }
    }
}

pub fn print_edges(command: &str, symbol: &str, matches: &[EdgeMatch]) {
    println!("command: {command}");
    println!("query: {symbol}");
    println!("results: {}", matches.len());
    for result in matches {
        println!(
            "{}:{}:{} {} ({}) [{:.2}]",
            result.file_path,
            result.line,
            result.column,
            result.symbol,
            result.kind,
            result.confidence,
        );
    }
}

#[derive(Debug, Serialize)]
struct JsonEdgeOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [EdgeMatch],
}

pub fn print_edges_json(command: &str, symbol: &str, matches: &[EdgeMatch]) -> anyhow::Result<()> {
    let payload = JsonEdgeOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command,
        query: symbol,
        results: matches,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_deps(file: &str, deps: &FileDeps) {
    println!("command: deps");
    println!("file: {file}");
    if !deps.depends_on.is_empty() {
        println!("depends_on:");
        for dep in &deps.depends_on {
            println!("  {} ({})", dep.file_path, dep.edge_count);
        }
    }
    if !deps.depended_on_by.is_empty() {
        println!("depended_on_by:");
        for dep in &deps.depended_on_by {
            println!("  {} ({})", dep.file_path, dep.edge_count);
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonDepsOutput<'a> {
    schema_version: u32,
    command: &'a str,
    file: &'a str,
    depends_on: &'a [crate::query::FileDep],
    depended_on_by: &'a [crate::query::FileDep],
}

pub fn print_deps_json(file: &str, deps: &FileDeps) -> anyhow::Result<()> {
    let payload = JsonDepsOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "deps",
        file,
        depends_on: &deps.depends_on,
        depended_on_by: &deps.depended_on_by,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_hotspots(entries: &[HotspotEntry]) {
    if entries.is_empty() {
        println!("No hotspots found.");
        return;
    }
    println!("hotspots:");
    for (i, e) in entries.iter().enumerate() {
        println!(
            "  #{}: {} ({}) in {} — fan_in: {}, fan_out: {}, total: {}",
            i + 1,
            e.symbol,
            e.kind,
            e.file_path,
            e.fan_in,
            e.fan_out,
            e.total
        );
    }
}

#[derive(Serialize)]
struct JsonHotspotOutput<'a> {
    schema_version: u32,
    command: &'a str,
    results: &'a [HotspotEntry],
}

pub fn print_hotspots_json(entries: &[HotspotEntry]) -> anyhow::Result<()> {
    let payload = JsonHotspotOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "hotspots",
        results: entries,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_refs_grouped(symbol: &str, matches: &[QueryMatch]) {
    println!("command: refs");
    println!("query: {symbol}");
    println!("results: {}", matches.len());

    let mut definitions = Vec::new();
    let mut source = Vec::new();
    let mut tests = Vec::new();
    let mut docs = Vec::new();
    let mut other = Vec::new();

    for m in matches {
        if m.why_matched.contains("ast_definition") {
            definitions.push(m);
        } else if m.file_path.ends_with(".md") {
            docs.push(m);
        } else if m.file_path.starts_with("tests/")
            || m.file_path.contains("_test.")
            || m.file_path.contains(".test.")
        {
            tests.push(m);
        } else if m.file_path.starts_with("src/") {
            source.push(m);
        } else {
            other.push(m);
        }
    }

    let sections: &[(&str, &[&QueryMatch])] = &[
        ("Definitions", &definitions),
        ("Source", &source),
        ("Test", &tests),
        ("Documentation", &docs),
        ("Other", &other),
    ];

    for (label, items) in sections {
        if items.is_empty() {
            continue;
        }
        println!("\n{label}:");
        for r in *items {
            println!(
                "  {}:{}:{} {} [{} {}]",
                r.file_path, r.line, r.column, r.symbol, r.why_matched, r.confidence,
            );
        }
    }
}

pub fn print_call_path(from: &str, to: &str, path: &Option<Vec<String>>) {
    println!("command: call-path");
    println!("from: {from}");
    println!("to: {to}");
    match path {
        Some(steps) => {
            println!("path_length: {}", steps.len());
            println!("path: {}", steps.join(" -> "));
        }
        None => {
            println!("path: none (no call path found)");
        }
    }
}

#[derive(Serialize)]
struct JsonCallPathOutput<'a> {
    schema_version: u32,
    command: &'a str,
    from: &'a str,
    to: &'a str,
    path: &'a Option<Vec<String>>,
}

pub fn print_call_path_json(
    from: &str,
    to: &str,
    path: &Option<Vec<String>>,
) -> anyhow::Result<()> {
    let payload = JsonCallPathOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "call-path",
        from,
        to,
        path,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_related(symbol: &str, results: &[RelatedSymbol]) {
    println!("command: related");
    println!("query: {symbol}");
    println!("results: {}", results.len());
    for r in results {
        println!(
            "  {} ({}) in {} [{}]",
            r.symbol, r.kind, r.file_path, r.relationship,
        );
    }
}

#[derive(Serialize)]
struct JsonRelatedOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [RelatedSymbol],
}

pub fn print_related_json(symbol: &str, results: &[RelatedSymbol]) -> anyhow::Result<()> {
    let payload = JsonRelatedOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "related",
        query: symbol,
        results,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_health(report: &HealthReport, show_files: bool, show_functions: bool) {
    println!("Code health report:");
    println!();

    if show_files {
        println!("  LARGEST FILES:");
        if report.largest_files.is_empty() {
            println!("    (none above threshold)");
        }
        for (i, file) in report.largest_files.iter().enumerate() {
            println!(
                "    #{:<3} {:<40} {} lines   {} symbols",
                i + 1,
                file.file_path,
                file.line_count,
                file.symbol_count
            );
        }
        println!();
    }

    if show_functions {
        println!("  LARGEST FUNCTIONS:");
        if report.largest_functions.is_empty() {
            println!("    (none above threshold)");
        }
        for (i, func) in report.largest_functions.iter().enumerate() {
            println!(
                "    #{:<3} {:<40} {} lines    {}:{}",
                i + 1,
                func.symbol,
                func.line_count,
                func.file_path,
                func.start_line
            );
        }
    }
}

#[derive(Serialize)]
struct JsonHealthOutput<'a> {
    schema_version: u32,
    command: &'a str,
    largest_files: &'a [crate::query::diagnostics::FileHealth],
    largest_functions: &'a [crate::query::diagnostics::FunctionHealth],
}

pub fn print_health_json(report: &HealthReport) -> anyhow::Result<()> {
    let payload = JsonHealthOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "health",
        largest_files: &report.largest_files,
        largest_functions: &report.largest_functions,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_circular(report: &CircularReport) {
    if report.cycles.is_empty() {
        println!("No circular dependencies found.");
        return;
    }
    println!("Circular dependencies:");
    println!();
    for (i, cycle) in report.cycles.iter().enumerate() {
        println!("  Cycle {} ({} files):", i + 1, cycle.files.len());
        for edge in &cycle.edges {
            println!("    {} → {}", edge.from_file, edge.to_file);
            println!(
                "      via: {}::{}() {} {}::{}()",
                edge.from_file.trim_end_matches(".rs").replace('/', "::"),
                edge.from_symbol,
                edge.edge_kind,
                edge.to_file.trim_end_matches(".rs").replace('/', "::"),
                edge.to_symbol
            );
        }
        println!();
    }
    println!(
        "  Summary: {} cycle{} found",
        report.total_cycles,
        if report.total_cycles == 1 { "" } else { "s" }
    );
}

#[derive(Serialize)]
struct JsonCircularOutput<'a> {
    schema_version: u32,
    command: &'a str,
    cycles: &'a [crate::query::diagnostics::CycleDep],
    total_cycles: usize,
}

pub fn print_circular_json(report: &CircularReport) -> anyhow::Result<()> {
    let payload = JsonCircularOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "circular",
        cycles: &report.cycles,
        total_cycles: report.total_cycles,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonAnatomyOutput<'a> {
    schema_version: u32,
    command: &'a str,
    report: &'a AnatomyReport,
}

pub fn print_anatomy(report: &AnatomyReport) {
    println!("Anatomy of {}:", report.file_path);
    println!();
    println!(
        "  symbols: {} total ({} functions)",
        report.total_symbols, report.function_count
    );
    println!();
    for symbol in &report.symbols {
        let line_count = symbol
            .line_count
            .map_or_else(|| "?".to_string(), |value| value.to_string());
        println!(
            "  {} ({}) line {} size {}",
            symbol.symbol, symbol.kind, symbol.start_line, line_count
        );
    }
}

pub fn print_anatomy_json(report: &AnatomyReport) -> anyhow::Result<()> {
    let payload = JsonAnatomyOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "anatomy",
        report,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonCouplingOutput<'a> {
    schema_version: u32,
    command: &'a str,
    results: &'a [CouplingEntry],
}

pub fn print_coupling(entries: &[CouplingEntry]) {
    println!("File coupling report:");
    if entries.is_empty() {
        println!("  (no cross-file coupling found)");
        return;
    }
    for (index, entry) in entries.iter().enumerate() {
        println!(
            "  #{:<3} {} <-> {} ({} + {} = {})",
            index + 1,
            entry.file_a,
            entry.file_b,
            entry.a_to_b_edges,
            entry.b_to_a_edges,
            entry.total_edges
        );
    }
}

pub fn print_coupling_json(entries: &[CouplingEntry]) -> anyhow::Result<()> {
    let payload = JsonCouplingOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "coupling",
        results: entries,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonDeadOutput<'a> {
    schema_version: u32,
    command: &'a str,
    results: &'a [DeadSymbol],
}

pub fn print_dead(entries: &[DeadSymbol]) {
    println!("Dead symbol candidates:");
    if entries.is_empty() {
        println!("  (none)");
        return;
    }
    for entry in entries {
        println!(
            "  {} ({}) in {}:{} [{}] {}",
            entry.symbol, entry.kind, entry.file_path, entry.line, entry.confidence, entry.reason
        );
    }
}

pub fn print_dead_json(entries: &[DeadSymbol]) -> anyhow::Result<()> {
    let payload = JsonDeadOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "dead",
        results: entries,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonTestGapsOutput<'a> {
    schema_version: u32,
    command: &'a str,
    report: &'a TestGapReport,
}

pub fn print_test_gaps(report: &TestGapReport) {
    println!("Test gap analysis for {}:", report.target);
    println!("  STATUS: {}", report.analysis_state);
    println!();
    println!("  COVERED: {}", report.covered.len());
    for entry in &report.covered {
        println!(
            "    {} ({}, {} lines, {} test hits)",
            entry.symbol, entry.coverage_status, entry.line_count, entry.test_hits
        );
    }
    println!();
    println!("  UNCOVERED: {}", report.uncovered.len());
    for entry in &report.uncovered {
        println!(
            "    {} ({}, {}, {} lines)",
            entry.symbol, entry.coverage_status, entry.risk, entry.line_count
        );
    }
    let total = report.covered.len() + report.uncovered.len();
    println!();
    println!("  SUMMARY: {} / {} covered", report.covered.len(), total);
}

pub fn print_test_gaps_json(report: &TestGapReport) -> anyhow::Result<()> {
    let payload = JsonTestGapsOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "test-gaps",
        report,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonSuggestOutput<'a> {
    schema_version: u32,
    command: &'a str,
    results: &'a [Suggestion],
}

pub fn print_suggest(entries: &[Suggestion]) {
    println!("Refactoring suggestions:");
    if entries.is_empty() {
        println!("  (no candidates)");
        return;
    }
    for (index, entry) in entries.iter().enumerate() {
        println!(
            "  #{:<3} {}:{} score={:.1} lines={} fan_in={} tested={}",
            index + 1,
            entry.file_path,
            entry.symbol,
            entry.refactoring_value,
            entry.line_count,
            entry.fan_in,
            if entry.has_tests { "yes" } else { "no" }
        );
    }
}

pub fn print_suggest_json(entries: &[Suggestion]) -> anyhow::Result<()> {
    let payload = JsonSuggestOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "suggest",
        results: entries,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

#[derive(Serialize)]
struct JsonBoundaryOutput<'a> {
    schema_version: u32,
    command: &'a str,
    report: &'a BoundaryReport,
}

pub fn print_boundary(report: &BoundaryReport, public_only: bool) {
    println!("Boundary analysis for {}:", report.file_path);
    println!();
    println!("  PUBLIC:");
    if report.public_symbols.is_empty() {
        println!("    (none)");
    }
    for symbol in &report.public_symbols {
        println!(
            "    {} ({}) external_refs={}",
            symbol.symbol, symbol.kind, symbol.external_references
        );
    }
    if public_only {
        return;
    }
    println!();
    println!("  INTERNAL:");
    if report.internal_symbols.is_empty() {
        println!("    (none)");
    }
    for symbol in &report.internal_symbols {
        println!(
            "    {} ({}) external_refs={}",
            symbol.symbol, symbol.kind, symbol.external_references
        );
    }
}

pub fn print_boundary_json(report: &BoundaryReport) -> anyhow::Result<()> {
    let payload = JsonBoundaryOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "boundary",
        report,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_tree(report: &TreeReport) {
    print_tree_node(&report.root, "", true);
}

fn print_tree_node(node: &TreeNode, prefix: &str, is_last: bool) {
    let connector = if prefix.is_empty() {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    match node.kind {
        TreeNodeKind::Directory => {
            let stats = if node.total_files > 0 {
                format!(
                    " [{} files, {} symbols]",
                    node.total_files, node.total_symbols
                )
            } else {
                String::new()
            };
            println!("{prefix}{connector}{}/{stats}", node.name);
        }
        TreeNodeKind::File => {
            let line_info = node
                .line_count
                .map(|lc| format!("{lc} lines"))
                .unwrap_or_default();
            let sym_info = if node.symbol_count > 0 {
                format!(" │ {} symbols", node.symbol_count)
            } else {
                String::new()
            };
            let stats = if !line_info.is_empty() || !sym_info.is_empty() {
                format!(" [{line_info}{sym_info}]")
            } else {
                String::new()
            };
            println!("{prefix}{connector}{}{stats}", node.name);
        }
    }

    // Print dependency annotations for files
    let child_prefix = if prefix.is_empty() {
        String::new()
    } else if is_last {
        format!("{prefix}    ")
    } else {
        format!("{prefix}│   ")
    };

    if !node.imports.is_empty() {
        println!("{child_prefix}→ imports: {}", node.imports.join(", "));
    }
    if !node.used_by.is_empty() {
        println!("{child_prefix}← used by: {}", node.used_by.join(", "));
    }

    // Print symbols if present
    for sym in &node.symbols {
        println!(
            "{child_prefix}  {} {} (line {})",
            sym.kind, sym.name, sym.start_line
        );
    }

    // Print children
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == child_count - 1;
        print_tree_node(child, &child_prefix, child_is_last);
    }
}

#[derive(Serialize)]
struct JsonTreeOutput<'a> {
    schema_version: u32,
    command: &'a str,
    tree: &'a TreeNode,
}

pub fn print_tree_json(report: &TreeReport) -> anyhow::Result<()> {
    let payload = JsonTreeOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "tree",
        tree: &report.root,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}

pub fn print_orient(report: &OrientReport) {
    println!("Orientation report:");
    println!();

    println!("═══ STRUCTURE ═══");
    print_tree(&report.tree);
    println!();

    println!("═══ HEALTH ═══");
    print_health(&report.health, true, true);
    println!();

    println!("═══ HOTSPOTS ═══");
    print_hotspots(&report.hotspots);
    println!();

    println!("═══ CIRCULAR ═══");
    print_circular(&report.circular);
    println!();

    println!("═══ RECOMMENDATIONS ═══");
    if report.recommendations.is_empty() {
        println!("  No recommendations.");
    } else {
        for rec in &report.recommendations {
            println!("  {}", rec.message);
        }
    }
}

#[derive(Serialize)]
struct JsonOrientOutput<'a> {
    schema_version: u32,
    command: &'a str,
    tree: &'a TreeNode,
    health: &'a HealthReport,
    hotspots: &'a [HotspotEntry],
    circular: &'a CircularReport,
    recommendations: &'a [crate::query::orientation::Recommendation],
}

pub fn print_orient_json(report: &OrientReport) -> anyhow::Result<()> {
    let payload = JsonOrientOutput {
        schema_version: JSON_SCHEMA_VERSION_V2,
        command: "orient",
        tree: &report.tree.root,
        health: &report.health,
        hotspots: &report.hotspots,
        circular: &report.circular,
        recommendations: &report.recommendations,
    };
    let serialized = serde_json::to_string_pretty(&payload)?;
    println!("{serialized}");
    Ok(())
}
