use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Clone, Serialize)]
pub struct TestTarget {
    pub target: String,
    pub target_kind: String,
    pub why_included: String,
    pub confidence: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationStep {
    pub step: String,
    pub scope: String,
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

pub fn tests_for_symbol(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<TestTarget>> {
    let connection = Connection::open(db_path)?;
    let mut targets = Vec::new();
    for (target, hit_count) in test_targets_for_symbol(&connection, symbol)? {
        targets.push(TestTarget {
            target: target.clone(),
            target_kind: "integration_test_file".to_string(),
            why_included: format!("direct symbol match for '{symbol}' in test file"),
            confidence: if hit_count > 1 {
                "graph_likely".to_string()
            } else {
                "context_medium".to_string()
            },
            score: if hit_count > 1 { 0.9 } else { 0.75 },
        });
    }

    Ok(targets)
}

pub fn verify_plan_for_changed_files(
    db_path: &Path,
    changed_files: &[String],
) -> anyhow::Result<Vec<VerificationStep>> {
    let connection = Connection::open(db_path)?;

    let mut steps_by_command: HashMap<String, VerificationStep> = HashMap::new();

    for changed_file in changed_files {
        if let Some(command) = test_command_for_target(changed_file) {
            upsert_verification_step(
                &mut steps_by_command,
                VerificationStep {
                    step: command,
                    scope: "targeted".to_string(),
                    why_included: format!("changed file '{changed_file}' is itself a test target"),
                    confidence: "context_high".to_string(),
                    score: 0.95,
                },
            );
        }

        let mut symbols_statement = connection.prepare(
            "SELECT DISTINCT symbol
             FROM symbols_v2
             WHERE file_path = ?1
             ORDER BY symbol ASC",
        )?;
        let symbol_rows =
            symbols_statement.query_map(params![changed_file], |row| row.get::<_, String>(0))?;

        for symbol_row in symbol_rows {
            let symbol = symbol_row?;
            for (target, hit_count) in test_targets_for_symbol(&connection, &symbol)? {
                if let Some(command) = test_command_for_target(&target) {
                    let (confidence, score) = if hit_count > 1 {
                        ("graph_likely", 0.9)
                    } else {
                        ("context_medium", 0.8)
                    };
                    upsert_verification_step(
                        &mut steps_by_command,
                        VerificationStep {
                            step: command,
                            scope: "targeted".to_string(),
                            why_included: format!(
                                "targeted test references changed symbol '{symbol}'"
                            ),
                            confidence: confidence.to_string(),
                            score,
                        },
                    );
                }
            }
        }
    }

    upsert_verification_step(
        &mut steps_by_command,
        VerificationStep {
            step: "cargo test".to_string(),
            scope: "full_suite".to_string(),
            why_included: "required safety gate after refactor".to_string(),
            confidence: "context_high".to_string(),
            score: 1.0,
        },
    );

    let mut steps = steps_by_command.into_values().collect::<Vec<_>>();

    steps.sort_by(|left, right| {
        verification_scope_rank(&left.scope)
            .cmp(&verification_scope_rank(&right.scope))
            .then(left.step.cmp(&right.step))
            .then(left.why_included.cmp(&right.why_included))
    });
    Ok(steps)
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

fn test_targets_for_symbol(
    connection: &Connection,
    symbol: &str,
) -> anyhow::Result<Vec<(String, i64)>> {
    let mut statement = connection.prepare(
        "SELECT file_path, COUNT(*) AS hit_count
         FROM text_occurrences
         WHERE symbol = ?1
           AND (
               file_path LIKE 'tests/%'
               OR file_path LIKE '%/tests/%'
               OR file_path LIKE '%_test.rs'
               OR file_path LIKE '%test.rs'
           )
         GROUP BY file_path
         ORDER BY hit_count DESC, file_path ASC",
    )?;

    let rows = statement.query_map(params![symbol], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    let mut targets = Vec::new();
    for row in rows {
        targets.push(row?);
    }
    Ok(targets)
}

fn test_command_for_target(target: &str) -> Option<String> {
    let file_path = Path::new(target);
    let mut components = file_path.components();
    if components.next()?.as_os_str() != "tests" {
        return None;
    }
    let test_file = Path::new(components.next()?.as_os_str());
    if components.next().is_some() {
        return None;
    }

    let stem = test_file.file_stem()?.to_str()?;
    Some(format!("cargo test --test {stem}"))
}

fn verification_scope_rank(scope: &str) -> u8 {
    match scope {
        "targeted" => 0,
        "full_suite" => 1,
        _ => 2,
    }
}

fn upsert_verification_step(
    steps_by_command: &mut HashMap<String, VerificationStep>,
    candidate: VerificationStep,
) {
    let key = candidate.step.clone();
    match steps_by_command.get_mut(&key) {
        Some(existing) => {
            if candidate.score > existing.score
                || (candidate.score == existing.score
                    && confidence_rank(&candidate.confidence)
                        > confidence_rank(&existing.confidence))
                || (candidate.score == existing.score
                    && confidence_rank(&candidate.confidence)
                        == confidence_rank(&existing.confidence)
                    && candidate.why_included < existing.why_included)
            {
                *existing = candidate;
            }
        }
        None => {
            steps_by_command.insert(key, candidate);
        }
    }
}

fn confidence_rank(confidence: &str) -> u8 {
    match confidence {
        "graph_likely" => 3,
        "context_high" => 2,
        "context_medium" => 1,
        _ => 0,
    }
}
