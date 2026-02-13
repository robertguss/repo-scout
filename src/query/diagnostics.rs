use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealth {
    pub file_path: String,
    pub line_count: u32,
    pub symbol_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionHealth {
    pub file_path: String,
    pub symbol: String,
    pub line_count: u32,
    pub start_line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub largest_files: Vec<FileHealth>,
    pub largest_functions: Vec<FunctionHealth>,
}

pub fn health_report(db_path: &Path, top_n: u32, threshold: u32) -> anyhow::Result<HealthReport> {
    let connection = Connection::open(db_path)?;

    let largest_files = {
        let mut stmt = connection.prepare(
            "SELECT f.file_path, f.line_count, COUNT(s.symbol_id) as sym_count
             FROM indexed_files f
             LEFT JOIN symbols_v2 s ON f.file_path = s.file_path
             WHERE f.line_count IS NOT NULL AND f.line_count >= ?1
             GROUP BY f.file_path
             ORDER BY f.line_count DESC, f.file_path ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![threshold, top_n], |row| {
            Ok(FileHealth {
                file_path: row.get(0)?,
                line_count: row.get(1)?,
                symbol_count: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    let largest_functions = {
        let mut stmt = connection.prepare(
            "SELECT file_path, symbol, line_count, start_line
             FROM symbols_v2
             WHERE kind = 'function' AND line_count IS NOT NULL AND line_count >= ?1
             ORDER BY line_count DESC, file_path ASC, symbol ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![threshold, top_n], |row| {
            Ok(FunctionHealth {
                file_path: row.get(0)?,
                symbol: row.get(1)?,
                line_count: row.get(2)?,
                start_line: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(HealthReport {
        largest_files,
        largest_functions,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CircularReport {
    pub cycles: Vec<CycleDep>,
    pub total_cycles: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnatomySymbol {
    pub symbol: String,
    pub kind: String,
    pub start_line: u32,
    pub line_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnatomyReport {
    pub file_path: String,
    pub total_symbols: u32,
    pub function_count: u32,
    pub symbols: Vec<AnatomySymbol>,
}

pub fn file_anatomy(db_path: &Path, file_path: &str) -> anyhow::Result<AnatomyReport> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT symbol, kind, start_line, line_count
         FROM symbols_v2
         WHERE file_path = ?1
         ORDER BY start_line ASC, symbol ASC",
    )?;
    let rows = stmt.query_map(params![file_path], |row| {
        Ok(AnatomySymbol {
            symbol: row.get(0)?,
            kind: row.get(1)?,
            start_line: row.get(2)?,
            line_count: row.get(3)?,
        })
    })?;
    let symbols = rows.collect::<Result<Vec<_>, _>>()?;
    let total_symbols = u32::try_from(symbols.len()).unwrap_or(u32::MAX);
    let function_count =
        u32::try_from(symbols.iter().filter(|s| s.kind == "function").count()).unwrap_or(u32::MAX);

    Ok(AnatomyReport {
        file_path: file_path.to_string(),
        total_symbols,
        function_count,
        symbols,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CouplingEntry {
    pub file_a: String,
    pub file_b: String,
    pub a_to_b_edges: u32,
    pub b_to_a_edges: u32,
    pub total_edges: u32,
}

pub fn coupling_report(db_path: &Path, limit: u32) -> anyhow::Result<Vec<CouplingEntry>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "WITH file_edges AS (
            SELECT src.file_path AS from_file, tgt.file_path AS to_file, COUNT(*) AS edge_count
            FROM symbol_edges_v2 e
            JOIN symbols_v2 src ON src.symbol_id = e.from_symbol_id
            JOIN symbols_v2 tgt ON tgt.symbol_id = e.to_symbol_id
            WHERE src.file_path != tgt.file_path
            GROUP BY src.file_path, tgt.file_path
        )
        SELECT e1.from_file, e1.to_file, e1.edge_count, COALESCE(e2.edge_count, 0) AS reverse_count
        FROM file_edges e1
        LEFT JOIN file_edges e2
               ON e2.from_file = e1.to_file
              AND e2.to_file = e1.from_file
        WHERE e1.from_file < e1.to_file
        ORDER BY (e1.edge_count + COALESCE(e2.edge_count, 0)) DESC,
                 e1.from_file ASC,
                 e1.to_file ASC
        LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        let a_to_b_edges: u32 = row.get(2)?;
        let b_to_a_edges: u32 = row.get(3)?;
        Ok(CouplingEntry {
            file_a: row.get(0)?,
            file_b: row.get(1)?,
            a_to_b_edges,
            b_to_a_edges,
            total_edges: a_to_b_edges.saturating_add(b_to_a_edges),
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[derive(Debug, Clone, Serialize)]
pub struct DeadSymbol {
    pub file_path: String,
    pub symbol: String,
    pub kind: String,
    pub line: u32,
}

pub fn dead_symbols(db_path: &Path) -> anyhow::Result<Vec<DeadSymbol>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT s.file_path, s.symbol, s.kind, s.start_line
         FROM symbols_v2 s
         LEFT JOIN symbol_edges_v2 e ON e.to_symbol_id = s.symbol_id
         WHERE e.to_symbol_id IS NULL
           AND s.kind IN ('function', 'struct', 'enum', 'trait')
         ORDER BY s.file_path ASC, s.start_line ASC, s.symbol ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(DeadSymbol {
            file_path: row.get(0)?,
            symbol: row.get(1)?,
            kind: row.get(2)?,
            line: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[derive(Debug, Clone, Serialize)]
pub struct TestGapEntry {
    pub symbol: String,
    pub line_count: u32,
    pub test_hits: u32,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestGapReport {
    pub target: String,
    pub covered: Vec<TestGapEntry>,
    pub uncovered: Vec<TestGapEntry>,
}

pub fn test_gap_analysis(db_path: &Path, target: &str) -> anyhow::Result<TestGapReport> {
    let connection = Connection::open(db_path)?;
    let is_file_target = target.contains('/');
    let file_filter = if is_file_target { Some(target) } else { None };
    let symbol_filter = if is_file_target { None } else { Some(target) };
    let mut entries = Vec::new();

    let sql = "SELECT s.symbol,
                      COALESCE(s.line_count, s.end_line - s.start_line + 1) AS line_count,
                      COUNT(DISTINCT t.file_path) AS test_hits
               FROM symbols_v2 s
               LEFT JOIN text_occurrences t
                      ON t.symbol = s.symbol
                     AND (t.file_path LIKE 'tests/%' OR t.file_path LIKE '%_test.%')
               WHERE s.kind = 'function'
                 AND (?1 IS NULL OR s.file_path = ?1)
                 AND (?2 IS NULL OR s.symbol = ?2)
               GROUP BY s.symbol, line_count
               ORDER BY s.symbol ASC";
    let mut stmt = connection.prepare(sql)?;
    let rows = stmt.query_map(params![file_filter, symbol_filter], |row| {
        let line_count: u32 = row.get(1)?;
        let risk = if line_count > 80 {
            "high"
        } else if line_count > 30 {
            "medium"
        } else {
            "low"
        };
        Ok(TestGapEntry {
            symbol: row.get(0)?,
            line_count,
            test_hits: row.get(2)?,
            risk: risk.to_string(),
        })
    })?;
    for row in rows {
        entries.push(row?);
    }

    let mut covered = Vec::new();
    let mut uncovered = Vec::new();
    for entry in entries {
        if entry.test_hits > 0 {
            covered.push(entry);
        } else {
            uncovered.push(entry);
        }
    }

    Ok(TestGapReport {
        target: target.to_string(),
        covered,
        uncovered,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub symbol: String,
    pub file_path: String,
    pub line_count: u32,
    pub fan_in: u32,
    pub has_tests: bool,
    pub refactoring_value: f64,
}

pub fn suggest_refactorings(
    db_path: &Path,
    top: u32,
    safe_only: bool,
    min_score: Option<f64>,
) -> anyhow::Result<Vec<Suggestion>> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT s.file_path,
                s.symbol,
                COALESCE(s.line_count, s.end_line - s.start_line + 1) AS line_count,
                COUNT(DISTINCT incoming.edge_id) AS fan_in,
                CASE
                    WHEN COUNT(DISTINCT tests.file_path) > 0 THEN 1
                    ELSE 0
                END AS has_tests
         FROM symbols_v2 s
         LEFT JOIN symbol_edges_v2 incoming ON incoming.to_symbol_id = s.symbol_id
         LEFT JOIN text_occurrences tests
                ON tests.symbol = s.symbol
               AND (tests.file_path LIKE 'tests/%' OR tests.file_path LIKE '%_test.%')
         WHERE s.kind = 'function'
         GROUP BY s.symbol_id
         HAVING line_count >= 10
         ORDER BY line_count DESC, fan_in DESC, s.file_path ASC, s.symbol ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let line_count: u32 = row.get(2)?;
        let fan_in: u32 = row.get(3)?;
        let has_tests = row.get::<_, u32>(4)? > 0;
        let test_penalty = if has_tests { 0.0 } else { 20.0 };
        let refactoring_value = f64::from(line_count) + f64::from(fan_in) * 5.0 + test_penalty;
        Ok(Suggestion {
            file_path: row.get(0)?,
            symbol: row.get(1)?,
            line_count,
            fan_in,
            has_tests,
            refactoring_value,
        })
    })?;

    let mut suggestions = Vec::new();
    for row in rows {
        let suggestion = row?;
        if safe_only && !suggestion.has_tests {
            continue;
        }
        if let Some(min_score_value) = min_score
            && suggestion.refactoring_value < min_score_value
        {
            continue;
        }
        suggestions.push(suggestion);
    }
    suggestions.sort_by(|a, b| {
        b.refactoring_value
            .partial_cmp(&a.refactoring_value)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.file_path.cmp(&b.file_path))
            .then(a.symbol.cmp(&b.symbol))
    });
    suggestions.truncate(usize::try_from(top).unwrap_or(usize::MAX));
    Ok(suggestions)
}

#[derive(Debug, Clone, Serialize)]
pub struct CycleDep {
    pub files: Vec<String>,
    pub edges: Vec<CycleEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CycleEdge {
    pub from_file: String,
    pub from_symbol: String,
    pub to_file: String,
    pub to_symbol: String,
    pub edge_kind: String,
}

pub fn detect_circular_deps(db_path: &Path, max_length: u32) -> anyhow::Result<CircularReport> {
    let connection = Connection::open(db_path)?;

    // Step 1: Build directed file-level adjacency list
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    {
        let mut stmt = connection.prepare(
            "SELECT DISTINCT src_sym.file_path, tgt_sym.file_path
             FROM symbol_edges_v2 e
             JOIN symbols_v2 src_sym ON e.from_symbol_id = src_sym.symbol_id
             JOIN symbols_v2 tgt_sym ON e.to_symbol_id = tgt_sym.symbol_id
             WHERE src_sym.file_path != tgt_sym.file_path
             ORDER BY src_sym.file_path ASC, tgt_sym.file_path ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (from, to) = row?;
            adj.entry(from).or_default().push(to);
        }
    }

    // Step 2: Tarjan's SCC
    let sccs = tarjan_scc(&adj)?;

    // Step 3: Filter to multi-file SCCs within max_length
    let mut cycles = Vec::new();
    for mut scc in sccs {
        if scc.len() < 2 || scc.len() as u32 > max_length {
            continue;
        }
        scc.sort();

        // Step 4: Get the symbol-level edges within this SCC
        let placeholders: Vec<String> = (1..=scc.len()).map(|i| format!("?{i}")).collect();
        let placeholder_list = placeholders.join(", ");
        let sql = format!(
            "SELECT src_sym.file_path, src_sym.symbol, tgt_sym.file_path, tgt_sym.symbol, e.edge_kind
             FROM symbol_edges_v2 e
             JOIN symbols_v2 src_sym ON e.from_symbol_id = src_sym.symbol_id
             JOIN symbols_v2 tgt_sym ON e.to_symbol_id = tgt_sym.symbol_id
             WHERE src_sym.file_path IN ({placeholder_list}) AND tgt_sym.file_path IN ({placeholder_list})
               AND src_sym.file_path != tgt_sym.file_path
             ORDER BY src_sym.file_path ASC, src_sym.symbol ASC, tgt_sym.file_path ASC, tgt_sym.symbol ASC"
        );
        let mut stmt = connection.prepare(&sql)?;
        // Bind the SCC files twice (for both IN clauses)
        let mut edge_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        for file in &scc {
            edge_params.push(Box::new(file.clone()));
        }
        let edges_rows = stmt.query_map(
            rusqlite::params_from_iter(edge_params.iter().map(|p| p.as_ref())),
            |row| {
                Ok(CycleEdge {
                    from_file: row.get(0)?,
                    from_symbol: row.get(1)?,
                    to_file: row.get(2)?,
                    to_symbol: row.get(3)?,
                    edge_kind: row.get(4)?,
                })
            },
        )?;
        let edges: Vec<CycleEdge> = edges_rows.filter_map(|r| r.ok()).collect();

        cycles.push(CycleDep { files: scc, edges });
    }

    cycles.sort_by(|a, b| {
        a.files
            .len()
            .cmp(&b.files.len())
            .then(a.files.cmp(&b.files))
    });
    let total_cycles = cycles.len();

    Ok(CircularReport {
        cycles,
        total_cycles,
    })
}

/// Tarjan's Strongly Connected Components algorithm.
fn tarjan_scc(adj: &HashMap<String, Vec<String>>) -> anyhow::Result<Vec<Vec<String>>> {
    let mut index_counter: u32 = 0;
    let mut stack: Vec<String> = Vec::new();
    let mut on_stack: HashMap<String, bool> = HashMap::new();
    let mut index: HashMap<String, u32> = HashMap::new();
    let mut lowlink: HashMap<String, u32> = HashMap::new();
    let mut result: Vec<Vec<String>> = Vec::new();

    // Collect all nodes (both sources and targets)
    let mut all_nodes: Vec<String> = adj.keys().cloned().collect();
    for targets in adj.values() {
        for t in targets {
            if !adj.contains_key(t) {
                all_nodes.push(t.clone());
            }
        }
    }
    all_nodes.sort();
    all_nodes.dedup();

    fn strongconnect(
        v: &str,
        adj: &HashMap<String, Vec<String>>,
        index_counter: &mut u32,
        stack: &mut Vec<String>,
        on_stack: &mut HashMap<String, bool>,
        index: &mut HashMap<String, u32>,
        lowlink: &mut HashMap<String, u32>,
        result: &mut Vec<Vec<String>>,
    ) -> anyhow::Result<()> {
        index.insert(v.to_string(), *index_counter);
        lowlink.insert(v.to_string(), *index_counter);
        *index_counter += 1;
        stack.push(v.to_string());
        on_stack.insert(v.to_string(), true);

        if let Some(neighbors) = adj.get(v) {
            for w in neighbors {
                if !index.contains_key(w.as_str()) {
                    strongconnect(
                        w,
                        adj,
                        index_counter,
                        stack,
                        on_stack,
                        index,
                        lowlink,
                        result,
                    )?;
                    let w_low = lowlink[w.as_str()];
                    let v_low = lowlink[v];
                    if w_low < v_low {
                        lowlink.insert(v.to_string(), w_low);
                    }
                } else if on_stack.get(w.as_str()).copied().unwrap_or(false) {
                    let w_idx = index[w.as_str()];
                    let v_low = lowlink[v];
                    if w_idx < v_low {
                        lowlink.insert(v.to_string(), w_idx);
                    }
                }
            }
        }

        if lowlink[v] == index[v] {
            let mut component = Vec::new();
            loop {
                let w = match stack.pop() {
                    Some(w) => w,
                    None => {
                        return Err(anyhow::anyhow!(
                            "Tarjan invariant violated: stack exhausted while processing component rooted at '{v}'"
                        ));
                    }
                };
                on_stack.insert(w.clone(), false);
                component.push(w.clone());
                if w == v {
                    break;
                }
            }
            result.push(component);
        }
        Ok(())
    }

    for node in &all_nodes {
        if !index.contains_key(node.as_str()) {
            strongconnect(
                node,
                adj,
                &mut index_counter,
                &mut stack,
                &mut on_stack,
                &mut index,
                &mut lowlink,
                &mut result,
            )?;
        }
    }

    Ok(result)
}
