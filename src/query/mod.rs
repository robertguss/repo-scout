use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fs;
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryScope {
    pub code_only: bool,
    pub exclude_tests: bool,
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

#[derive(Debug, Clone, Serialize)]
pub struct ExplainInboundSummary {
    pub called_by: u32,
    pub imported_by: u32,
    pub implemented_by: u32,
    pub contained_by: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplainOutboundSummary {
    pub calls: u32,
    pub imports: u32,
    pub implements: u32,
    pub contains: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplainMatch {
    pub symbol: String,
    pub qualified_symbol: String,
    pub kind: String,
    pub language: String,
    pub file_path: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub inbound: ExplainInboundSummary,
    pub outbound: ExplainOutboundSummary,
    pub why_included: String,
    pub confidence: String,
    pub provenance: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "result_kind")]
pub enum DiffImpactMatch {
    #[serde(rename = "impacted_symbol")]
    ImpactedSymbol {
        symbol: String,
        qualified_symbol: String,
        kind: String,
        language: String,
        file_path: String,
        line: u32,
        column: u32,
        distance: u32,
        relationship: String,
        why_included: String,
        confidence: String,
        provenance: String,
        score: f64,
    },
    #[serde(rename = "test_target")]
    TestTarget {
        target: String,
        target_kind: String,
        language: String,
        why_included: String,
        confidence: String,
        provenance: String,
        score: f64,
    },
}

#[derive(Debug, Clone)]
pub struct ChangedLineRange {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
}

#[derive(Debug, Clone)]
pub struct DiffImpactOptions {
    pub max_distance: u32,
    pub include_tests: bool,
    pub include_imports: bool,
    pub changed_lines: Vec<ChangedLineRange>,
    pub changed_symbols: Vec<String>,
    pub exclude_changed: bool,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyPlanOptions {
    pub max_targeted: Option<usize>,
    pub changed_lines: Vec<ChangedLineRange>,
    pub changed_symbols: Vec<String>,
}

pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: usize = 8;

pub fn diff_impact_for_changed_files(
    db_path: &Path,
    changed_files: &[String],
    options: &DiffImpactOptions,
) -> anyhow::Result<Vec<DiffImpactMatch>> {
    let connection = Connection::open(db_path)?;
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let mut changed_symbol_ids = Vec::new();
    let changed_lines_by_file = changed_lines_by_file(&options.changed_lines);
    let changed_symbol_filter = options
        .changed_symbols
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

    for changed_file in changed_files {
        let mut statement = connection.prepare(
            "SELECT symbol_id, symbol, kind, file_path, start_line, start_column, end_line, language, qualified_symbol
             FROM symbols_v2
             WHERE file_path = ?1
               AND (?2 OR kind <> 'import')
             ORDER BY start_line ASC, start_column ASC, symbol ASC",
        )?;
        let rows = statement.query_map(params![changed_file, options.include_imports], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)? as u32,
                row.get::<_, i64>(5)? as u32,
                row.get::<_, i64>(6)? as u32,
                row.get::<_, String>(7)?,
                row.get::<_, Option<String>>(8)?,
            ))
        })?;

        for row in rows {
            let (
                symbol_id,
                symbol,
                kind,
                file_path,
                line,
                column,
                end_line,
                language,
                qualified_symbol,
            ) = row?;
            if let Some(ranges) = changed_lines_by_file.get(changed_file)
                && !ranges.iter().any(|range| {
                    line_range_overlaps(line, end_line, range.start_line, range.end_line)
                })
            {
                continue;
            }
            if !changed_symbol_filter.is_empty() && !changed_symbol_filter.contains(&symbol) {
                continue;
            }
            let language = normalized_language(&language, &file_path).to_string();
            let qualified_symbol =
                qualified_symbol.unwrap_or_else(|| format!("{language}:{file_path}::{symbol}"));
            let key = format!("{file_path}:{line}:{column}:{qualified_symbol}:changed_symbol:0");
            if !seen.insert(key) {
                continue;
            }

            changed_symbol_ids.push((symbol_id, symbol.clone()));
            results.push(DiffImpactMatch::ImpactedSymbol {
                symbol,
                qualified_symbol,
                kind,
                language,
                file_path,
                line,
                column,
                distance: 0,
                relationship: "changed_symbol".to_string(),
                why_included: "symbol defined in changed file".to_string(),
                confidence: "graph_exact".to_string(),
                provenance: "ast_definition".to_string(),
                score: 1.0,
            });
        }
    }

    let changed_symbol_id_set = changed_symbol_ids
        .iter()
        .map(|(symbol_id, _)| *symbol_id)
        .collect::<HashSet<_>>();

    if options.max_distance >= 1 {
        let traversal_limit = options.max_distance;
        for (changed_symbol_id, changed_symbol) in changed_symbol_ids {
            let mut frontier = VecDeque::new();
            let mut min_distance_by_symbol = HashMap::new();
            frontier.push_back((changed_symbol_id, 0_u32));
            min_distance_by_symbol.insert(changed_symbol_id, 0_u32);

            while let Some((to_symbol_id, distance)) = frontier.pop_front() {
                if distance >= traversal_limit {
                    continue;
                }
                let next_distance = distance + 1;

                let mut incoming_statement = connection.prepare(
                    "SELECT fs.symbol_id, fs.symbol, fs.kind, fs.file_path, fs.start_line, fs.start_column, fs.language, fs.qualified_symbol, e.edge_kind, e.confidence, e.provenance
                     FROM symbol_edges_v2 e
                     JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
                     WHERE e.to_symbol_id = ?1
                     ORDER BY fs.file_path ASC, fs.start_line ASC, fs.start_column ASC, fs.symbol ASC",
                )?;
                let incoming_rows = incoming_statement.query_map(params![to_symbol_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)? as u32,
                        row.get::<_, i64>(5)? as u32,
                        row.get::<_, String>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, f64>(9)?,
                        row.get::<_, String>(10)?,
                    ))
                })?;

                for incoming in incoming_rows {
                    let (
                        from_symbol_id,
                        symbol,
                        kind,
                        file_path,
                        line,
                        column,
                        language,
                        qualified_symbol,
                        edge_kind,
                        score,
                        provenance,
                    ) = incoming?;
                    if changed_symbol_id_set.contains(&from_symbol_id) {
                        continue;
                    }
                    if min_distance_by_symbol
                        .get(&from_symbol_id)
                        .is_some_and(|known| *known < next_distance)
                    {
                        continue;
                    }

                    let known_distance = min_distance_by_symbol.get(&from_symbol_id).copied();
                    let should_expand_frontier = known_distance != Some(next_distance);
                    if known_distance.is_none_or(|known| next_distance < known) {
                        min_distance_by_symbol.insert(from_symbol_id, next_distance);
                    }

                    let language = normalized_language(&language, &file_path).to_string();
                    let qualified_symbol = qualified_symbol
                        .unwrap_or_else(|| format!("{language}:{file_path}::{symbol}"));
                    let relationship = edge_kind_relationship(&edge_kind);
                    let provenance = normalized_provenance(&provenance, &edge_kind);
                    let key = format!(
                        "{file_path}:{line}:{column}:{qualified_symbol}:{relationship}:distance{next_distance}"
                    );
                    if !seen.insert(key) {
                        continue;
                    }

                    results.push(DiffImpactMatch::ImpactedSymbol {
                        symbol: symbol.clone(),
                        qualified_symbol,
                        kind,
                        language,
                        file_path,
                        line,
                        column,
                        distance: next_distance,
                        relationship: relationship.to_string(),
                        why_included: format!(
                            "direct {relationship} neighbor of changed symbol '{changed_symbol}'"
                        ),
                        confidence: "graph_likely".to_string(),
                        provenance,
                        score,
                    });
                    if should_expand_frontier {
                        frontier.push_back((from_symbol_id, next_distance));
                    }
                }
            }
        }
    }

    if options.include_tests {
        let mut impacted_symbols = results
            .iter()
            .filter_map(|item| match item {
                DiffImpactMatch::ImpactedSymbol { symbol, .. } => Some(symbol.clone()),
                DiffImpactMatch::TestTarget { .. } => None,
            })
            .collect::<Vec<_>>();
        impacted_symbols.sort();
        impacted_symbols.dedup();

        let mut selected_test_targets: BTreeMap<String, DiffImpactMatch> = BTreeMap::new();
        for symbol in impacted_symbols {
            for (target, hit_count) in test_targets_for_symbol(&connection, &symbol)? {
                let (confidence, score) = if hit_count > 1 {
                    ("graph_likely", 0.86)
                } else {
                    ("context_medium", 0.72)
                };

                let key = format!("integration_test_file:{target}");
                let should_replace = match selected_test_targets.get(&key) {
                    Some(DiffImpactMatch::TestTarget {
                        score: existing_score,
                        ..
                    }) => score > *existing_score,
                    _ => true,
                };
                if !should_replace {
                    continue;
                }

                selected_test_targets.insert(
                    key,
                    DiffImpactMatch::TestTarget {
                        target: target.clone(),
                        target_kind: "integration_test_file".to_string(),
                        language: language_for_file_path(&target).to_string(),
                        why_included: format!("references impacted symbol '{symbol}'"),
                        confidence: confidence.to_string(),
                        provenance: "text_fallback".to_string(),
                        score,
                    },
                );
            }
        }

        results.extend(selected_test_targets.into_values());
    }

    if options.exclude_changed {
        results.retain(|item| {
            !matches!(
                item,
                DiffImpactMatch::ImpactedSymbol { relationship, .. } if relationship == "changed_symbol"
            )
        });
    }

    results.sort_by(diff_impact_sort_key);
    if let Some(max_results) = options.max_results {
        results.truncate(max_results);
    }
    Ok(results)
}

fn changed_lines_by_file(
    changed_lines: &[ChangedLineRange],
) -> HashMap<String, Vec<ChangedLineRange>> {
    let mut ranges_by_file = HashMap::new();
    for range in changed_lines {
        ranges_by_file
            .entry(range.file_path.clone())
            .or_insert_with(Vec::new)
            .push(range.clone());
    }
    ranges_by_file
}

fn line_range_overlaps(
    symbol_start_line: u32,
    symbol_end_line: u32,
    changed_start_line: u32,
    changed_end_line: u32,
) -> bool {
    symbol_start_line <= changed_end_line && symbol_end_line >= changed_start_line
}

pub fn explain_symbol(
    db_path: &Path,
    symbol: &str,
    include_snippets: bool,
) -> anyhow::Result<Vec<ExplainMatch>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "SELECT symbol_id, symbol, kind, file_path, start_line, start_column, end_line, end_column, signature, language, qualified_symbol
         FROM symbols_v2
         WHERE symbol = ?1
         ORDER BY file_path ASC, start_line ASC, start_column ASC, kind ASC",
    )?;
    let rows = statement.query_map(params![symbol], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)? as u32,
            row.get::<_, i64>(5)? as u32,
            row.get::<_, i64>(6)? as u32,
            row.get::<_, i64>(7)? as u32,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, String>(9)?,
            row.get::<_, Option<String>>(10)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        let (
            symbol_id,
            symbol,
            kind,
            file_path,
            start_line,
            start_column,
            end_line,
            end_column,
            signature,
            language,
            qualified_symbol,
        ) = row?;
        let language = normalized_language(&language, &file_path).to_string();
        let qualified_symbol =
            qualified_symbol.unwrap_or_else(|| format!("{language}:{file_path}::{symbol}"));
        let snippet = include_snippets
            .then(|| extract_symbol_snippet(db_path, &file_path, start_line, end_line))
            .flatten();
        let (inbound, outbound) = relationship_summaries_for_symbol_id(&connection, symbol_id)?;
        results.push(ExplainMatch {
            symbol,
            qualified_symbol,
            kind,
            language,
            file_path,
            start_line,
            start_column,
            end_line,
            end_column,
            signature,
            inbound,
            outbound,
            why_included: "exact symbol definition match".to_string(),
            confidence: "graph_exact".to_string(),
            provenance: "ast_definition".to_string(),
            score: 1.0,
            snippet,
        });
    }

    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.start_line.cmp(&right.start_line))
            .then(left.start_column.cmp(&right.start_column))
            .then(left.qualified_symbol.cmp(&right.qualified_symbol))
    });
    Ok(results)
}

fn relationship_summaries_for_symbol_id(
    connection: &Connection,
    symbol_id: i64,
) -> anyhow::Result<(ExplainInboundSummary, ExplainOutboundSummary)> {
    let mut inbound = ExplainInboundSummary {
        called_by: 0,
        imported_by: 0,
        implemented_by: 0,
        contained_by: 0,
    };
    let mut outbound = ExplainOutboundSummary {
        calls: 0,
        imports: 0,
        implements: 0,
        contains: 0,
    };

    let mut inbound_statement = connection.prepare(
        "SELECT edge_kind, COUNT(*)
         FROM symbol_edges_v2
         WHERE to_symbol_id = ?1
         GROUP BY edge_kind",
    )?;
    let inbound_rows = inbound_statement.query_map(params![symbol_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
    })?;
    for row in inbound_rows {
        let (edge_kind, count) = row?;
        match edge_kind.as_str() {
            "calls" => inbound.called_by = count,
            "imports" => inbound.imported_by = count,
            "implements" => inbound.implemented_by = count,
            "contains" => inbound.contained_by = count,
            _ => {}
        }
    }

    let mut outbound_statement = connection.prepare(
        "SELECT edge_kind, COUNT(*)
         FROM symbol_edges_v2
         WHERE from_symbol_id = ?1
         GROUP BY edge_kind",
    )?;
    let outbound_rows = outbound_statement.query_map(params![symbol_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
    })?;
    for row in outbound_rows {
        let (edge_kind, count) = row?;
        match edge_kind.as_str() {
            "calls" => outbound.calls = count,
            "imports" => outbound.imports = count,
            "implements" => outbound.implements = count,
            "contains" => outbound.contains = count,
            _ => {}
        }
    }

    Ok((inbound, outbound))
}

fn diff_impact_sort_key(left: &DiffImpactMatch, right: &DiffImpactMatch) -> std::cmp::Ordering {
    diff_impact_score(right)
        .partial_cmp(&diff_impact_score(left))
        .unwrap_or(std::cmp::Ordering::Equal)
        .then(diff_impact_kind_rank(left).cmp(&diff_impact_kind_rank(right)))
        .then_with(|| match (left, right) {
            (
                DiffImpactMatch::ImpactedSymbol {
                    file_path: lf,
                    line: ll,
                    column: lc,
                    qualified_symbol: lq,
                    ..
                },
                DiffImpactMatch::ImpactedSymbol {
                    file_path: rf,
                    line: rl,
                    column: rc,
                    qualified_symbol: rq,
                    ..
                },
            ) => lf
                .cmp(rf)
                .then(ll.cmp(rl))
                .then(lc.cmp(rc))
                .then(lq.cmp(rq)),
            (
                DiffImpactMatch::TestTarget {
                    target_kind: lk,
                    target: lt,
                    ..
                },
                DiffImpactMatch::TestTarget {
                    target_kind: rk,
                    target: rt,
                    ..
                },
            ) => lk.cmp(rk).then(lt.cmp(rt)),
            _ => std::cmp::Ordering::Equal,
        })
}

fn diff_impact_score(item: &DiffImpactMatch) -> f64 {
    match item {
        DiffImpactMatch::ImpactedSymbol { score, .. }
        | DiffImpactMatch::TestTarget { score, .. } => *score,
    }
}

fn diff_impact_kind_rank(item: &DiffImpactMatch) -> u8 {
    match item {
        DiffImpactMatch::ImpactedSymbol { .. } => 0,
        DiffImpactMatch::TestTarget { .. } => 1,
    }
}

fn edge_kind_relationship(edge_kind: &str) -> &'static str {
    match edge_kind {
        "calls" => "called_by",
        "contains" => "contained_by",
        "imports" => "imported_by",
        "implements" => "implemented_by",
        _ => "called_by",
    }
}

fn normalized_provenance(provenance: &str, edge_kind: &str) -> String {
    match provenance {
        "ast_definition" | "ast_reference" | "import_resolution" | "call_resolution"
        | "text_fallback" => provenance.to_string(),
        _ => match edge_kind {
            "calls" => "call_resolution".to_string(),
            "imports" => "import_resolution".to_string(),
            "contains" => "ast_definition".to_string(),
            "implements" => "ast_reference".to_string(),
            _ => "ast_reference".to_string(),
        },
    }
}

fn language_for_file_path(file_path: &str) -> &'static str {
    if file_path.ends_with(".rs") {
        "rust"
    } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        "typescript"
    } else if file_path.ends_with(".py") {
        "python"
    } else {
        "unknown"
    }
}

fn normalized_language(language: &str, file_path: &str) -> &'static str {
    match language {
        "rust" => "rust",
        "typescript" => "typescript",
        "python" => "python",
        "unknown" => "unknown",
        _ => language_for_file_path(file_path),
    }
}

fn extract_symbol_snippet(
    db_path: &Path,
    file_path: &str,
    start_line: u32,
    end_line: u32,
) -> Option<String> {
    let repo_root = db_path.parent()?.parent()?;
    let absolute_path = repo_root.join(file_path);
    let source = fs::read_to_string(absolute_path).ok()?;
    let lines = source.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }

    let start_index = start_line.saturating_sub(1) as usize;
    if start_index >= lines.len() {
        return None;
    }

    let end_index = end_line.max(start_line).saturating_sub(1) as usize;
    let clamped_end = std::cmp::min(end_index, lines.len().saturating_sub(1));
    let snippet = lines[start_index..=clamped_end].join("\n");
    let trimmed = snippet.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

/// Finds code locations that match `symbol`, preferring exact AST definitions and falling back to text matches.
///
/// Searches the SQLite database at `db_path` for exact AST definition matches of `symbol`. If any AST definition matches are found those are returned; otherwise the function returns ranked text-based matches.
///
/// # Parameters
///
/// - `db_path`: Path to the SQLite database containing indexed symbols and occurrences.
/// - `symbol`: The symbol name to search for.
///
/// # Returns
///
/// A vector of `QueryMatch` entries representing locations where `symbol` appears, ordered by relevance.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// let matches = find_matches(Path::new("index.sqlite"), "my_symbol").unwrap();
/// // `matches` contains locations (file_path, line, column, ...) where `my_symbol` was found.
/// ```
#[allow(dead_code)]
pub fn find_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    find_matches_scoped(db_path, symbol, &QueryScope::default())
}

pub fn find_matches_scoped(
    db_path: &Path,
    symbol: &str,
    scope: &QueryScope,
) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let ast_definitions = ast_definition_matches(&connection, symbol)?;
    if !ast_definitions.is_empty() {
        return Ok(ast_definitions);
    }

    ranked_text_matches(&connection, symbol, scope)
}

/// Finds references to `symbol` in the database, preferring AST-derived reference matches.
///
/// If any AST references are present for the symbol those matches are returned. If no AST
/// references are found, a ranked set of text-based matches is returned instead.
///
/// # Returns
///
/// A `Vec<QueryMatch>` containing occurrences of the symbol. Each `QueryMatch` describes
/// a file location and why it was matched (e.g., `"ast_reference"`, `"exact_symbol_name"`,
/// `"text_substring_match"`), with associated confidence and score.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// let matches = refs_matches(Path::new("code_index.sqlite"), "my_function").unwrap();
/// // matches contains locations where `my_function` is referenced.
/// ```
#[allow(dead_code)]
pub fn refs_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    refs_matches_scoped(db_path, symbol, &QueryScope::default())
}

pub fn refs_matches_scoped(
    db_path: &Path,
    symbol: &str,
    scope: &QueryScope,
) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let ast_references = ast_reference_matches(&connection, symbol)?;
    if !ast_references.is_empty() {
        return Ok(ast_references);
    }

    ranked_text_matches(&connection, symbol, scope)
}

/// Finds symbols that directly impact the given symbol by querying the stored symbol graph.
///
/// The function returns incoming graph edges targeting `symbol`, producing `ImpactMatch` records
/// that describe the referring symbol, its location, the relationship (e.g. `called_by`,
/// `contained_by`), a fixed graph distance of 1, a confidence hint, and a score. Results are
/// deduplicated by (file_path, line, column, symbol, relationship) and ordered by score
/// descending, then by file_path, line, column, symbol, and relationship.
///
/// # Returns
///
/// A vector of `ImpactMatch` entries matching incoming graph edges for `symbol`, ordered by score
/// (highest first) and then by location and symbol.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// // `db_path` should point to a SQLite database prepared with the expected schema.
/// let matches = impact_matches(Path::new("code_index.sqlite"), "my_crate::MyType")
///     .expect("query failed");
/// // `matches` contains ImpactMatch entries referring to symbols that impact `my_crate::MyType`.
/// ```
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

/// Finds symbols relevant to a natural-language task by extracting keywords from the task,
/// matching exact symbol definitions, and including their graph neighbors.
///
/// The function returns a vector of ContextMatch ordered by score (highest first) and
/// then by file path, start line, and symbol. Results are truncated to at most
/// max(1, budget / 200) entries.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// // Assume a SQLite DB at "db.sqlite" with the expected schema.
/// let matches = query::context_matches(Path::new("db.sqlite"), "refactor authentication flow", 1000).unwrap();
/// assert!(!matches.is_empty());
/// ```
pub fn context_matches(
    db_path: &Path,
    task: &str,
    budget: usize,
) -> anyhow::Result<Vec<ContextMatch>> {
    context_matches_scoped(db_path, task, budget, &QueryScope::default())
}

pub fn context_matches_scoped(
    db_path: &Path,
    task: &str,
    budget: usize,
    scope: &QueryScope,
) -> anyhow::Result<Vec<ContextMatch>> {
    let connection = Connection::open(db_path)?;
    let keywords = extract_keywords(task);
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    let mut seen = HashSet::new();

    let mut symbols_statement = connection.prepare(
        "SELECT symbol_id, file_path, symbol, kind, start_line, end_line
         FROM symbols_v2
         ORDER BY file_path ASC, start_line ASC, start_column ASC, symbol ASC",
    )?;
    let symbol_rows = symbols_statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)? as u32,
            row.get::<_, i64>(5)? as u32,
        ))
    })?;

    for row in symbol_rows {
        let (symbol_id, file_path, symbol, kind, start_line, end_line) = row?;
        let symbol_tokens = symbol_keywords(&symbol);
        let matched_keywords = matched_task_keywords(&keywords, &symbol_tokens);
        if matched_keywords.is_empty() {
            continue;
        }
        let overlap_count = matched_keywords.len();
        let exact_symbol_match = keywords
            .iter()
            .any(|keyword| keyword == &symbol.to_ascii_lowercase());
        let direct_score =
            context_direct_score(overlap_count, exact_symbol_match, symbol_tokens.len());
        let key = format!("{file_path}:{start_line}:{symbol}:{kind}:direct");
        if seen.insert(key) {
            matches.push(ContextMatch {
                file_path: file_path.clone(),
                start_line,
                end_line,
                symbol: symbol.clone(),
                kind: kind.clone(),
                why_included: format!(
                    "direct definition token-overlap relevance for [{}]",
                    matched_keywords.join(", ")
                ),
                confidence: if overlap_count >= 2 || exact_symbol_match {
                    "context_high".to_string()
                } else {
                    "context_medium".to_string()
                },
                score: direct_score,
            });
        }

        let mut neighbor_statement = connection.prepare(
            "SELECT n.file_path, n.symbol, n.kind, n.start_line, n.end_line
             FROM symbol_edges_v2 e
             JOIN symbols_v2 n ON n.symbol_id = e.to_symbol_id
             WHERE e.from_symbol_id = ?1
             ORDER BY n.file_path ASC, n.start_line ASC, n.start_column ASC, n.symbol ASC",
        )?;
        let neighbor_rows = neighbor_statement.query_map(params![symbol_id], |neighbor_row| {
            Ok((
                neighbor_row.get::<_, String>(0)?,
                neighbor_row.get::<_, String>(1)?,
                neighbor_row.get::<_, String>(2)?,
                neighbor_row.get::<_, i64>(3)? as u32,
                neighbor_row.get::<_, i64>(4)? as u32,
            ))
        })?;

        for neighbor in neighbor_rows {
            let (n_file, n_symbol, n_kind, n_start, n_end) = neighbor?;
            let neighbor_key = format!("{n_file}:{n_start}:{n_symbol}:{n_kind}:neighbor");
            if seen.insert(neighbor_key) {
                matches.push(ContextMatch {
                    file_path: n_file,
                    start_line: n_start,
                    end_line: n_end,
                    symbol: n_symbol,
                    kind: n_kind,
                    why_included: format!(
                        "graph neighbor of '{symbol}' from token-overlap relevance [{}]",
                        matched_keywords.join(", ")
                    ),
                    confidence: "context_medium".to_string(),
                    score: (direct_score - 0.2).max(0.55),
                });
            }
        }
    }

    matches.retain(|item| {
        (!scope.code_only || is_code_file_path(&item.file_path))
            && (!scope.exclude_tests || !is_test_like_path(&item.file_path))
    });

    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.file_path.cmp(&right.file_path))
            .then(left.start_line.cmp(&right.start_line))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.kind.cmp(&right.kind))
            .then(left.end_line.cmp(&right.end_line))
            .then(left.why_included.cmp(&right.why_included))
    });

    let max_results = std::cmp::max(1, budget / 200);
    matches.truncate(max_results);
    Ok(matches)
}

/// Finds test files that reference `symbol` and returns them as prioritized test targets.
///
/// Each returned `TestTarget` describes a candidate test file (usually an integration test)
/// that contains direct occurrences of `symbol`, along with a short rationale (`why_included`),
/// a confidence string, and a numeric `score` used for ranking.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// // Query the database at "my_index.sqlite" for tests that mention "my_symbol".
/// let db = Path::new("my_index.sqlite");
/// let targets = tests_for_symbol(db, "my_symbol").unwrap();
/// for t in targets {
///     println!("{} -> {} (score={})", t.target, t.why_included, t.score);
/// }
/// ```
pub fn tests_for_symbol(
    db_path: &Path,
    symbol: &str,
    include_support: bool,
) -> anyhow::Result<Vec<TestTarget>> {
    let connection = Connection::open(db_path)?;
    let mut ranked_targets = test_targets_for_symbol(&connection, symbol)?
        .into_iter()
        .map(|(target, hit_count)| {
            let is_runnable = is_runnable_test_target(&target);
            (target, hit_count, is_runnable)
        })
        .filter(|(_, _, is_runnable)| include_support || *is_runnable)
        .collect::<Vec<_>>();
    ranked_targets.sort_by(|left, right| {
        right
            .2
            .cmp(&left.2)
            .then(right.1.cmp(&left.1))
            .then(left.0.cmp(&right.0))
    });

    let mut targets = Vec::new();
    for (target, hit_count, is_runnable) in ranked_targets {
        let (confidence, score) = if is_runnable {
            if hit_count > 1 {
                ("graph_likely", 0.9)
            } else {
                ("context_medium", 0.75)
            }
        } else if hit_count > 1 {
            ("context_medium", 0.62)
        } else {
            ("context_medium", 0.58)
        };
        targets.push(TestTarget {
            target: target.clone(),
            target_kind: if is_runnable {
                "integration_test_file".to_string()
            } else {
                "support_test_file".to_string()
            },
            why_included: if is_runnable {
                format!("direct symbol match for '{symbol}' in test file")
            } else {
                format!("direct symbol match for '{symbol}' in support path")
            },
            confidence: confidence.to_string(),
            score,
        });
    }

    Ok(targets)
}

/// Builds a prioritized verification plan (test commands) for the given changed files.
///
/// The function inspects the symbol and test information stored in the SQLite database at
/// `db_path` and produces a list of `VerificationStep` entries describing which test commands
/// to run and why. The returned steps always include a final full-suite step (`"cargo test"`).
///
/// Parameters:
/// - `db_path`: path to the SQLite database containing symbols, references, and test metadata.
/// - `changed_files`: list of changed file paths to analyze for impacted tests.
///
/// The returned vector is sorted by verification scope (targeted steps before full-suite), then
/// by command string, then by the `why_included` message.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// let db = Path::new("code_index.sqlite");
/// let changed = vec!["src/lib.rs".to_string(), "tests/my_test.rs".to_string()];
/// let steps = verify_plan_for_changed_files(db, &changed, &VerifyPlanOptions::default()).unwrap();
/// // `steps` is a Vec<VerificationStep> describing targeted test commands and a final
/// // "cargo test" full-suite step.
/// ```
pub fn verify_plan_for_changed_files(
    db_path: &Path,
    changed_files: &[String],
    options: &VerifyPlanOptions,
) -> anyhow::Result<Vec<VerificationStep>> {
    let connection = Connection::open(db_path)?;
    let changed_lines_by_file = changed_lines_by_file(&options.changed_lines);
    let changed_symbol_filter = options
        .changed_symbols
        .iter()
        .cloned()
        .collect::<HashSet<_>>();

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
            "SELECT DISTINCT symbol, start_line, end_line
             FROM symbols_v2
             WHERE file_path = ?1
             ORDER BY symbol ASC, start_line ASC, end_line ASC",
        )?;
        let symbol_rows = symbols_statement.query_map(params![changed_file], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)? as u32,
                row.get::<_, i64>(2)? as u32,
            ))
        })?;

        for symbol_row in symbol_rows {
            let (symbol, start_line, end_line) = symbol_row?;
            if !changed_symbol_filter.is_empty() && !changed_symbol_filter.contains(&symbol) {
                continue;
            }
            if let Some(ranges) = changed_lines_by_file.get(changed_file)
                && !ranges.iter().any(|range| {
                    line_range_overlaps(start_line, end_line, range.start_line, range.end_line)
                })
            {
                continue;
            }
            if is_generic_changed_symbol(&symbol) {
                continue;
            }
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

    let mut steps = steps_by_command
        .into_values()
        .filter(|step| step.scope == "targeted")
        .collect::<Vec<_>>();
    steps.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(confidence_rank(&right.confidence).cmp(&confidence_rank(&left.confidence)))
            .then(left.step.cmp(&right.step))
            .then(left.why_included.cmp(&right.why_included))
    });
    let mut prioritized = steps
        .iter()
        .filter(|step| is_changed_test_target_reason(&step.why_included))
        .cloned()
        .collect::<Vec<_>>();
    prioritized.sort_by(|left, right| {
        left.step
            .cmp(&right.step)
            .then(left.why_included.cmp(&right.why_included))
    });

    let targeted_cap = options
        .max_targeted
        .unwrap_or(DEFAULT_VERIFY_PLAN_MAX_TARGETED);
    let mut capped = Vec::new();
    let mut capped_count = 0usize;
    for step in steps {
        if is_changed_test_target_reason(&step.why_included) {
            continue;
        }
        if capped_count >= targeted_cap {
            continue;
        }
        capped_count += 1;
        capped.push(step);
    }

    let mut steps = prioritized;
    steps.extend(capped);

    steps.push(VerificationStep {
        step: "cargo test".to_string(),
        scope: "full_suite".to_string(),
        why_included: "required safety gate after refactor".to_string(),
        confidence: "context_high".to_string(),
        score: 1.0,
    });

    steps.sort_by(|left, right| {
        verification_scope_rank(&left.scope)
            .cmp(&verification_scope_rank(&right.scope))
            .then(left.step.cmp(&right.step))
            .then(left.why_included.cmp(&right.why_included))
    });
    Ok(steps)
}

/// Finds AST definition occurrences for the given symbol in the provided SQLite connection.
///
/// Returns a vector of `QueryMatch` entries representing exact AST definition locations for
/// `symbol`. Each returned match has `why_matched = "ast_definition"`, `confidence = "ast_exact"`,
/// and `score = 1.0`.
///
/// # Examples
///
/// ```
/// # use rusqlite::Connection;
/// # fn setup_db(conn: &Connection) { /* populate ast_definitions as needed for the example */ }
/// let conn = Connection::open_in_memory().unwrap();
/// setup_db(&conn);
/// let matches = ast_definition_matches(&conn, "my_symbol").unwrap();
/// // `matches` contains QueryMatch entries for exact AST definitions of "my_symbol".
/// ```
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

fn ranked_text_matches(
    connection: &Connection,
    symbol: &str,
    scope: &QueryScope,
) -> anyhow::Result<Vec<QueryMatch>> {
    let mut matches = text_exact_matches(connection, symbol, scope)?;
    matches.extend(text_substring_matches(connection, symbol, scope)?);
    Ok(matches)
}

fn text_exact_matches(
    connection: &Connection,
    symbol: &str,
    scope: &QueryScope,
) -> anyhow::Result<Vec<QueryMatch>> {
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

    collect_rows(rows).map(|matches| apply_scope_filters(matches, scope))
}

fn text_substring_matches(
    connection: &Connection,
    symbol: &str,
    scope: &QueryScope,
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

    collect_rows(rows).map(|matches| apply_scope_filters(matches, scope))
}

fn apply_scope_filters(matches: Vec<QueryMatch>, scope: &QueryScope) -> Vec<QueryMatch> {
    matches
        .into_iter()
        .filter(|item| !scope.code_only || is_code_file_path(&item.file_path))
        .filter(|item| !scope.exclude_tests || !is_test_like_path(&item.file_path))
        .collect()
}

fn is_code_file_path(file_path: &str) -> bool {
    file_path.ends_with(".rs")
        || file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".py")
}

fn is_test_like_path(file_path: &str) -> bool {
    file_path.starts_with("tests/")
        || file_path.contains("/tests/")
        || file_path.ends_with("_test.rs")
}

/// Collects all mapped rows into a vector of `QueryMatch`.
///
/// This consumes the provided `MappedRows` iterator, returning a `Vec<QueryMatch>` built from each successful row mapping. Any row-mapping error is returned.
///
/// # Returns
///
/// A `Vec<QueryMatch>` containing the results of mapping every row.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
///
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute("CREATE TABLE t(x TEXT, line INTEGER, col INTEGER)", [], ).unwrap();
/// conn.execute("INSERT INTO t VALUES('sym',1,2)", [], ).unwrap();
///
/// let mut stmt = conn.prepare("SELECT x, line, col FROM t").unwrap();
/// let mapped = stmt.query_map([], |row| {
///     Ok(QueryMatch{
///         file_path: "file.rs".into(),
///         line: row.get(1)?,
///         column: row.get(2)?,
///         symbol: row.get(0)?,
///         why_matched: "example".into(),
///         confidence: "example_conf".into(),
///         score: 1.0,
///     })
/// }).unwrap();
///
/// let vec = collect_rows(mapped).unwrap();
/// assert_eq!(vec.len(), 1);
/// ```
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

/// Extracts meaningful lowercase keywords from a task string.
///
/// Splits the input on characters that are not ASCII alphanumeric or underscore,
/// lowercases each token, ignores tokens shorter than 3 characters, and returns
/// deduplicated tokens in their first-occurrence order.
///
/// # Examples
///
/// ```
/// let kws = extract_keywords("Fix crash in HTTPServer::handle_req v2");
/// assert_eq!(kws, vec!["fix", "crash", "httpserver", "handle_req"]);
/// ```
fn extract_keywords(task: &str) -> Vec<String> {
    let mut keywords = Vec::new();
    let mut seen = HashSet::new();
    let stopwords = [
        "about", "after", "and", "for", "from", "into", "that", "the", "then", "this", "when",
        "with",
    ];

    for token in task
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|token| !token.is_empty())
    {
        let lowered = token.to_ascii_lowercase();
        if lowered.len() < 3 {
            continue;
        }
        if stopwords.contains(&lowered.as_str()) {
            continue;
        }
        if seen.insert(lowered.clone()) {
            keywords.push(lowered);
        }
    }

    keywords
}

fn symbol_keywords(symbol: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut previous_was_lower = false;

    for ch in symbol.chars() {
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            if !current.is_empty() {
                tokens.push(current.to_ascii_lowercase());
                current.clear();
            }
            previous_was_lower = false;
            continue;
        }

        if ch == '_' {
            if !current.is_empty() {
                tokens.push(current.to_ascii_lowercase());
                current.clear();
            }
            previous_was_lower = false;
            continue;
        }

        if ch.is_ascii_uppercase() && previous_was_lower && !current.is_empty() {
            tokens.push(current.to_ascii_lowercase());
            current.clear();
        }

        previous_was_lower = ch.is_ascii_lowercase();
        current.push(ch);
    }

    if !current.is_empty() {
        tokens.push(current.to_ascii_lowercase());
    }

    let mut filtered = tokens
        .into_iter()
        .filter(|token| token.len() >= 3)
        .collect::<Vec<_>>();
    filtered.sort();
    filtered.dedup();
    filtered
}

fn matched_task_keywords(task_keywords: &[String], symbol_keywords: &[String]) -> Vec<String> {
    let mut matched = Vec::new();
    for task_keyword in task_keywords {
        if symbol_keywords.iter().any(|symbol_keyword| {
            symbol_keyword == task_keyword
                || symbol_keyword.starts_with(task_keyword)
                || task_keyword.starts_with(symbol_keyword)
        }) {
            matched.push(task_keyword.clone());
        }
    }
    matched
}

fn context_direct_score(
    overlap_count: usize,
    exact_symbol_match: bool,
    symbol_token_count: usize,
) -> f64 {
    let specificity_bonus = std::cmp::min(symbol_token_count, 4) as f64 * 0.04;
    let mut score = 0.62 + (overlap_count as f64 * 0.09) + specificity_bonus;
    if exact_symbol_match {
        score += 0.04;
    }
    score.min(0.98)
}

/// Finds test files that reference a symbol and how often the symbol appears in each.
///
/// Returns a vector of `(file_path, hit_count)` for files that look like test targets
/// (paths under `tests/` or files matching `*_test.rs` / `*test.rs`), ordered by `hit_count` descending
/// and then by `file_path` ascending.
///
/// # Examples
///
/// ```no_run
/// use rusqlite::Connection;
/// let conn = Connection::open("path/to/db.sqlite").unwrap();
/// let targets = test_targets_for_symbol(&conn, "my_symbol").unwrap();
/// for (file_path, hit_count) in targets {
///     println!("{} -> {}", file_path, hit_count);
/// }
/// ```
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
               OR file_path GLOB '*_test.rs'
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

/// Derives a `cargo test` invocation for a standalone test file located directly under a `tests/` directory.
///
/// Returns `Some` with the command `cargo test --test {stem}` when `target` is a path of the form `tests/<file>` (no additional subdirectories)
/// and the file has a valid stem; returns `None` otherwise.
///
/// # Examples
///
/// ```
/// assert_eq!(
///     test_command_for_target("tests/integration_test.rs"),
///     Some("cargo test --test integration_test".to_string())
/// );
///
/// // Nested paths are rejected
/// assert_eq!(test_command_for_target("tests/subdir/integration_test.rs"), None);
///
/// // Non-tests directory is rejected
/// assert_eq!(test_command_for_target("src/lib.rs"), None);
/// ```
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

fn is_runnable_test_target(target: &str) -> bool {
    test_command_for_target(target).is_some()
}

/// Assigns a numeric rank to a verification scope for ordering.
///
/// # Returns
///
/// 0 for "targeted", 1 for "full_suite", and 2 for any other scope.
///
/// # Examples
///
/// ```
/// assert_eq!(verification_scope_rank("targeted"), 0);
/// assert_eq!(verification_scope_rank("full_suite"), 1);
/// assert_eq!(verification_scope_rank("foo"), 2);
/// ```
fn verification_scope_rank(scope: &str) -> u8 {
    match scope {
        "targeted" => 0,
        "full_suite" => 1,
        _ => 2,
    }
}

/// Insert or update a verification step in the map keyed by its `step`, keeping the best candidate.
///
/// Replaces an existing entry for the same `step` if the `candidate` has:
/// - a greater `score`, or
/// - the same `score` but a higher `confidence` (as ranked by `confidence_rank`), or
/// - the same `score` and `confidence` rank but a lexicographically smaller `why_included`.
///
/// # Arguments
///
/// * `steps_by_command` - Map keyed by `step` to the chosen `VerificationStep`.
/// * `candidate` - Candidate `VerificationStep` to insert or to use as a possible replacement.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
///
/// // Construct two simple candidates that differ by score.
/// let mut map: HashMap<String, super::VerificationStep> = HashMap::new();
///
/// let a = super::VerificationStep {
///     step: "cargo test".to_string(),
///     scope: "full_suite".to_string(),
///     why_included: "initial".to_string(),
///     confidence: "context_medium".to_string(),
///     score: 0.8,
/// };
///
/// let b = super::VerificationStep {
///     step: "cargo test".to_string(),
///     scope: "full_suite".to_string(),
///     why_included: "replacement".to_string(),
///     confidence: "context_medium".to_string(),
///     score: 0.95,
/// };
///
/// super::upsert_verification_step(&mut map, a);
/// super::upsert_verification_step(&mut map, b);
///
/// assert_eq!(map.get("cargo test").unwrap().score, 0.95);
/// ```
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

/// Convert a confidence label into a numeric ranking where larger values indicate stronger confidence.
///
/// # Returns
///
/// `u8` where larger values indicate greater confidence: `3` for `"graph_likely"`, `2` for `"context_high"`, `1` for `"context_medium"`, and `0` for any other input.
///
/// # Examples
///
/// ```
/// assert_eq!(confidence_rank("graph_likely"), 3);
/// assert_eq!(confidence_rank("context_high"), 2);
/// assert_eq!(confidence_rank("context_medium"), 1);
/// assert_eq!(confidence_rank("unknown"), 0);
/// ```
fn confidence_rank(confidence: &str) -> u8 {
    match confidence {
        "graph_likely" => 3,
        "context_high" => 2,
        "context_medium" => 1,
        _ => 0,
    }
}

fn is_generic_changed_symbol(symbol: &str) -> bool {
    const GENERIC_SYMBOLS: &[&str] = &[
        "args", "common", "error", "file", "files", "json", "main", "mod", "output", "path",
        "query", "repo", "result", "run", "symbol", "test", "tests", "value",
    ];

    let lowered = symbol.to_ascii_lowercase();
    GENERIC_SYMBOLS.contains(&lowered.as_str())
}

fn is_changed_test_target_reason(why_included: &str) -> bool {
    why_included.starts_with("changed file '") && why_included.ends_with("is itself a test target")
}
