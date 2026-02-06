use std::collections::HashSet;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct QueryMatch {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub symbol: String,
    pub why_matched: String,
    pub confidence: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactMatch {
    pub symbol: String,
    pub kind: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub distance: u32,
    pub relationship: String,
    pub confidence: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextMatch {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub symbol: String,
    pub kind: String,
    pub why_included: String,
    pub confidence: String,
    pub score: f64,
}

pub fn find_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let ast_definitions = ast_definition_matches(&connection, symbol)?;
    if !ast_definitions.is_empty() {
        return Ok(ast_definitions);
    }

    ranked_text_matches(&connection, symbol)
}

pub fn refs_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let ast_references = ast_reference_matches(&connection, symbol)?;
    if !ast_references.is_empty() {
        return Ok(ast_references);
    }

    ranked_text_matches(&connection, symbol)
}

pub fn impact_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<ImpactMatch>> {
    let connection = Connection::open(db_path)?;
    let mut target_ids_statement = connection.prepare(
        "SELECT symbol_id
         FROM symbols_v2
         WHERE symbol = ?1
         ORDER BY file_path ASC, start_line ASC, start_column ASC",
    )?;
    let target_ids_rows =
        target_ids_statement.query_map(params![symbol], |row| row.get::<_, i64>(0))?;

    let mut target_ids = Vec::new();
    for row in target_ids_rows {
        target_ids.push(row?);
    }

    let mut results = Vec::new();
    let mut seen = HashSet::new();

    for target_id in target_ids {
        let mut incoming_statement = connection.prepare(
            "SELECT fs.file_path, fs.start_line, fs.start_column, fs.symbol, fs.kind, e.edge_kind, e.confidence
             FROM symbol_edges_v2 e
             JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
             WHERE e.to_symbol_id = ?1
             ORDER BY fs.file_path ASC, fs.start_line ASC, fs.start_column ASC, fs.symbol ASC",
        )?;
        let incoming_rows = incoming_statement.query_map(params![target_id], |row| {
            let edge_kind: String = row.get(5)?;
            let relationship = match edge_kind.as_str() {
                "calls" => "called_by".to_string(),
                "contains" => "contained_by".to_string(),
                "imports" => "imported_by".to_string(),
                "implements" => "implemented_by".to_string(),
                _ => edge_kind,
            };
            Ok(ImpactMatch {
                file_path: row.get(0)?,
                line: row.get::<_, i64>(1)? as u32,
                column: row.get::<_, i64>(2)? as u32,
                symbol: row.get(3)?,
                kind: row.get(4)?,
                distance: 1,
                relationship,
                confidence: "graph_likely".to_string(),
                score: row.get(6)?,
            })
        })?;
        for row in incoming_rows {
            let item = row?;
            let key = format!(
                "{}:{}:{}:{}:{}",
                item.file_path, item.line, item.column, item.symbol, item.relationship
            );
            if seen.insert(key) {
                results.push(item);
            }
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.relationship.cmp(&right.relationship))
    });
    Ok(results)
}

pub fn context_matches(
    db_path: &Path,
    task: &str,
    budget: usize,
) -> anyhow::Result<Vec<ContextMatch>> {
    let connection = Connection::open(db_path)?;
    let keywords = extract_keywords(task);

    let mut matches = Vec::new();
    let mut seen = HashSet::new();

    for keyword in keywords {
        let mut exact_statement = connection.prepare(
            "SELECT symbol_id, file_path, symbol, kind, start_line, end_line
             FROM symbols_v2
             WHERE lower(symbol) = lower(?1)
             ORDER BY file_path ASC, start_line ASC, start_column ASC",
        )?;
        let exact_rows = exact_statement.query_map(params![keyword], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)? as u32,
                row.get::<_, i64>(5)? as u32,
            ))
        })?;

        for row in exact_rows {
            let (symbol_id, file_path, symbol, kind, start_line, end_line) = row?;
            let key = format!("{file_path}:{start_line}:{symbol}:direct");
            if seen.insert(key) {
                matches.push(ContextMatch {
                    file_path: file_path.clone(),
                    start_line,
                    end_line,
                    symbol: symbol.clone(),
                    kind: kind.clone(),
                    why_included: format!("direct definition match for task keyword '{keyword}'"),
                    confidence: "context_high".to_string(),
                    score: 0.95,
                });
            }

            let mut neighbor_statement = connection.prepare(
                "SELECT n.file_path, n.symbol, n.kind, n.start_line, n.end_line
                 FROM symbol_edges_v2 e
                 JOIN symbols_v2 n ON n.symbol_id = e.to_symbol_id
                 WHERE e.from_symbol_id = ?1
                 ORDER BY n.file_path ASC, n.start_line ASC, n.start_column ASC",
            )?;
            let neighbor_rows = neighbor_statement.query_map(params![symbol_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? as u32,
                    row.get::<_, i64>(4)? as u32,
                ))
            })?;

            for neighbor in neighbor_rows {
                let (n_file, n_symbol, n_kind, n_start, n_end) = neighbor?;
                let neighbor_key = format!("{n_file}:{n_start}:{n_symbol}:neighbor");
                if seen.insert(neighbor_key) {
                    matches.push(ContextMatch {
                        file_path: n_file,
                        start_line: n_start,
                        end_line: n_end,
                        symbol: n_symbol,
                        kind: n_kind,
                        why_included: format!("graph neighbor of '{symbol}'"),
                        confidence: "context_medium".to_string(),
                        score: 0.7,
                    });
                }
            }
        }
    }

    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.start_line.cmp(&right.start_line))
            .then(left.symbol.cmp(&right.symbol))
    });

    let max_results = std::cmp::max(1, budget / 200);
    matches.truncate(max_results);
    Ok(matches)
}

fn ast_definition_matches(
    connection: &Connection,
    symbol: &str,
) -> anyhow::Result<Vec<QueryMatch>> {
    let mut statement = connection.prepare(
        "SELECT file_path, line, column, symbol
         FROM ast_definitions
         WHERE symbol = ?1
         ORDER BY file_path ASC, line ASC, column ASC",
    )?;
    let rows = statement.query_map(params![symbol], |row| {
        Ok(QueryMatch {
            file_path: row.get(0)?,
            line: row.get::<_, i64>(1)? as u32,
            column: row.get::<_, i64>(2)? as u32,
            symbol: row.get(3)?,
            why_matched: "ast_definition".to_string(),
            confidence: "ast_exact".to_string(),
            score: 1.0,
        })
    })?;

    collect_rows(rows)
}

fn ast_reference_matches(connection: &Connection, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let mut statement = connection.prepare(
        "SELECT file_path, line, column, symbol
         FROM ast_references
         WHERE symbol = ?1
         ORDER BY file_path ASC, line ASC, column ASC",
    )?;
    let rows = statement.query_map(params![symbol], |row| {
        Ok(QueryMatch {
            file_path: row.get(0)?,
            line: row.get::<_, i64>(1)? as u32,
            column: row.get::<_, i64>(2)? as u32,
            symbol: row.get(3)?,
            why_matched: "ast_reference".to_string(),
            confidence: "ast_likely".to_string(),
            score: 0.95,
        })
    })?;

    collect_rows(rows)
}

fn ranked_text_matches(connection: &Connection, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let mut matches = text_exact_matches(connection, symbol)?;
    matches.extend(text_substring_matches(connection, symbol)?);
    Ok(matches)
}

fn text_exact_matches(connection: &Connection, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let mut statement = connection.prepare(
        "SELECT file_path, line, column, symbol
         FROM text_occurrences
         WHERE symbol = ?1
         ORDER BY file_path ASC, line ASC, column ASC, symbol ASC",
    )?;
    let rows = statement.query_map(params![symbol], |row| {
        Ok(QueryMatch {
            file_path: row.get(0)?,
            line: row.get::<_, i64>(1)? as u32,
            column: row.get::<_, i64>(2)? as u32,
            symbol: row.get(3)?,
            why_matched: "exact_symbol_name".to_string(),
            confidence: "text_fallback".to_string(),
            score: 0.8,
        })
    })?;

    collect_rows(rows)
}

fn text_substring_matches(
    connection: &Connection,
    symbol: &str,
) -> anyhow::Result<Vec<QueryMatch>> {
    let pattern = format!("%{symbol}%");
    let mut statement = connection.prepare(
        "SELECT file_path, line, column, symbol
         FROM text_occurrences
         WHERE symbol LIKE ?1 AND symbol <> ?2
         ORDER BY file_path ASC, line ASC, column ASC, symbol ASC",
    )?;
    let rows = statement.query_map(params![pattern, symbol], |row| {
        Ok(QueryMatch {
            file_path: row.get(0)?,
            line: row.get::<_, i64>(1)? as u32,
            column: row.get::<_, i64>(2)? as u32,
            symbol: row.get(3)?,
            why_matched: "text_substring_match".to_string(),
            confidence: "text_fallback".to_string(),
            score: 0.4,
        })
    })?;

    collect_rows(rows)
}

fn collect_rows<F>(rows: rusqlite::MappedRows<'_, F>) -> anyhow::Result<Vec<QueryMatch>>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<QueryMatch>,
{
    let mut matches = Vec::new();
    for row in rows {
        matches.push(row?);
    }
    Ok(matches)
}

fn extract_keywords(task: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let mut seen = HashSet::new();

    for token in task
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
    {
        let lowered = token.to_ascii_lowercase();
        if lowered.len() < 3 {
            continue;
        }
        if seen.insert(lowered.clone()) {
            keywords.push(lowered);
        }
    }

    keywords
}
