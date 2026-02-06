use std::path::Path;

pub fn print_index(index_path: &Path, schema_version: i64, indexed_files: usize) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
    println!("indexed_files: {indexed_files}");
}

pub fn print_status(index_path: &Path, schema_version: i64) {
    println!("index_path: {}", index_path.display());
    println!("schema_version: {schema_version}");
}

pub fn print_query(command: &str, symbol: &str, result_count: usize) {
    println!("command: {command}");
    println!("query: {symbol}");
    println!("results: {result_count}");
}
