use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::indexer::languages::LanguageAdapter;
use crate::indexer::languages::rust::RustLanguageAdapter;
use crate::indexer::languages::typescript::TypeScriptLanguageAdapter;

pub mod files;
pub mod languages;
pub mod rust_ast;
pub mod text;

#[derive(Debug)]
pub struct IndexSummary {
    pub indexed_files: usize,
    pub skipped_files: usize,
}

/// Builds or refreshes an index of source files from `repo` into the SQLite database at `db_path`.
///
/// This function discovers source files, prunes database rows for files no longer present, and for
/// each file that changed it updates token occurrences, AST definitions and references, symbols,
/// and symbol edges. Processing for each file is performed inside a database transaction so that
/// the file's related rows are replaced atomically. Rust files receive additional AST parsing and
/// relation-hint extraction (imports, impls) which are incorporated into symbol edges.
///
/// # Returns
///
/// An `IndexSummary` containing the number of files that were indexed and the number of files
/// that were skipped because their content hash did not change.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// // `repo` should be a path to a source workspace and `db_path` to a writable SQLite file.
/// let summary = index_repository(Path::new("path/to/repo"), Path::new("path/to/db.sqlite")).unwrap();
/// // summary contains counts of processed and skipped files
/// assert!(summary.indexed_files >= 0);
/// assert!(summary.skipped_files >= 0);
/// ```
pub fn index_repository(repo: &Path, db_path: &Path) -> anyhow::Result<IndexSummary> {
    let mut connection = Connection::open(db_path)?;
    let source_files = files::discover_source_files(repo)?;
    let live_paths: HashSet<String> = source_files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect();

    prune_stale_file_rows(&mut connection, &live_paths)?;

    let mut indexed_files = 0usize;
    let mut skipped_files = 0usize;
    let mut deferred_edges: Vec<(String, String, String, f64, String)> = Vec::new();

    for file in source_files {
        let existing_hash: Option<String> = connection
            .query_row(
                "SELECT content_hash FROM indexed_files WHERE file_path = ?1",
                [&file.relative_path],
                |row| row.get(0),
            )
            .optional()?;

        if existing_hash.as_deref() == Some(file.content_hash.as_str()) {
            skipped_files += 1;
            continue;
        }

        let text_content = std::str::from_utf8(&file.bytes).ok();
        let token_occurrences = text_content
            .map(text::extract_token_occurrences)
            .unwrap_or_default();
        let extraction_unit = text_content
            .map(|source| extract_with_adapter(&file.relative_path, source))
            .transpose()?
            .unwrap_or_default();
        let languages::ExtractionUnit {
            symbols: extracted_symbols,
            references: extracted_references,
            edges: extracted_edges,
        } = extraction_unit;
        let mut reusable_symbol_ids = existing_symbol_ids(&connection, &file.relative_path)?;
        let mut next_symbol_id = next_symbol_id_start(&connection)?;

        let tx = connection.transaction()?;
        tx.execute(
            "DELETE FROM text_occurrences WHERE file_path = ?1",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM ast_definitions WHERE file_path = ?1",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM ast_references WHERE file_path = ?1",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM symbol_edges_v2
             WHERE from_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)
                OR to_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM symbols_v2 WHERE file_path = ?1",
            [&file.relative_path],
        )?;

        for occurrence in token_occurrences {
            tx.execute(
                "INSERT INTO text_occurrences(file_path, symbol, line, column)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    file.relative_path,
                    occurrence.symbol,
                    i64::from(occurrence.line),
                    i64::from(occurrence.column)
                ],
            )?;
        }

        let pending_edges: Vec<(String, String, String, f64, String)> = extracted_edges
            .into_iter()
            .map(|edge| {
                (
                    edge.from_symbol_key.symbol,
                    edge.to_symbol_key.symbol,
                    edge.edge_kind,
                    edge.confidence,
                    edge.provenance,
                )
            })
            .collect();

        for definition in extracted_symbols {
            let symbol = definition.symbol;
            let kind = definition.kind;
            let language = definition.language;
            let qualified_symbol = definition
                .qualified_symbol
                .unwrap_or_else(|| format!("{language}:{}::{symbol}", file.relative_path));
            let container = definition.container;
            let start_line = i64::from(definition.start_line);
            let start_column = i64::from(definition.start_column);
            let end_line = i64::from(definition.end_line);
            let end_column = i64::from(definition.end_column);
            let signature = definition.signature;
            let symbol_id = take_reusable_symbol_id(&mut reusable_symbol_ids, &symbol, &kind)
                .unwrap_or_else(|| {
                    let generated = next_symbol_id;
                    next_symbol_id += 1;
                    generated
                });

            tx.execute(
                "INSERT INTO ast_definitions(file_path, symbol, kind, line, column)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    &file.relative_path,
                    &symbol,
                    &kind,
                    start_line,
                    start_column
                ],
            )?;
            tx.execute(
                "INSERT INTO symbols_v2(
                    symbol_id, file_path, symbol, kind, language, qualified_symbol, container, start_line, start_column, end_line, end_column, signature
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    symbol_id,
                    &file.relative_path,
                    &symbol,
                    &kind,
                    &language,
                    &qualified_symbol,
                    container.as_deref(),
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                    signature.as_deref()
                ],
            )?;
        }

        for reference in extracted_references {
            let symbol = reference.symbol;
            tx.execute(
                "INSERT INTO ast_references(file_path, symbol, line, column)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    file.relative_path,
                    &symbol,
                    i64::from(reference.line),
                    i64::from(reference.column)
                ],
            )?;
        }

        for (from_symbol, to_symbol, edge_kind, confidence, provenance) in pending_edges {
            let Some(from_symbol_id) = resolve_symbol_id_in_tx(&tx, &from_symbol)? else {
                deferred_edges.push((from_symbol, to_symbol, edge_kind, confidence, provenance));
                continue;
            };
            let Some(to_symbol_id) = resolve_symbol_id_in_tx(&tx, &to_symbol)? else {
                deferred_edges.push((from_symbol, to_symbol, edge_kind, confidence, provenance));
                continue;
            };
            if matches!(edge_kind.as_str(), "imports" | "implements")
                && symbol_kind_by_id_in_tx(&tx, to_symbol_id)?.as_deref() == Some("import")
            {
                deferred_edges.push((from_symbol, to_symbol, edge_kind, confidence, provenance));
                continue;
            }

            tx.execute(
                "INSERT INTO symbol_edges_v2(from_symbol_id, to_symbol_id, edge_kind, confidence, provenance)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(from_symbol_id, to_symbol_id, edge_kind)
                 DO UPDATE SET confidence = excluded.confidence, provenance = excluded.provenance",
                params![from_symbol_id, to_symbol_id, edge_kind, confidence, provenance],
            )?;
        }

        tx.execute(
            "INSERT INTO indexed_files(file_path, content_hash)
             VALUES (?1, ?2)
             ON CONFLICT(file_path) DO UPDATE SET content_hash = excluded.content_hash",
            params![file.relative_path, file.content_hash],
        )?;
        tx.commit()?;

        indexed_files += 1;
    }

    if !deferred_edges.is_empty() {
        let tx = connection.transaction()?;
        for (from_symbol, to_symbol, edge_kind, confidence, provenance) in deferred_edges {
            let Some(from_symbol_id) = resolve_symbol_id_in_tx(&tx, &from_symbol)? else {
                continue;
            };
            let Some(to_symbol_id) = resolve_symbol_id_in_tx(&tx, &to_symbol)? else {
                continue;
            };

            tx.execute(
                "INSERT INTO symbol_edges_v2(from_symbol_id, to_symbol_id, edge_kind, confidence, provenance)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(from_symbol_id, to_symbol_id, edge_kind)
                 DO UPDATE SET confidence = excluded.confidence, provenance = excluded.provenance",
                params![from_symbol_id, to_symbol_id, edge_kind, confidence, provenance],
            )?;
        }
        tx.commit()?;
    }

    Ok(IndexSummary {
        indexed_files,
        skipped_files,
    })
}

/// Remove database rows for files that are no longer present in the workspace.
///
/// This function deletes all rows associated with any file listed in `indexed_files`
/// that are not contained in `live_paths`. For each stale file it removes related
/// rows from `text_occurrences`, `ast_definitions`, `ast_references`, `symbol_edges_v2`,
/// `symbols_v2`, and `indexed_files`. Deletions are performed inside a single
/// transaction so each stale file's removals are applied atomically.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
/// use std::collections::HashSet;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// conn.execute_batch(r#"
/// CREATE TABLE indexed_files(file_path TEXT PRIMARY KEY, content_hash TEXT);
/// CREATE TABLE text_occurrences(file_path TEXT, symbol TEXT, line INTEGER, column INTEGER);
/// CREATE TABLE ast_definitions(file_path TEXT);
/// CREATE TABLE ast_references(file_path TEXT);
/// CREATE TABLE symbols_v2(symbol_id INTEGER PRIMARY KEY, file_path TEXT);
/// CREATE TABLE symbol_edges_v2(from_symbol_id INTEGER, to_symbol_id INTEGER, edge_kind TEXT);
/// "#).unwrap();
///
/// conn.execute("INSERT INTO indexed_files(file_path, content_hash) VALUES (?1, ?2)", ["a.rs", "h"]).unwrap();
/// let mut live = HashSet::new(); // empty => `a.rs` is stale
///
/// super::prune_stale_file_rows(&mut conn, &live).unwrap();
///
/// let count: i64 = conn.query_row("SELECT COUNT(*) FROM indexed_files", [], |r| r.get(0)).unwrap();
/// assert_eq!(count, 0);
/// ```
fn prune_stale_file_rows(
    connection: &mut Connection,
    live_paths: &HashSet<String>,
) -> anyhow::Result<()> {
    let stale_paths = {
        let mut statement =
            connection.prepare("SELECT file_path FROM indexed_files ORDER BY file_path ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

        let mut stale_paths = Vec::new();
        for row in rows {
            let path = row?;
            if !live_paths.contains(&path) {
                stale_paths.push(path);
            }
        }
        stale_paths
    };

    if stale_paths.is_empty() {
        return Ok(());
    }

    let tx = connection.transaction()?;
    for path in stale_paths {
        tx.execute("DELETE FROM text_occurrences WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM ast_definitions WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM ast_references WHERE file_path = ?1", [&path])?;
        tx.execute(
            "DELETE FROM symbol_edges_v2
             WHERE from_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)
                OR to_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)",
            [&path],
        )?;
        tx.execute("DELETE FROM symbols_v2 WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM indexed_files WHERE file_path = ?1", [&path])?;
    }
    tx.commit()?;

    Ok(())
}

/// Finds the database `symbol_id` for the given symbol name within the provided transaction.
///
/// This queries the `symbols_v2` table for rows matching `symbol` and returns the first
/// `symbol_id` ordered by `file_path`, `start_line`, then `start_column`. Order ties are
/// resolved by that ordering so the returned ID is the earliest occurrence by location.
///
/// # Returns
///
/// `Ok(Some(symbol_id))` with the matching symbol ID if one exists, `Ok(None)` if no row matches,
/// or an `Err` if the database query fails.
///
/// # Examples
///
/// ```ignore
/// // Illustrative usage (not compiled in doctest):
/// let tx: rusqlite::Transaction = /* obtain a transaction */ unimplemented!();
/// let id = resolve_symbol_id_in_tx(&tx, "my_crate::MyType")?;
/// if let Some(symbol_id) = id {
///     println!("Found symbol id: {}", symbol_id);
/// } else {
///     println!("Symbol not found");
/// }
/// ```
fn resolve_symbol_id_in_tx(
    tx: &rusqlite::Transaction<'_>,
    symbol: &str,
) -> anyhow::Result<Option<i64>> {
    let symbol_id = tx
        .query_row(
            "SELECT symbol_id
             FROM symbols_v2
             WHERE symbol = ?1
             ORDER BY CASE WHEN kind = 'import' THEN 1 ELSE 0 END ASC,
                      file_path ASC, start_line ASC, start_column ASC
             LIMIT 1",
            [symbol],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    Ok(symbol_id)
}

fn symbol_kind_by_id_in_tx(
    tx: &rusqlite::Transaction<'_>,
    symbol_id: i64,
) -> anyhow::Result<Option<String>> {
    let kind = tx
        .query_row(
            "SELECT kind
             FROM symbols_v2
             WHERE symbol_id = ?1",
            [symbol_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(kind)
}

/// Builds a mapping from (symbol, kind) to a list of existing `symbol_id`s for a given file.
///
/// The returned map groups all rows from `symbols_v2` for `file_path` by the tuple `(symbol, kind)`.
/// Each vector contains `symbol_id`s in ascending order (by `symbol_id`).
///
/// # Examples
///
/// ```no_run
/// # use rusqlite::Connection;
/// # fn example(conn: &Connection) -> anyhow::Result<()> {
/// let map = existing_symbol_ids(conn, "src/lib.rs")?;
/// if let Some(ids) = map.get(&("my_crate::foo".to_string(), "fn".to_string())) {
///     // ids is a Vec<i64> of symbol_id values ordered ascending
///     println!("found {} ids", ids.len());
/// }
/// # Ok(())
/// # }
/// ```
fn existing_symbol_ids(
    connection: &Connection,
    file_path: &str,
) -> anyhow::Result<HashMap<(String, String), Vec<i64>>> {
    let mut statement = connection.prepare(
        "SELECT symbol_id, symbol, kind
         FROM symbols_v2
         WHERE file_path = ?1
         ORDER BY symbol_id ASC",
    )?;
    let rows = statement.query_map([file_path], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut by_symbol_kind: HashMap<(String, String), Vec<i64>> = HashMap::new();
    for row in rows {
        let (symbol_id, symbol, kind) = row?;
        by_symbol_kind
            .entry((symbol, kind))
            .or_default()
            .push(symbol_id);
    }
    Ok(by_symbol_kind)
}

/// Compute the next available `symbol_id` for the `symbols_v2` table.
///
/// Queries `symbols_v2` for the current maximum `symbol_id` and returns one greater than that
/// value (or `1` if no rows exist).
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
///
/// // create an in-memory DB and the minimal table
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute_batch("CREATE TABLE symbols_v2 (symbol_id INTEGER PRIMARY KEY, symbol TEXT);").unwrap();
///
/// // no rows -> next id is 1
/// let next = next_symbol_id_start(&conn).unwrap();
/// assert_eq!(next, 1);
///
/// // insert a row with symbol_id = 5
/// conn.execute("INSERT INTO symbols_v2 (symbol_id, symbol) VALUES (?1, ?2);", &[&5i64, &"foo"]).unwrap();
/// let next = next_symbol_id_start(&conn).unwrap();
/// assert_eq!(next, 6);
/// ```
fn next_symbol_id_start(connection: &Connection) -> anyhow::Result<i64> {
    let max_id: i64 = connection.query_row(
        "SELECT COALESCE(MAX(symbol_id), 0) FROM symbols_v2",
        [],
        |row| row.get(0),
    )?;
    Ok(max_id + 1)
}

/// Consume and return a reusable symbol ID for a given `(symbol, kind)` pair if one exists.
///
/// Searches `reusable_symbol_ids` for the key `(symbol, kind)`, removes and returns the first
/// ID from the associated vector if present, and leaves the vector mutated (consuming that ID).
///
/// # Parameters
///
/// - `reusable_symbol_ids`: mutable map from `(symbol, kind)` to a list of reusable symbol IDs.
/// - `symbol`: symbol name to look up.
/// - `kind`: symbol kind to look up.
///
/// # Returns
///
/// `Some(id)` with the consumed symbol ID if an ID was available for the `(symbol, kind)` key,
/// `None` otherwise.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// let mut map: HashMap<(String, String), Vec<i64>> = HashMap::new();
/// map.insert(("foo".to_string(), "fn".to_string()), vec![42]);
/// let id = take_reusable_symbol_id(&mut map, "foo", "fn");
/// assert_eq!(id, Some(42));
/// assert_eq!(map.get(&("foo".to_string(), "fn".to_string())).unwrap().len(), 0);
/// ```
fn take_reusable_symbol_id(
    reusable_symbol_ids: &mut HashMap<(String, String), Vec<i64>>,
    symbol: &str,
    kind: &str,
) -> Option<i64> {
    let ids = reusable_symbol_ids.get_mut(&(symbol.to_string(), kind.to_string()))?;
    if ids.is_empty() {
        return None;
    }
    Some(ids.remove(0))
}

fn extract_with_adapter(
    file_path: &str,
    source: &str,
) -> anyhow::Result<languages::ExtractionUnit> {
    let rust_adapter = RustLanguageAdapter;
    let typescript_adapter = TypeScriptLanguageAdapter;

    for adapter in [
        &rust_adapter as &dyn LanguageAdapter,
        &typescript_adapter as &dyn LanguageAdapter,
    ] {
        if adapter
            .file_extensions()
            .iter()
            .any(|extension| file_path.ends_with(&format!(".{extension}")))
        {
            return adapter.extract(file_path, source);
        }
    }

    Ok(languages::ExtractionUnit::default())
}
