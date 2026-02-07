use std::path::Path;

use crate::query::{
    ContextMatch, DiffImpactMatch, ExplainMatch, ImpactMatch, QueryMatch, TestTarget,
    VerificationStep,
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
    budget: usize,
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
/// This writes four lines showing `index_path`, `schema_version`, `indexed_files`, and `skipped_files`.
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
    skipped_files: usize,
) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
    println!("indexed_files: {indexed_files}");
    println!("skipped_files: {skipped_files}");
}

pub fn print_status(index_path: &Path, schema_version: i64) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
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
/// let _ = crate::output::print_query_json("query", "my_symbol", &[] as &[crate::types::QueryMatch]);
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

/// Serializes impact matches using the v2 JSON schema and prints the pretty-formatted JSON to stdout.
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
pub fn print_context(task: &str, budget: usize, matches: &[ContextMatch]) {
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
/// The emitted JSON contains the schema version, command ("context"), task, budget, and the `results` array.
///
/// # Examples
///
/// ```
/// // prints a JSON object for an empty results list
/// let res = print_context_json("build-docs", 5, &[]);
/// assert!(res.is_ok());
/// ```
pub fn print_context_json(
    task: &str,
    budget: usize,
    matches: &[ContextMatch],
) -> anyhow::Result<()> {
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
