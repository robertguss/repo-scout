use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileHealth {
    pub file_path: String,
    pub line_count: u32,
    pub symbol_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionHealth {
    pub file_path: String,
    pub symbol: String,
    pub line_count: u32,
    pub start_line: u32,
}

#[derive(Debug, Clone, Serialize)]
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

        cycles.push(CycleDep {
            files: scc,
            edges,
        });
    }

    cycles.sort_by(|a, b| a.files.len().cmp(&b.files.len()).then(a.files.cmp(&b.files)));
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
                    strongconnect(w, adj, index_counter, stack, on_stack, index, lowlink, result)?;
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
