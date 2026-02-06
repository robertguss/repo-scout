use std::path::Path;

use crate::query::QueryMatch;
use serde::Serialize;

pub const JSON_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
struct JsonQueryOutput<'a> {
    schema_version: u32,
    command: &'a str,
    query: &'a str,
    results: &'a [QueryMatch],
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
