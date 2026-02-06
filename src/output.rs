use std::path::Path;

use crate::query::{ContextMatch, ImpactMatch, QueryMatch};
use serde::Serialize;

pub const JSON_SCHEMA_VERSION: u32 = 1;
pub const JSON_SCHEMA_VERSION_V2: u32 = 2;

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
