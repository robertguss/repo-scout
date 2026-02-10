use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;
use serde_json::Value as JsonValue;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum QueryPathMode {
    #[default]
    AllFiles,
    CodeOnly,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum QueryTestMode {
    #[default]
    IncludeTests,
    ExcludeTests,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryScope {
    pub path_mode: QueryPathMode,
    pub test_mode: QueryTestMode,
}

impl QueryScope {
    #[must_use]
    pub fn from_flags(code_only: bool, exclude_tests: bool) -> Self {
        let path_mode = if code_only {
            QueryPathMode::CodeOnly
        } else {
            QueryPathMode::AllFiles
        };
        let test_mode = if exclude_tests {
            QueryTestMode::ExcludeTests
        } else {
            QueryTestMode::IncludeTests
        };
        Self {
            path_mode,
            test_mode,
        }
    }

    #[must_use]
    fn includes_path(self, file_path: &str) -> bool {
        let path_allowed = match self.path_mode {
            QueryPathMode::AllFiles => true,
            QueryPathMode::CodeOnly => is_code_file_path(file_path),
        };
        let tests_allowed = match self.test_mode {
            QueryTestMode::IncludeTests => true,
            QueryTestMode::ExcludeTests => !is_test_like_path(file_path),
        };
        path_allowed && tests_allowed
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiffImpactTestMode {
    #[default]
    IncludeTests,
    ExcludeTests,
}

impl DiffImpactTestMode {
    #[must_use]
    pub fn include_tests(self) -> bool {
        matches!(self, Self::IncludeTests)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiffImpactImportMode {
    #[default]
    ExcludeImports,
    IncludeImports,
}

impl DiffImpactImportMode {
    #[must_use]
    pub fn include_imports(self) -> bool {
        matches!(self, Self::IncludeImports)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DiffImpactChangedMode {
    #[default]
    IncludeChanged,
    ExcludeChanged,
}

impl DiffImpactChangedMode {
    #[must_use]
    pub fn exclude_changed(self) -> bool {
        matches!(self, Self::ExcludeChanged)
    }
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
    pub test_mode: DiffImpactTestMode,
    pub import_mode: DiffImpactImportMode,
    pub changed_lines: Vec<ChangedLineRange>,
    pub changed_symbols: Vec<String>,
    pub changed_mode: DiffImpactChangedMode,
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyPlanOptions {
    pub max_targeted: Option<u32>,
    pub changed_lines: Vec<ChangedLineRange>,
    pub changed_symbols: Vec<String>,
}

pub const DEFAULT_VERIFY_PLAN_MAX_TARGETED: u32 = 8;

#[must_use]
fn bounded_usize(value: u32) -> usize {
    let supports_u32_boundary = usize::BITS >= 32;
    debug_assert!(
        supports_u32_boundary,
        "repo-scout requires usize to represent u32 boundary values"
    );
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[must_use = "diff-impact results should be consumed by callers"]
pub fn diff_impact_for_changed_files(
    db_path: &Path,
    changed_files: &[String],
    options: &DiffImpactOptions,
) -> anyhow::Result<Vec<DiffImpactMatch>> {
    let connection = Connection::open(db_path)?;
    let changed_lines_by_file = changed_lines_by_file(&options.changed_lines);
    let changed_symbol_filter = options
        .changed_symbols
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let mut state = DiffImpactState::default();
    collect_changed_symbol_matches(
        &connection,
        changed_files,
        options,
        &changed_lines_by_file,
        &changed_symbol_filter,
        &mut state,
    )?;
    expand_changed_symbol_neighbors(&connection, options.max_distance, &mut state)?;
    if options.test_mode.include_tests() {
        append_diff_impact_test_targets(&connection, &mut state.results)?;
    }
    if options.changed_mode.exclude_changed() {
        remove_changed_symbol_rows(&mut state.results);
    }
    sort_and_cap_diff_impact_results(&mut state.results, options.max_results);
    Ok(state.results)
}

#[derive(Debug, Default)]
struct DiffImpactState {
    results: Vec<DiffImpactMatch>,
    seen: HashSet<String>,
    changed_symbol_ids: Vec<(i64, String)>,
}

#[derive(Debug)]
struct ChangedSymbolSeed {
    symbol_id: i64,
    symbol: String,
    kind: String,
    file_path: String,
    line: u32,
    column: u32,
    end_line: u32,
    language: String,
    qualified_symbol: Option<String>,
}

#[derive(Debug)]
struct IncomingNeighbor {
    from_symbol_id: i64,
    symbol: String,
    kind: String,
    file_path: String,
    line: u32,
    column: u32,
    language: String,
    qualified_symbol: Option<String>,
    edge_kind: String,
    score: f64,
    provenance: String,
}

fn collect_changed_symbol_matches(
    connection: &Connection,
    changed_files: &[String],
    options: &DiffImpactOptions,
    changed_lines_by_file: &HashMap<String, Vec<ChangedLineRange>>,
    changed_symbol_filter: &HashSet<String>,
    state: &mut DiffImpactState,
) -> anyhow::Result<()> {
    for changed_file in changed_files {
        for seed in changed_symbol_seeds(
            connection,
            changed_file,
            options.import_mode.include_imports(),
        )? {
            if !matches_changed_symbol_filters(
                &seed,
                changed_file,
                changed_lines_by_file,
                changed_symbol_filter,
            ) {
                continue;
            }
            let language = normalized_language(&seed.language, &seed.file_path).to_string();
            let qualified_symbol = seed
                .qualified_symbol
                .unwrap_or_else(|| format!("{language}:{}::{}", seed.file_path, seed.symbol));
            let key = format!(
                "{}:{}:{}:{qualified_symbol}:changed_symbol:0",
                seed.file_path, seed.line, seed.column
            );
            if !state.seen.insert(key) {
                continue;
            }
            state
                .changed_symbol_ids
                .push((seed.symbol_id, seed.symbol.clone()));
            state.results.push(DiffImpactMatch::ImpactedSymbol {
                symbol: seed.symbol,
                qualified_symbol,
                kind: seed.kind,
                language,
                file_path: seed.file_path,
                line: seed.line,
                column: seed.column,
                distance: 0,
                relationship: "changed_symbol".to_string(),
                why_included: "symbol defined in changed file".to_string(),
                confidence: "graph_exact".to_string(),
                provenance: "ast_definition".to_string(),
                score: 1.0,
            });
        }
    }
    Ok(())
}

fn changed_symbol_seeds(
    connection: &Connection,
    changed_file: &str,
    include_imports: bool,
) -> anyhow::Result<Vec<ChangedSymbolSeed>> {
    let mut statement = connection.prepare(
        "SELECT symbol_id, symbol, kind, file_path, start_line, start_column, end_line,
                language, qualified_symbol
         FROM symbols_v2
         WHERE file_path = ?1
           AND (?2 OR kind <> 'import')
         ORDER BY start_line ASC, start_column ASC, symbol ASC",
    )?;
    let rows = statement.query_map(params![changed_file, include_imports], |row| {
        Ok(ChangedSymbolSeed {
            symbol_id: row.get::<_, i64>(0)?,
            symbol: row.get::<_, String>(1)?,
            kind: row.get::<_, String>(2)?,
            file_path: row.get::<_, String>(3)?,
            line: row.get::<_, i64>(4)? as u32,
            column: row.get::<_, i64>(5)? as u32,
            end_line: row.get::<_, i64>(6)? as u32,
            language: row.get::<_, String>(7)?,
            qualified_symbol: row.get::<_, Option<String>>(8)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn matches_changed_symbol_filters(
    seed: &ChangedSymbolSeed,
    changed_file: &str,
    changed_lines_by_file: &HashMap<String, Vec<ChangedLineRange>>,
    changed_symbol_filter: &HashSet<String>,
) -> bool {
    if let Some(ranges) = changed_lines_by_file.get(changed_file)
        && !ranges.iter().any(|range| {
            line_range_overlaps(seed.line, seed.end_line, range.start_line, range.end_line)
        })
    {
        return false;
    }
    changed_symbol_filter.is_empty() || changed_symbol_filter.contains(&seed.symbol)
}

fn expand_changed_symbol_neighbors(
    connection: &Connection,
    max_distance: u32,
    state: &mut DiffImpactState,
) -> anyhow::Result<()> {
    if max_distance < 1 {
        return Ok(());
    }
    let changed_symbol_id_set = state
        .changed_symbol_ids
        .iter()
        .map(|(symbol_id, _)| *symbol_id)
        .collect::<HashSet<_>>();
    let changed_symbol_ids = state.changed_symbol_ids.clone();
    for (changed_symbol_id, changed_symbol) in changed_symbol_ids {
        expand_neighbors_for_symbol(
            connection,
            max_distance,
            &changed_symbol_id_set,
            changed_symbol_id,
            &changed_symbol,
            state,
        )?;
    }
    Ok(())
}

fn expand_neighbors_for_symbol(
    connection: &Connection,
    traversal_limit: u32,
    changed_symbol_id_set: &HashSet<i64>,
    changed_symbol_id: i64,
    changed_symbol: &str,
    state: &mut DiffImpactState,
) -> anyhow::Result<()> {
    let mut frontier = VecDeque::new();
    let mut min_distance_by_symbol = HashMap::new();
    frontier.push_back((changed_symbol_id, 0_u32));
    min_distance_by_symbol.insert(changed_symbol_id, 0_u32);
    while let Some((to_symbol_id, distance)) = frontier.pop_front() {
        if distance >= traversal_limit {
            continue;
        }
        let next_distance = distance + 1;
        for incoming in incoming_neighbors(connection, to_symbol_id)? {
            push_incoming_neighbor(
                incoming,
                next_distance,
                changed_symbol,
                changed_symbol_id_set,
                &mut min_distance_by_symbol,
                &mut frontier,
                state,
            );
        }
    }
    Ok(())
}

fn incoming_neighbors(
    connection: &Connection,
    to_symbol_id: i64,
) -> anyhow::Result<Vec<IncomingNeighbor>> {
    let mut statement = connection.prepare(
        "SELECT fs.symbol_id, fs.symbol, fs.kind, fs.file_path, fs.start_line, fs.start_column,
                fs.language, fs.qualified_symbol, e.edge_kind, e.confidence, e.provenance
         FROM symbol_edges_v2 e
         JOIN symbols_v2 fs ON fs.symbol_id = e.from_symbol_id
         WHERE e.to_symbol_id = ?1
         ORDER BY fs.file_path ASC, fs.start_line ASC, fs.start_column ASC, fs.symbol ASC",
    )?;
    let rows = statement.query_map(params![to_symbol_id], |row| {
        Ok(IncomingNeighbor {
            from_symbol_id: row.get::<_, i64>(0)?,
            symbol: row.get::<_, String>(1)?,
            kind: row.get::<_, String>(2)?,
            file_path: row.get::<_, String>(3)?,
            line: row.get::<_, i64>(4)? as u32,
            column: row.get::<_, i64>(5)? as u32,
            language: row.get::<_, String>(6)?,
            qualified_symbol: row.get::<_, Option<String>>(7)?,
            edge_kind: row.get::<_, String>(8)?,
            score: row.get::<_, f64>(9)?,
            provenance: row.get::<_, String>(10)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn push_incoming_neighbor(
    incoming: IncomingNeighbor,
    next_distance: u32,
    changed_symbol: &str,
    changed_symbol_id_set: &HashSet<i64>,
    min_distance_by_symbol: &mut HashMap<i64, u32>,
    frontier: &mut VecDeque<(i64, u32)>,
    state: &mut DiffImpactState,
) {
    if changed_symbol_id_set.contains(&incoming.from_symbol_id) {
        return;
    }
    if min_distance_by_symbol
        .get(&incoming.from_symbol_id)
        .is_some_and(|known| *known < next_distance)
    {
        return;
    }
    let known_distance = min_distance_by_symbol
        .get(&incoming.from_symbol_id)
        .copied();
    let should_expand_frontier = known_distance != Some(next_distance);
    if known_distance.is_none_or(|known| next_distance < known) {
        min_distance_by_symbol.insert(incoming.from_symbol_id, next_distance);
    }
    let language = normalized_language(&incoming.language, &incoming.file_path).to_string();
    let qualified_symbol = incoming
        .qualified_symbol
        .unwrap_or_else(|| format!("{language}:{}::{}", incoming.file_path, incoming.symbol));
    let relationship = edge_kind_relationship(&incoming.edge_kind);
    let provenance = normalized_provenance(&incoming.provenance, &incoming.edge_kind);
    let confidence = calibrated_semantic_confidence(&provenance);
    let score = calibrated_semantic_score(relationship, &provenance, next_distance, incoming.score);
    let key = format!(
        "{}:{}:{}:{qualified_symbol}:{relationship}:distance{next_distance}",
        incoming.file_path, incoming.line, incoming.column
    );
    if !state.seen.insert(key) {
        return;
    }
    state.results.push(DiffImpactMatch::ImpactedSymbol {
        symbol: incoming.symbol,
        qualified_symbol,
        kind: incoming.kind,
        language,
        file_path: incoming.file_path,
        line: incoming.line,
        column: incoming.column,
        distance: next_distance,
        relationship: relationship.to_string(),
        why_included: format!(
            "direct {relationship} neighbor of changed symbol '{changed_symbol}'"
        ),
        confidence,
        provenance,
        score,
    });
    if should_expand_frontier {
        frontier.push_back((incoming.from_symbol_id, next_distance));
    }
}

fn append_diff_impact_test_targets(
    connection: &Connection,
    results: &mut Vec<DiffImpactMatch>,
) -> anyhow::Result<()> {
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
        for (target, hit_count) in test_targets_for_symbol(connection, &symbol)? {
            let (confidence, score) = calibrated_test_target_rank(hit_count);
            let key = format!("integration_test_file:{target}");
            if !should_replace_test_target(&selected_test_targets, &key, score) {
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
    Ok(())
}

fn should_replace_test_target(
    selected_test_targets: &BTreeMap<String, DiffImpactMatch>,
    key: &str,
    score: f64,
) -> bool {
    match selected_test_targets.get(key) {
        Some(DiffImpactMatch::TestTarget {
            score: existing_score,
            ..
        }) => score > *existing_score,
        _ => true,
    }
}

fn remove_changed_symbol_rows(results: &mut Vec<DiffImpactMatch>) {
    results.retain(|item| {
        !matches!(
            item,
            DiffImpactMatch::ImpactedSymbol { relationship, .. } if relationship == "changed_symbol"
        )
    });
}

fn sort_and_cap_diff_impact_results(results: &mut Vec<DiffImpactMatch>, max_results: Option<u32>) {
    results.sort_by(diff_impact_sort_key);
    if let Some(max_results) = max_results {
        results.truncate(bounded_usize(max_results));
        debug_assert!(
            results.len() <= bounded_usize(max_results),
            "diff-impact truncation must not exceed configured max_results"
        );
    }
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
        "SELECT symbol_id, symbol, kind, file_path, start_line, start_column, end_line,
                end_column, signature, language, qualified_symbol
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

fn calibrated_semantic_confidence(provenance: &str) -> String {
    match provenance {
        "call_resolution" | "import_resolution" | "ast_definition" | "ast_reference" => {
            "graph_likely".to_string()
        }
        "text_fallback" => "context_medium".to_string(),
        _ => "context_low".to_string(),
    }
}

fn calibrated_semantic_score(
    relationship: &str,
    provenance: &str,
    distance: u32,
    _raw_score: f64,
) -> f64 {
    let baseline = match (provenance, relationship) {
        ("call_resolution", "called_by") => 0.97,
        ("import_resolution", "imported_by") => 0.95,
        ("ast_reference", "implemented_by") => 0.94,
        ("ast_definition", "contained_by") => 0.94,
        ("call_resolution", _) => 0.95,
        ("import_resolution", _) => 0.94,
        ("ast_reference", _) => 0.93,
        ("ast_definition", _) => 0.93,
        ("text_fallback", _) => 0.72,
        _ => 0.90,
    };
    let distance_penalty = if distance > 1 {
        (distance.saturating_sub(1) as f64) * 0.01
    } else {
        0.0
    };
    (baseline - distance_penalty).clamp(0.0, 1.0)
}

fn calibrated_test_target_rank(hit_count: i64) -> (&'static str, f64) {
    if hit_count > 1 {
        ("graph_likely", 0.84)
    } else {
        ("context_medium", 0.70)
    }
}

fn language_for_file_path(file_path: &str) -> &'static str {
    if file_path.ends_with(".rs") {
        "rust"
    } else if file_path.ends_with(".ts") || file_path.ends_with(".tsx") {
        "typescript"
    } else if file_path.ends_with(".py") {
        "python"
    } else if file_path.ends_with(".go") {
        "go"
    } else {
        "unknown"
    }
}

fn normalized_language(language: &str, file_path: &str) -> &'static str {
    match language {
        "rust" => "rust",
        "typescript" => "typescript",
        "python" => "python",
        "go" => "go",
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

/// Finds code locations that match `symbol`.
/// Prefers exact AST definitions and falls back to text matches.
///
/// Searches the SQLite database at `db_path` for exact AST definition matches of `symbol`.
/// If AST matches are found, those are returned.
/// Otherwise, the function returns ranked text-based matches.
///
/// # Parameters
///
/// - `db_path`: Path to the SQLite database containing indexed symbols and occurrences.
/// - `symbol`: The symbol name to search for.
///
/// # Returns
///
/// A vector of `QueryMatch` entries where `symbol` appears, ordered by relevance.
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

#[must_use = "query results should be consumed by callers"]
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

#[must_use = "reference results should be consumed by callers"]
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
            "SELECT fs.file_path, fs.start_line, fs.start_column, fs.symbol, fs.kind,
                    e.edge_kind, e.confidence, e.provenance
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
                _ => edge_kind.clone(),
            };
            let raw_score = row.get::<_, f64>(6)?;
            let raw_provenance = row.get::<_, String>(7)?;
            let provenance = normalized_provenance(&raw_provenance, &edge_kind);
            Ok(ImpactMatch {
                file_path: row.get(0)?,
                line: row.get::<_, i64>(1)? as u32,
                column: row.get::<_, i64>(2)? as u32,
                symbol: row.get(3)?,
                kind: row.get(4)?,
                distance: 1,
                relationship: relationship.clone(),
                confidence: calibrated_semantic_confidence(&provenance),
                score: calibrated_semantic_score(&relationship, &provenance, 1, raw_score),
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
/// let matches = query::context_matches(
///     Path::new("db.sqlite"),
///     "refactor authentication flow",
///     1000,
/// )
/// .unwrap();
/// assert!(!matches.is_empty());
/// ```
pub fn context_matches(
    db_path: &Path,
    task: &str,
    budget: u32,
) -> anyhow::Result<Vec<ContextMatch>> {
    context_matches_scoped(db_path, task, budget, &QueryScope::default())
}

#[must_use = "context matches should be consumed by callers"]
pub fn context_matches_scoped(
    db_path: &Path,
    task: &str,
    budget: u32,
    scope: &QueryScope,
) -> anyhow::Result<Vec<ContextMatch>> {
    let connection = Connection::open(db_path)?;
    let keywords = extract_keywords(task);
    if keywords.is_empty() {
        return Ok(Vec::new());
    }
    let mut matches = Vec::new();
    let mut seen = HashSet::new();
    for seed in context_seed_symbols(&connection)? {
        let Some(metadata) = context_match_metadata(&keywords, &seed.symbol) else {
            continue;
        };
        push_direct_context_match(&mut matches, &mut seen, &seed, &metadata);
        push_neighbor_context_matches(
            &connection,
            seed.symbol_id,
            &seed.symbol,
            &metadata.matched_keywords,
            metadata.direct_score,
            &mut seen,
            &mut matches,
        )?;
    }
    filter_context_matches_by_scope(&mut matches, scope);
    sort_context_matches(&mut matches);
    truncate_context_matches_by_budget(&mut matches, budget);
    Ok(matches)
}

#[derive(Debug)]
struct ContextSeedSymbol {
    symbol_id: i64,
    file_path: String,
    symbol: String,
    kind: String,
    start_line: u32,
    end_line: u32,
}

#[derive(Debug)]
struct ContextMatchMetadata {
    matched_keywords: Vec<String>,
    overlap_count: usize,
    exact_symbol_match: bool,
    direct_score: f64,
}

fn context_seed_symbols(connection: &Connection) -> anyhow::Result<Vec<ContextSeedSymbol>> {
    let mut statement = connection.prepare(
        "SELECT symbol_id, file_path, symbol, kind, start_line, end_line
         FROM symbols_v2
         ORDER BY file_path ASC, start_line ASC, start_column ASC, symbol ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(ContextSeedSymbol {
            symbol_id: row.get::<_, i64>(0)?,
            file_path: row.get::<_, String>(1)?,
            symbol: row.get::<_, String>(2)?,
            kind: row.get::<_, String>(3)?,
            start_line: row.get::<_, i64>(4)? as u32,
            end_line: row.get::<_, i64>(5)? as u32,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn context_match_metadata(keywords: &[String], symbol: &str) -> Option<ContextMatchMetadata> {
    let symbol_tokens = symbol_keywords(symbol);
    let matched_keywords = matched_task_keywords(keywords, &symbol_tokens);
    if matched_keywords.is_empty() {
        return None;
    }
    let overlap_count = matched_keywords.len();
    let exact_symbol_match = keywords
        .iter()
        .any(|keyword| keyword == &symbol.to_ascii_lowercase());
    let direct_score = context_direct_score(overlap_count, exact_symbol_match, symbol_tokens.len());
    Some(ContextMatchMetadata {
        matched_keywords,
        overlap_count,
        exact_symbol_match,
        direct_score,
    })
}

fn push_direct_context_match(
    matches: &mut Vec<ContextMatch>,
    seen: &mut HashSet<String>,
    seed: &ContextSeedSymbol,
    metadata: &ContextMatchMetadata,
) {
    let key = format!(
        "{}:{}:{}:{}:direct",
        seed.file_path, seed.start_line, seed.symbol, seed.kind
    );
    if !seen.insert(key) {
        return;
    }
    matches.push(ContextMatch {
        file_path: seed.file_path.clone(),
        start_line: seed.start_line,
        end_line: seed.end_line,
        symbol: seed.symbol.clone(),
        kind: seed.kind.clone(),
        why_included: format!(
            "direct definition token-overlap relevance for [{}]",
            metadata.matched_keywords.join(", ")
        ),
        confidence: if metadata.overlap_count >= 2 || metadata.exact_symbol_match {
            "context_high".to_string()
        } else {
            "context_medium".to_string()
        },
        score: metadata.direct_score,
    });
}

fn push_neighbor_context_matches(
    connection: &Connection,
    symbol_id: i64,
    symbol: &str,
    matched_keywords: &[String],
    direct_score: f64,
    seen: &mut HashSet<String>,
    matches: &mut Vec<ContextMatch>,
) -> anyhow::Result<()> {
    let mut statement = connection.prepare(
        "SELECT n.file_path, n.symbol, n.kind, n.start_line, n.end_line
         FROM symbol_edges_v2 e
         JOIN symbols_v2 n ON n.symbol_id = e.to_symbol_id
         WHERE e.from_symbol_id = ?1
         ORDER BY n.file_path ASC, n.start_line ASC, n.start_column ASC, n.symbol ASC",
    )?;
    let rows = statement.query_map(params![symbol_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)? as u32,
            row.get::<_, i64>(4)? as u32,
        ))
    })?;
    for row in rows {
        let (file_path, neighbor_symbol, kind, start_line, end_line) = row?;
        let key = format!("{file_path}:{start_line}:{neighbor_symbol}:{kind}:neighbor");
        if !seen.insert(key) {
            continue;
        }
        matches.push(ContextMatch {
            file_path,
            start_line,
            end_line,
            symbol: neighbor_symbol,
            kind,
            why_included: format!(
                "graph neighbor of '{symbol}' from token-overlap relevance [{}]",
                matched_keywords.join(", ")
            ),
            confidence: "context_medium".to_string(),
            score: (direct_score - 0.2).max(0.55),
        });
    }
    Ok(())
}

fn filter_context_matches_by_scope(matches: &mut Vec<ContextMatch>, scope: &QueryScope) {
    matches.retain(|item| scope.includes_path(&item.file_path));
}

fn sort_context_matches(matches: &mut [ContextMatch]) {
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
}

fn truncate_context_matches_by_budget(matches: &mut Vec<ContextMatch>, budget: u32) {
    let max_results = std::cmp::max(1, budget / 200);
    debug_assert!(
        max_results >= 1,
        "context budget must map to at least one result"
    );
    matches.truncate(bounded_usize(max_results));
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
    let runners = RecommendationRunners::for_db_path(db_path);
    let mut ranked_targets = test_targets_for_symbol(&connection, symbol)?
        .into_iter()
        .map(|(target, hit_count)| {
            let is_runnable = is_runnable_test_target(&target, &runners);
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
/// to run and why. The returned steps always include a final full-suite safety gate whose command
/// depends on detected runner context (`cargo test` by default).
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
/// // full-suite safety-gate step.
/// ```
#[must_use = "verification plans should be consumed by callers"]
pub fn verify_plan_for_changed_files(
    db_path: &Path,
    changed_files: &[String],
    options: &VerifyPlanOptions,
) -> anyhow::Result<Vec<VerificationStep>> {
    let connection = Connection::open(db_path)?;
    let runners = RecommendationRunners::for_db_path(db_path);
    let changed_lines_by_file = changed_lines_by_file(&options.changed_lines);
    let changed_symbol_filter = options
        .changed_symbols
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let mut steps_by_command: HashMap<String, VerificationStep> = HashMap::new();
    for changed_file in changed_files {
        add_changed_file_target_step(changed_file, &runners, &mut steps_by_command);
        add_changed_symbol_target_steps(
            &connection,
            changed_file,
            &changed_lines_by_file,
            &changed_symbol_filter,
            &runners,
            &mut steps_by_command,
        )?;
    }
    let targeted_cap = options
        .max_targeted
        .unwrap_or(DEFAULT_VERIFY_PLAN_MAX_TARGETED);
    let mut steps = finalize_targeted_verification_steps(steps_by_command, targeted_cap);
    append_full_suite_verification_step(&mut steps, changed_files, &runners);
    sort_verification_steps(&mut steps);
    Ok(steps)
}

#[derive(Debug, Clone, Copy, Default)]
struct RecommendationRunners {
    pytest: bool,
    node: NodeTestRunner,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum NodeTestRunner {
    #[default]
    None,
    Jest,
    Vitest,
    Ambiguous,
}

impl NodeTestRunner {
    fn targeted_command_for(self, target: &str) -> Option<String> {
        match self {
            Self::Jest => Some(format!("npx jest --runTestsByPath {target}")),
            Self::Vitest => Some(format!("npx vitest run {target}")),
            Self::None | Self::Ambiguous => None,
        }
    }

    fn full_suite_command(self) -> Option<&'static str> {
        match self {
            Self::Jest => Some("npx jest"),
            Self::Vitest => Some("npx vitest run"),
            Self::None | Self::Ambiguous => None,
        }
    }
}

impl RecommendationRunners {
    fn for_db_path(db_path: &Path) -> Self {
        let Some(repo_root) = repo_root_from_db_path(db_path) else {
            return Self::default();
        };
        Self {
            pytest: is_pytest_explicitly_configured(repo_root),
            node: detect_node_test_runner(repo_root),
        }
    }
}

fn repo_root_from_db_path(db_path: &Path) -> Option<&Path> {
    let index_dir = db_path.parent()?;
    let is_repo_scout_dir = index_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".repo-scout");
    if !is_repo_scout_dir {
        return None;
    }
    index_dir.parent()
}

fn is_pytest_explicitly_configured(repo_root: &Path) -> bool {
    repo_root.join("pytest.ini").is_file()
        || file_contains(
            repo_root.join("pyproject.toml"),
            "[tool.pytest.ini_options]",
        )
        || file_contains(repo_root.join("tox.ini"), "[pytest]")
        || file_contains(repo_root.join("setup.cfg"), "[tool:pytest]")
}

fn detect_node_test_runner(repo_root: &Path) -> NodeTestRunner {
    let package_json_path = repo_root.join("package.json");
    let Ok(contents) = fs::read_to_string(package_json_path) else {
        return NodeTestRunner::None;
    };
    let Ok(package_json) = serde_json::from_str::<JsonValue>(&contents) else {
        return NodeTestRunner::None;
    };

    let has_jest = package_json_signals_runner(&package_json, "jest");
    let has_vitest = package_json_signals_runner(&package_json, "vitest");
    match (has_jest, has_vitest) {
        (true, false) => NodeTestRunner::Jest,
        (false, true) => NodeTestRunner::Vitest,
        (true, true) => NodeTestRunner::Ambiguous,
        (false, false) => NodeTestRunner::None,
    }
}

fn package_json_signals_runner(package_json: &JsonValue, runner: &str) -> bool {
    script_test_contains_runner(package_json, runner)
        || dependency_declares_runner(package_json, "dependencies", runner)
        || dependency_declares_runner(package_json, "devDependencies", runner)
        || dependency_declares_runner(package_json, "peerDependencies", runner)
        || dependency_declares_runner(package_json, "optionalDependencies", runner)
}

fn script_test_contains_runner(package_json: &JsonValue, runner: &str) -> bool {
    package_json
        .get("scripts")
        .and_then(JsonValue::as_object)
        .and_then(|scripts| scripts.get("test"))
        .and_then(JsonValue::as_str)
        .is_some_and(|script| command_contains_token(script, runner))
}

fn dependency_declares_runner(package_json: &JsonValue, section: &str, runner: &str) -> bool {
    package_json
        .get(section)
        .and_then(JsonValue::as_object)
        .is_some_and(|deps| deps.contains_key(runner))
}

fn command_contains_token(command: &str, token: &str) -> bool {
    command
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .filter(|part| !part.is_empty())
        .any(|part| part.eq_ignore_ascii_case(token))
}

fn file_contains(path: impl AsRef<Path>, marker: &str) -> bool {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map(|contents| contents.contains(marker))
        .unwrap_or(false)
}

#[derive(Debug)]
struct ChangedFileSymbol {
    symbol: String,
    start_line: u32,
    end_line: u32,
}

fn add_changed_file_target_step(
    changed_file: &str,
    runners: &RecommendationRunners,
    steps_by_command: &mut HashMap<String, VerificationStep>,
) {
    let Some(command) = test_command_for_target(changed_file, runners) else {
        return;
    };
    upsert_verification_step(
        steps_by_command,
        VerificationStep {
            step: command,
            scope: "targeted".to_string(),
            why_included: format!("changed file '{changed_file}' is itself a test target"),
            confidence: "context_high".to_string(),
            score: 0.95,
        },
    );
}

fn add_changed_symbol_target_steps(
    connection: &Connection,
    changed_file: &str,
    changed_lines_by_file: &HashMap<String, Vec<ChangedLineRange>>,
    changed_symbol_filter: &HashSet<String>,
    runners: &RecommendationRunners,
    steps_by_command: &mut HashMap<String, VerificationStep>,
) -> anyhow::Result<()> {
    for symbol in changed_file_symbols(connection, changed_file)? {
        if !include_changed_file_symbol(
            &symbol,
            changed_file,
            changed_lines_by_file,
            changed_symbol_filter,
        ) {
            continue;
        }
        for (target, hit_count) in test_targets_for_symbol(connection, &symbol.symbol)? {
            let Some(command) = test_command_for_target(&target, runners) else {
                continue;
            };
            let (confidence, score) = if hit_count > 1 {
                ("graph_likely", 0.9)
            } else {
                ("context_medium", 0.8)
            };
            upsert_verification_step(
                steps_by_command,
                VerificationStep {
                    step: command,
                    scope: "targeted".to_string(),
                    why_included: format!(
                        "targeted test references changed symbol '{}'",
                        symbol.symbol
                    ),
                    confidence: confidence.to_string(),
                    score,
                },
            );
        }
    }
    Ok(())
}

fn changed_file_symbols(
    connection: &Connection,
    changed_file: &str,
) -> anyhow::Result<Vec<ChangedFileSymbol>> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT symbol, start_line, end_line
         FROM symbols_v2
         WHERE file_path = ?1
         ORDER BY symbol ASC, start_line ASC, end_line ASC",
    )?;
    let rows = statement.query_map(params![changed_file], |row| {
        Ok(ChangedFileSymbol {
            symbol: row.get::<_, String>(0)?,
            start_line: row.get::<_, i64>(1)? as u32,
            end_line: row.get::<_, i64>(2)? as u32,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn include_changed_file_symbol(
    symbol: &ChangedFileSymbol,
    changed_file: &str,
    changed_lines_by_file: &HashMap<String, Vec<ChangedLineRange>>,
    changed_symbol_filter: &HashSet<String>,
) -> bool {
    if !changed_symbol_filter.is_empty() && !changed_symbol_filter.contains(&symbol.symbol) {
        return false;
    }
    if let Some(ranges) = changed_lines_by_file.get(changed_file)
        && !ranges.iter().any(|range| {
            line_range_overlaps(
                symbol.start_line,
                symbol.end_line,
                range.start_line,
                range.end_line,
            )
        })
    {
        return false;
    }
    !is_generic_changed_symbol(&symbol.symbol)
}

fn finalize_targeted_verification_steps(
    steps_by_command: HashMap<String, VerificationStep>,
    targeted_cap: u32,
) -> Vec<VerificationStep> {
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
    let (mut prioritized, non_prioritized): (Vec<_>, Vec<_>) = steps
        .into_iter()
        .partition(|step| is_changed_test_target_reason(&step.why_included));
    prioritized.sort_by(|left, right| {
        left.step
            .cmp(&right.step)
            .then(left.why_included.cmp(&right.why_included))
    });
    prioritized.extend(
        non_prioritized
            .into_iter()
            .take(bounded_usize(targeted_cap)),
    );
    prioritized
}

fn append_full_suite_verification_step(
    steps: &mut Vec<VerificationStep>,
    changed_files: &[String],
    runners: &RecommendationRunners,
) {
    let full_suite_step = select_full_suite_command(steps, changed_files, runners);
    steps.push(VerificationStep {
        step: full_suite_step,
        scope: "full_suite".to_string(),
        why_included: "required safety gate after refactor".to_string(),
        confidence: "context_high".to_string(),
        score: 1.0,
    });
}

fn select_full_suite_command(
    targeted_steps: &[VerificationStep],
    changed_files: &[String],
    runners: &RecommendationRunners,
) -> String {
    if targeted_steps
        .iter()
        .any(|step| step.step.starts_with("cargo test"))
        || changed_files.iter().any(|file| file.ends_with(".rs"))
    {
        return "cargo test".to_string();
    }
    if runners.pytest
        && (targeted_steps
            .iter()
            .any(|step| step.step.starts_with("pytest "))
            || changed_files.iter().any(|file| file.ends_with(".py")))
    {
        return "pytest".to_string();
    }
    if let Some(command) = runners.node.full_suite_command()
        && (targeted_steps
            .iter()
            .any(|step| step.step.starts_with("npx jest --runTestsByPath"))
            || targeted_steps
                .iter()
                .any(|step| step.step.starts_with("npx vitest run "))
            || changed_files
                .iter()
                .any(|file| is_typescript_source_file(file)))
    {
        return command.to_string();
    }
    "cargo test".to_string()
}

fn sort_verification_steps(steps: &mut [VerificationStep]) {
    steps.sort_by(|left, right| {
        verification_scope_rank(&left.scope)
            .cmp(&verification_scope_rank(&right.scope))
            .then(left.step.cmp(&right.step))
            .then(left.why_included.cmp(&right.why_included))
    });
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
         ORDER BY file_path ASC, line ASC, column ASC, symbol ASC",
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

    let mut matches = collect_rows(rows)?;
    matches.dedup_by(|left, right| {
        left.file_path == right.file_path
            && left.line == right.line
            && left.column == right.column
            && left.symbol == right.symbol
    });
    Ok(matches)
}

fn ranked_text_matches(
    connection: &Connection,
    symbol: &str,
    scope: &QueryScope,
) -> anyhow::Result<Vec<QueryMatch>> {
    let mut matches = text_exact_matches(connection, symbol, scope)?;
    matches.extend(text_substring_matches(connection, symbol, scope)?);
    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                fallback_path_class_rank(&left.file_path)
                    .cmp(&fallback_path_class_rank(&right.file_path)),
            )
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.why_matched.cmp(&right.why_matched))
    });
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
        .filter(|item| scope.includes_path(&item.file_path))
        .collect()
}

fn is_code_file_path(file_path: &str) -> bool {
    file_path.ends_with(".rs")
        || file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".py")
        || file_path.ends_with(".go")
}

fn is_test_like_path(file_path: &str) -> bool {
    file_path.starts_with("tests/")
        || file_path.contains("/tests/")
        || Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(is_test_like_file_name)
}

fn is_test_like_file_name(file_name: &str) -> bool {
    file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.py")
        || file_name.ends_with("_tests.py")
        || (file_name.starts_with("test_") && file_name.ends_with(".py"))
        || file_name.ends_with(".test.ts")
        || file_name.ends_with(".test.tsx")
        || file_name.ends_with(".spec.ts")
        || file_name.ends_with(".spec.tsx")
}

fn fallback_path_class_rank(file_path: &str) -> u8 {
    if is_code_file_path(file_path) && !is_test_like_path(file_path) {
        0
    } else if is_test_like_path(file_path) {
        1
    } else {
        2
    }
}

/// Collects all mapped rows into a vector of `QueryMatch`.
///
/// This consumes the provided `MappedRows` iterator, returning a `Vec<QueryMatch>`
/// built from each successful row mapping.
/// Any row-mapping error is returned.
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
/// (paths under `tests/` or common test naming patterns),
/// ordered by `hit_count` descending and then by `file_path` ascending.
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
               OR file_path GLOB '*_test.py'
               OR file_path GLOB '*_tests.py'
               OR file_path LIKE 'test_%.py'
               OR file_path LIKE '%/test_%.py'
               OR file_path LIKE '%.test.ts'
               OR file_path LIKE '%.test.tsx'
               OR file_path LIKE '%.spec.ts'
               OR file_path LIKE '%.spec.tsx'
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

/// Derives a `cargo test` invocation for a standalone test file directly under `tests/`.
///
/// Returns `Some` with command `cargo test --test {stem}` when `target` is a path
/// of the form `tests/<file>` (no additional subdirectories) and the file has
/// extension `.rs`; returns `None` otherwise.
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
fn test_command_for_target(target: &str, runners: &RecommendationRunners) -> Option<String> {
    cargo_test_command_for_target(target)
        .or_else(|| pytest_test_command_for_target(target, runners))
        .or_else(|| node_test_command_for_target(target, runners))
}

fn cargo_test_command_for_target(target: &str) -> Option<String> {
    let file_path = Path::new(target);
    let mut components = file_path.components();
    if components.next()?.as_os_str() != "tests" {
        return None;
    }
    let test_file = Path::new(components.next()?.as_os_str());
    if components.next().is_some() {
        return None;
    }
    if test_file.extension()?.to_str()? != "rs" {
        return None;
    }

    let stem = test_file.file_stem()?.to_str()?;
    Some(format!("cargo test --test {stem}"))
}

fn pytest_test_command_for_target(target: &str, runners: &RecommendationRunners) -> Option<String> {
    if !runners.pytest || !is_pytest_test_file(target) {
        return None;
    }
    Some(format!("pytest {target}"))
}

fn node_test_command_for_target(target: &str, runners: &RecommendationRunners) -> Option<String> {
    if !is_typescript_test_file(target) {
        return None;
    }
    runners.node.targeted_command_for(target)
}

fn is_pytest_test_file(target: &str) -> bool {
    if !target.ends_with(".py") {
        return false;
    }
    Path::new(target)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|file_name| {
            (file_name.starts_with("test_") && file_name.ends_with(".py"))
                || file_name.ends_with("_test.py")
                || file_name.ends_with("_tests.py")
        })
}

fn is_typescript_test_file(target: &str) -> bool {
    target.ends_with(".test.ts")
        || target.ends_with(".test.tsx")
        || target.ends_with(".spec.ts")
        || target.ends_with(".spec.tsx")
}

fn is_typescript_source_file(target: &str) -> bool {
    target.ends_with(".ts") || target.ends_with(".tsx")
}

fn is_runnable_test_target(target: &str, runners: &RecommendationRunners) -> bool {
    test_command_for_target(target, runners).is_some()
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

/// Convert a confidence label into a numeric rank.
/// Larger values indicate stronger confidence.
///
/// # Returns
///
/// `u8` where larger values indicate greater confidence.
/// `3` for `"graph_likely"`, `2` for `"context_high"`, `1` for `"context_medium"`,
/// and `0` for any other input.
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
