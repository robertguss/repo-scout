use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::indexer::languages::LanguageAdapter;
use crate::indexer::languages::go::GoLanguageAdapter;
use crate::indexer::languages::python::PythonLanguageAdapter;
use crate::indexer::languages::rust::RustLanguageAdapter;
use crate::indexer::languages::typescript::TypeScriptLanguageAdapter;

pub mod files;
pub mod languages;
pub mod rust_ast;
pub mod text;

#[derive(Debug)]
pub struct IndexSummary {
    pub indexed_files: usize,
    pub non_source_files: usize,
}

type DeferredEdge = (
    languages::SymbolKey,
    languages::SymbolKey,
    String,
    f64,
    String,
);

#[derive(Debug)]
enum FileIndexOutcome {
    Indexed,
    Skipped,
}

#[derive(Debug)]
struct PreparedFileData {
    token_occurrences: Vec<text::TokenOccurrence>,
    extracted_symbols: Vec<languages::ExtractedSymbol>,
    extracted_references: Vec<languages::ExtractedReference>,
    pending_edges: Vec<DeferredEdge>,
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
/// assert!(summary.non_source_files >= 0);
/// ```
pub fn index_repository(repo: &Path, db_path: &Path) -> anyhow::Result<IndexSummary> {
    let mut connection = Connection::open(db_path)?;
    let source_files = files::discover_source_files(repo)?;
    let live_paths: HashSet<String> = source_files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect();
    prune_stale_file_rows(&mut connection, &live_paths)?;
    let mut summary = IndexSummary {
        indexed_files: 0,
        non_source_files: 0,
    };
    let mut deferred_edges = Vec::new();
    for file in source_files {
        match index_file(&mut connection, file, &mut deferred_edges)? {
            FileIndexOutcome::Indexed => summary.indexed_files += 1,
            FileIndexOutcome::Skipped => summary.non_source_files += 1,
        }
    }
    replay_deferred_edges(&mut connection, deferred_edges)?;
    Ok(summary)
}

fn index_file(
    connection: &mut Connection,
    file: files::SourceFile,
    deferred_edges: &mut Vec<DeferredEdge>,
) -> anyhow::Result<FileIndexOutcome> {
    if file_is_unchanged(connection, &file)? {
        return Ok(FileIndexOutcome::Skipped);
    }
    let prepared = prepare_file_data(&file)?;
    let mut reusable_symbol_ids = existing_symbol_ids(connection, &file.relative_path)?;
    let mut next_symbol_id = next_symbol_id_start(connection)?;
    let tx = connection.transaction()?;
    clear_file_rows(&tx, &file.relative_path)?;
    insert_text_occurrences(&tx, &file.relative_path, prepared.token_occurrences)?;
    let insert_symbols_result = insert_symbols(
        &tx,
        &file.relative_path,
        prepared.extracted_symbols,
        &mut reusable_symbol_ids,
        &mut next_symbol_id,
    );
    insert_symbols_result?;
    insert_references(&tx, &file.relative_path, prepared.extracted_references)?;
    insert_or_defer_edges(&tx, prepared.pending_edges, deferred_edges)?;
    upsert_indexed_file_row(&tx, &file.relative_path, &file.content_hash)?;
    tx.commit()?;
    Ok(FileIndexOutcome::Indexed)
}

fn file_is_unchanged(connection: &Connection, file: &files::SourceFile) -> anyhow::Result<bool> {
    let existing_hash: Option<String> = connection
        .query_row(
            "SELECT content_hash FROM indexed_files WHERE file_path = ?1",
            [&file.relative_path],
            |row| row.get(0),
        )
        .optional()?;
    Ok(existing_hash.as_deref() == Some(file.content_hash.as_str()))
}

fn prepare_file_data(file: &files::SourceFile) -> anyhow::Result<PreparedFileData> {
    let text_content = std::str::from_utf8(&file.bytes).ok();
    let token_occurrences = text_content
        .map(text::extract_token_occurrences)
        .unwrap_or_default();
    let extraction_unit = text_content
        .map(|source| extract_with_adapter(&file.relative_path, source))
        .transpose()?
        .unwrap_or_default();
    let pending_edges = extraction_unit
        .edges
        .into_iter()
        .map(|edge| {
            (
                edge.from_symbol_key,
                edge.to_symbol_key,
                edge.edge_kind,
                edge.confidence,
                edge.provenance,
            )
        })
        .collect();
    Ok(PreparedFileData {
        token_occurrences,
        extracted_symbols: extraction_unit.symbols,
        extracted_references: extraction_unit.references,
        pending_edges,
    })
}

fn clear_file_rows(tx: &rusqlite::Transaction<'_>, file_path: &str) -> anyhow::Result<()> {
    tx.execute(
        "DELETE FROM text_occurrences WHERE file_path = ?1",
        [file_path],
    )?;
    tx.execute(
        "DELETE FROM ast_definitions WHERE file_path = ?1",
        [file_path],
    )?;
    tx.execute(
        "DELETE FROM ast_references WHERE file_path = ?1",
        [file_path],
    )?;
    tx.execute(
        "DELETE FROM symbol_edges_v2
         WHERE from_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)
            OR to_symbol_id IN (SELECT symbol_id FROM symbols_v2 WHERE file_path = ?1)",
        [file_path],
    )?;
    tx.execute("DELETE FROM symbols_v2 WHERE file_path = ?1", [file_path])?;
    Ok(())
}

fn insert_text_occurrences(
    tx: &rusqlite::Transaction<'_>,
    file_path: &str,
    token_occurrences: Vec<text::TokenOccurrence>,
) -> anyhow::Result<()> {
    for occurrence in token_occurrences {
        tx.execute(
            "INSERT INTO text_occurrences(file_path, symbol, line, column)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                file_path,
                occurrence.symbol,
                i64::from(occurrence.line),
                i64::from(occurrence.column)
            ],
        )?;
    }
    Ok(())
}

fn insert_symbols(
    tx: &rusqlite::Transaction<'_>,
    file_path: &str,
    extracted_symbols: Vec<languages::ExtractedSymbol>,
    reusable_symbol_ids: &mut HashMap<(String, String), Vec<i64>>,
    next_symbol_id: &mut i64,
) -> anyhow::Result<()> {
    for definition in extracted_symbols {
        let symbol = definition.symbol;
        let kind = definition.kind;
        let language = definition.language;
        let qualified_symbol = definition
            .qualified_symbol
            .unwrap_or_else(|| format!("{language}:{file_path}::{symbol}"));
        let symbol_id = take_reusable_symbol_id(reusable_symbol_ids, &symbol, &kind)
            .unwrap_or_else(|| {
                let generated = *next_symbol_id;
                *next_symbol_id += 1;
                generated
            });
        tx.execute(
            "INSERT INTO ast_definitions(file_path, symbol, kind, line, column)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                file_path,
                &symbol,
                &kind,
                i64::from(definition.start_line),
                i64::from(definition.start_column)
            ],
        )?;
        tx.execute(
            "INSERT INTO symbols_v2(
                symbol_id, file_path, symbol, kind, language, qualified_symbol, container, start_line, start_column, end_line, end_column, signature
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                symbol_id,
                file_path,
                &symbol,
                &kind,
                &language,
                &qualified_symbol,
                definition.container.as_deref(),
                i64::from(definition.start_line),
                i64::from(definition.start_column),
                i64::from(definition.end_line),
                i64::from(definition.end_column),
                definition.signature.as_deref()
            ],
        )?;
    }
    Ok(())
}

fn insert_references(
    tx: &rusqlite::Transaction<'_>,
    file_path: &str,
    extracted_references: Vec<languages::ExtractedReference>,
) -> anyhow::Result<()> {
    for reference in extracted_references {
        tx.execute(
            "INSERT INTO ast_references(file_path, symbol, line, column)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                file_path,
                &reference.symbol,
                i64::from(reference.line),
                i64::from(reference.column)
            ],
        )?;
    }
    Ok(())
}

fn insert_or_defer_edges(
    tx: &rusqlite::Transaction<'_>,
    pending_edges: Vec<DeferredEdge>,
    deferred_edges: &mut Vec<DeferredEdge>,
) -> anyhow::Result<()> {
    for (from_symbol_key, to_symbol_key, edge_kind, confidence, provenance) in pending_edges {
        let Some(from_symbol_id) = resolve_symbol_id_in_tx(tx, &from_symbol_key)? else {
            deferred_edges.push((
                from_symbol_key,
                to_symbol_key,
                edge_kind,
                confidence,
                provenance,
            ));
            continue;
        };
        let Some(to_symbol_id) = resolve_symbol_id_in_tx(tx, &to_symbol_key)? else {
            deferred_edges.push((
                from_symbol_key,
                to_symbol_key,
                edge_kind,
                confidence,
                provenance,
            ));
            continue;
        };
        if should_defer_import_edge(tx, &edge_kind, to_symbol_id)? {
            deferred_edges.push((
                from_symbol_key,
                to_symbol_key,
                edge_kind,
                confidence,
                provenance,
            ));
            continue;
        }
        let insert_edge_result = insert_symbol_edge(
            tx,
            from_symbol_id,
            to_symbol_id,
            &edge_kind,
            confidence,
            &provenance,
        );
        insert_edge_result?;
    }
    Ok(())
}

fn upsert_indexed_file_row(
    tx: &rusqlite::Transaction<'_>,
    file_path: &str,
    content_hash: &str,
) -> anyhow::Result<()> {
    tx.execute(
        "INSERT INTO indexed_files(file_path, content_hash)
         VALUES (?1, ?2)
         ON CONFLICT(file_path) DO UPDATE SET content_hash = excluded.content_hash",
        params![file_path, content_hash],
    )?;
    Ok(())
}

fn replay_deferred_edges(
    connection: &mut Connection,
    deferred_edges: Vec<DeferredEdge>,
) -> anyhow::Result<()> {
    if deferred_edges.is_empty() {
        return Ok(());
    }
    let tx = connection.transaction()?;
    for (from_symbol_key, to_symbol_key, edge_kind, confidence, provenance) in deferred_edges {
        let Some(from_symbol_id) = resolve_symbol_id_in_tx(&tx, &from_symbol_key)? else {
            continue;
        };
        let Some(to_symbol_id) = resolve_symbol_id_in_tx(&tx, &to_symbol_key)? else {
            continue;
        };
        if should_defer_import_edge(&tx, &edge_kind, to_symbol_id)? {
            continue;
        }
        let insert_edge_result = insert_symbol_edge(
            &tx,
            from_symbol_id,
            to_symbol_id,
            &edge_kind,
            confidence,
            &provenance,
        );
        insert_edge_result?;
    }
    tx.commit()?;
    Ok(())
}

fn should_defer_import_edge(
    tx: &rusqlite::Transaction<'_>,
    edge_kind: &str,
    to_symbol_id: i64,
) -> anyhow::Result<bool> {
    if !matches!(edge_kind, "imports" | "implements") {
        return Ok(false);
    }
    Ok(symbol_kind_by_id_in_tx(tx, to_symbol_id)?.as_deref() == Some("import"))
}

fn insert_symbol_edge(
    tx: &rusqlite::Transaction<'_>,
    from_symbol_id: i64,
    to_symbol_id: i64,
    edge_kind: &str,
    confidence: f64,
    provenance: &str,
) -> anyhow::Result<()> {
    tx.execute(
        "INSERT INTO symbol_edges_v2(from_symbol_id, to_symbol_id, edge_kind, confidence, provenance)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(from_symbol_id, to_symbol_id, edge_kind)
         DO UPDATE SET confidence = excluded.confidence, provenance = excluded.provenance",
        params![from_symbol_id, to_symbol_id, edge_kind, confidence, provenance],
    )?;
    Ok(())
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

/// Resolves a symbol row using deterministic disambiguation hints.
///
/// Resolution order is:
/// 1. exact `qualified_symbol` match,
/// 2. exact `(file_path, symbol)` match with non-import preference,
/// 3. unique global `symbol` match,
/// 4. unresolved (`None`) when ambiguous.
///
/// # Returns
///
/// `Ok(Some(symbol_id))` when a deterministic match exists, `Ok(None)` when unresolved, or an
/// error if the query fails.
///
/// # Examples
///
/// ```ignore
/// // Illustrative usage (not compiled in doctest):
/// let tx: rusqlite::Transaction = /* obtain a transaction */ unimplemented!();
/// let id = resolve_symbol_id_in_tx(&tx, &crate::indexer::languages::SymbolKey {
///     symbol: "my_symbol".to_string(),
///     qualified_symbol: Some("rust:src/lib.rs::my_symbol".to_string()),
///     file_path: Some("src/lib.rs".to_string()),
///     language: Some("rust".to_string()),
/// })?;
/// if let Some(symbol_id) = id {
///     println!("Found symbol id: {}", symbol_id);
/// } else {
///     println!("Symbol not found");
/// }
/// ```
fn resolve_symbol_id_in_tx(
    tx: &rusqlite::Transaction<'_>,
    key: &languages::SymbolKey,
) -> anyhow::Result<Option<i64>> {
    if let Some(qualified_match) = resolve_symbol_id_by_qualified(tx, key)? {
        return Ok(Some(qualified_match));
    }
    if let Some(scoped_match) = resolve_symbol_id_by_scope(tx, key)? {
        return Ok(Some(scoped_match));
    }
    if key.qualified_symbol.is_some() {
        return Ok(None);
    }
    resolve_symbol_id_by_symbol(tx, key)
}

fn resolve_symbol_id_by_qualified(
    tx: &rusqlite::Transaction<'_>,
    key: &languages::SymbolKey,
) -> anyhow::Result<Option<i64>> {
    let Some(qualified_symbol) = key.qualified_symbol.as_deref() else {
        return Ok(None);
    };
    tx.query_row(
        "SELECT symbol_id
         FROM symbols_v2
         WHERE qualified_symbol = ?1
         ORDER BY CASE WHEN kind = 'import' THEN 1 ELSE 0 END ASC,
                  file_path ASC, start_line ASC, start_column ASC
         LIMIT 1",
        [qualified_symbol],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(Into::into)
}

fn resolve_symbol_id_by_scope(
    tx: &rusqlite::Transaction<'_>,
    key: &languages::SymbolKey,
) -> anyhow::Result<Option<i64>> {
    let Some(file_path) = key.file_path.as_deref() else {
        return Ok(None);
    };
    tx.query_row(
        "SELECT symbol_id
         FROM symbols_v2
         WHERE file_path = ?1
           AND symbol = ?2
         ORDER BY CASE WHEN kind = 'import' THEN 1 ELSE 0 END ASC,
                  start_line ASC, start_column ASC
         LIMIT 1",
        params![file_path, key.symbol],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(Into::into)
}

fn resolve_symbol_id_by_symbol(
    tx: &rusqlite::Transaction<'_>,
    key: &languages::SymbolKey,
) -> anyhow::Result<Option<i64>> {
    let query_with_language = "SELECT symbol_id
         FROM symbols_v2
         WHERE symbol = ?1
           AND language = ?2
         ORDER BY CASE WHEN kind = 'import' THEN 1 ELSE 0 END ASC,
                  file_path ASC, start_line ASC, start_column ASC
         LIMIT 2";
    let query_without_language = "SELECT symbol_id
         FROM symbols_v2
         WHERE symbol = ?1
         ORDER BY CASE WHEN kind = 'import' THEN 1 ELSE 0 END ASC,
                  file_path ASC, start_line ASC, start_column ASC
         LIMIT 2";
    let mut statement = if key.language.is_some() {
        tx.prepare(query_with_language)?
    } else {
        tx.prepare(query_without_language)?
    };
    let mut rows = if let Some(language) = key.language.as_deref() {
        statement.query(params![key.symbol, language])?
    } else {
        statement.query([key.symbol.as_str()])?
    };
    let Some(first_row) = rows.next()? else {
        return Ok(None);
    };
    let first_symbol_id = first_row.get::<_, i64>(0)?;
    if rows.next()?.is_some() {
        return Ok(None);
    }

    Ok(Some(first_symbol_id))
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
        "SELECT symbol_id, symbol, kind FROM symbols_v2 WHERE file_path = ?1 ORDER BY symbol_id ASC",
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
    let python_adapter = PythonLanguageAdapter;
    let go_adapter = GoLanguageAdapter;

    for adapter in [
        &rust_adapter as &dyn LanguageAdapter,
        &typescript_adapter as &dyn LanguageAdapter,
        &python_adapter as &dyn LanguageAdapter,
        &go_adapter as &dyn LanguageAdapter,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::schema;
    use rusqlite::params;

    fn bootstrap_connection() -> Connection {
        let connection = Connection::open_in_memory().expect("in-memory sqlite should open");
        schema::bootstrap_schema(&connection).expect("schema should bootstrap");
        connection
    }

    fn insert_symbol_row(
        connection: &Connection,
        symbol_id: i64,
        file_path: &str,
        symbol: &str,
        kind: &str,
        language: &str,
        qualified_symbol: Option<&str>,
    ) {
        connection
            .execute(
                "INSERT INTO symbols_v2(
                    symbol_id, file_path, symbol, kind, language, qualified_symbol, container,
                    start_line, start_column, end_line, end_column, signature
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 1, 1, 1, 2, NULL)",
                params![
                    symbol_id,
                    file_path,
                    symbol,
                    kind,
                    language,
                    qualified_symbol
                ],
            )
            .expect("symbol row should insert");
    }

    fn symbol_key(
        symbol: &str,
        file_path: Option<&str>,
        language: Option<&str>,
        qualified_symbol: Option<&str>,
    ) -> languages::SymbolKey {
        languages::SymbolKey {
            symbol: symbol.to_string(),
            qualified_symbol: qualified_symbol.map(str::to_string),
            file_path: file_path.map(str::to_string),
            language: language.map(str::to_string),
        }
    }

    #[test]
    fn index_file_indexes_changed_source_file() {
        let mut connection = bootstrap_connection();
        let bytes = b"notes about symbols\n".to_vec();
        let source_file = files::SourceFile {
            relative_path: "notes/readme.txt".to_string(),
            content_hash: blake3::hash(&bytes).to_hex().to_string(),
            bytes,
        };
        let mut deferred_edges = Vec::new();
        let outcome = index_file(&mut connection, source_file, &mut deferred_edges)
            .expect("index_file should succeed");
        assert!(matches!(outcome, FileIndexOutcome::Indexed));
    }

    #[test]
    fn insert_or_defer_edges_covers_missing_and_insert_paths() {
        let mut connection = bootstrap_connection();
        insert_symbol_row(
            &connection,
            1,
            "src/lib.rs",
            "caller",
            "function",
            "rust",
            Some("rust:src/lib.rs::caller"),
        );
        insert_symbol_row(
            &connection,
            2,
            "src/lib.rs",
            "callee",
            "function",
            "rust",
            Some("rust:src/lib.rs::callee"),
        );
        insert_symbol_row(
            &connection,
            3,
            "src/lib.rs",
            "imported",
            "import",
            "rust",
            Some("rust:src/lib.rs::imported"),
        );

        let tx = connection.transaction().expect("transaction should start");
        let mut deferred_edges = Vec::new();
        insert_or_defer_edges(
            &tx,
            vec![
                (
                    symbol_key(
                        "missing_from",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::missing_from"),
                    ),
                    symbol_key(
                        "callee",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::callee"),
                    ),
                    "calls".to_string(),
                    0.95,
                    "call_resolution".to_string(),
                ),
                (
                    symbol_key(
                        "caller",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::caller"),
                    ),
                    symbol_key(
                        "callee",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::callee"),
                    ),
                    "calls".to_string(),
                    0.95,
                    "call_resolution".to_string(),
                ),
                (
                    symbol_key(
                        "caller",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::caller"),
                    ),
                    symbol_key(
                        "imported",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::imported"),
                    ),
                    "imports".to_string(),
                    0.9,
                    "import_resolution".to_string(),
                ),
            ],
            &mut deferred_edges,
        )
        .expect("edge insertion should succeed");

        let edge_count: i64 = tx
            .query_row("SELECT COUNT(*) FROM symbol_edges_v2", [], |row| row.get(0))
            .expect("edge count should query");
        assert_eq!(edge_count, 1);
        assert_eq!(
            deferred_edges.len(),
            2,
            "missing-from and deferred import edges should be queued"
        );
        tx.commit().expect("transaction should commit");
    }

    #[test]
    fn replay_deferred_edges_skips_missing_and_replays_resolved_edges() {
        let mut connection = bootstrap_connection();
        insert_symbol_row(
            &connection,
            1,
            "src/lib.rs",
            "caller",
            "function",
            "rust",
            Some("rust:src/lib.rs::caller"),
        );
        insert_symbol_row(
            &connection,
            2,
            "src/lib.rs",
            "callee",
            "function",
            "rust",
            Some("rust:src/lib.rs::callee"),
        );

        replay_deferred_edges(
            &mut connection,
            vec![
                (
                    symbol_key(
                        "missing_from",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::missing_from"),
                    ),
                    symbol_key(
                        "callee",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::callee"),
                    ),
                    "calls".to_string(),
                    0.95,
                    "call_resolution".to_string(),
                ),
                (
                    symbol_key(
                        "caller",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::caller"),
                    ),
                    symbol_key(
                        "callee",
                        Some("src/lib.rs"),
                        Some("rust"),
                        Some("rust:src/lib.rs::callee"),
                    ),
                    "calls".to_string(),
                    0.95,
                    "call_resolution".to_string(),
                ),
            ],
        )
        .expect("replay should succeed");

        let edge_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM symbol_edges_v2", [], |row| row.get(0))
            .expect("edge count should query");
        assert_eq!(edge_count, 1);
    }

    #[test]
    fn symbol_resolution_and_reuse_helpers_cover_fallback_paths() {
        let mut connection = bootstrap_connection();
        insert_symbol_row(
            &connection,
            11,
            "src/lib.rs",
            "solo",
            "function",
            "rust",
            Some("rust:src/lib.rs::solo"),
        );

        let tx = connection.transaction().expect("transaction should start");
        let resolved = resolve_symbol_id_by_symbol(&tx, &symbol_key("solo", None, None, None))
            .expect("symbol resolution should succeed");
        assert_eq!(resolved, Some(11));
        tx.commit().expect("transaction should commit");

        let existing = existing_symbol_ids(&connection, "src/lib.rs")
            .expect("existing symbol ids should load");
        assert_eq!(
            existing
                .get(&(String::from("solo"), String::from("function")))
                .expect("symbol key should exist"),
            &vec![11]
        );
        assert_eq!(
            next_symbol_id_start(&connection).expect("next id should load"),
            12
        );

        let mut reusable = HashMap::from([(
            (String::from("solo"), String::from("function")),
            Vec::<i64>::new(),
        )]);
        assert_eq!(
            take_reusable_symbol_id(&mut reusable, "solo", "function"),
            None,
            "empty reusable-id buckets should return None"
        );
    }

    #[test]
    fn symbol_id_helpers_surface_missing_schema_errors() {
        let connection = Connection::open_in_memory().expect("in-memory db should open");
        assert!(
            existing_symbol_ids(&connection, "src/lib.rs").is_err(),
            "existing-symbol-id helper should fail when symbols_v2 is missing"
        );
        assert!(
            next_symbol_id_start(&connection).is_err(),
            "next-symbol-id helper should fail when symbols_v2 is missing"
        );
    }
}
