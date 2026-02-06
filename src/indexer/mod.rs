use std::collections::{HashMap, HashSet};
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

pub mod files;
pub mod rust_ast;
pub mod text;

#[derive(Debug)]
pub struct IndexSummary {
    pub indexed_files: usize,
    pub skipped_files: usize,
}

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
        let (ast_definitions, ast_references) = if file.relative_path.ends_with(".rs") {
            text_content
                .map(rust_ast::extract_rust_items)
                .transpose()?
                .unwrap_or_default()
        } else {
            (Vec::new(), Vec::new())
        };
        let relation_hints = if file.relative_path.ends_with(".rs") {
            text_content.map(extract_relation_hints).unwrap_or_default()
        } else {
            Vec::new()
        };
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

        let mut pending_edges: Vec<(String, String, String, f64)> = relation_hints;

        for definition in ast_definitions {
            let symbol = definition.symbol;
            let kind = definition.kind;
            let container = definition.container;
            let start_line = i64::from(definition.line);
            let start_column = i64::from(definition.column);
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
                    symbol_id, file_path, symbol, kind, container, start_line, start_column, end_line, end_column, signature
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    symbol_id,
                    &file.relative_path,
                    &symbol,
                    &kind,
                    container.as_deref(),
                    start_line,
                    start_column,
                    end_line,
                    end_column,
                    signature.as_deref()
                ],
            )?;

            if let Some(container_symbol) = container.as_deref() {
                pending_edges.push((
                    container_symbol.to_string(),
                    symbol.clone(),
                    "contains".to_string(),
                    1.0,
                ));
            }
        }

        for reference in ast_references {
            let caller = reference.caller;
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

            if let Some(caller_symbol) = caller {
                pending_edges.push((caller_symbol, symbol, "calls".to_string(), 0.95));
            }
        }

        for (from_symbol, to_symbol, edge_kind, confidence) in pending_edges {
            let Some(from_symbol_id) = resolve_symbol_id_in_tx(&tx, &from_symbol)? else {
                continue;
            };
            let Some(to_symbol_id) = resolve_symbol_id_in_tx(&tx, &to_symbol)? else {
                continue;
            };

            tx.execute(
                "INSERT INTO symbol_edges_v2(from_symbol_id, to_symbol_id, edge_kind, confidence)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(from_symbol_id, to_symbol_id, edge_kind)
                 DO UPDATE SET confidence = excluded.confidence",
                params![from_symbol_id, to_symbol_id, edge_kind, confidence],
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

    Ok(IndexSummary {
        indexed_files,
        skipped_files,
    })
}

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

fn resolve_symbol_id_in_tx(
    tx: &rusqlite::Transaction<'_>,
    symbol: &str,
) -> anyhow::Result<Option<i64>> {
    let symbol_id = tx
        .query_row(
            "SELECT symbol_id
             FROM symbols_v2
             WHERE symbol = ?1
             ORDER BY file_path ASC, start_line ASC, start_column ASC
             LIMIT 1",
            [symbol],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;
    Ok(symbol_id)
}

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

fn next_symbol_id_start(connection: &Connection) -> anyhow::Result<i64> {
    let max_id: i64 = connection.query_row(
        "SELECT COALESCE(MAX(symbol_id), 0) FROM symbols_v2",
        [],
        |row| row.get(0),
    )?;
    Ok(max_id + 1)
}

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

fn extract_relation_hints(content: &str) -> Vec<(String, String, String, f64)> {
    let mut edges = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("use ") {
            let statement = rest.trim().trim_end_matches(';').trim();
            if let Some((left, right)) = statement.split_once(" as ") {
                let Some(target_symbol) = last_rust_identifier(left) else {
                    continue;
                };
                let Some(alias_symbol) = last_rust_identifier(right) else {
                    continue;
                };
                if alias_symbol != target_symbol {
                    edges.push((alias_symbol, target_symbol, "imports".to_string(), 0.9));
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("impl ") {
            if let Some((trait_part, type_part)) = rest.split_once(" for ") {
                let Some(trait_symbol) = last_rust_identifier(trait_part) else {
                    continue;
                };
                let Some(type_symbol) = last_rust_identifier(type_part) else {
                    continue;
                };
                edges.push((type_symbol, trait_symbol, "implements".to_string(), 0.95));
            }
        }
    }

    edges
}

fn last_rust_identifier(segment: &str) -> Option<String> {
    segment
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
        .last()
        .map(str::to_string)
}
