use std::path::Path;

use crate::query::QueryMatch;

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
            "{}:{}:{} {} [text_identifier_match text_fallback]",
            result.file_path, result.line, result.column, result.symbol
        );
    }
}
