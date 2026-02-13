use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TreeReport {
    pub root: TreeNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub name: String,
    pub kind: TreeNodeKind,
    pub line_count: Option<u32>,
    pub symbol_count: u32,
    pub children: Vec<TreeNode>,
    pub imports: Vec<String>,
    pub used_by: Vec<String>,
    pub total_files: u32,
    pub total_symbols: u32,
    pub symbols: Vec<TreeSymbol>,
}

#[derive(Debug, Clone, Serialize)]
pub enum TreeNodeKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize)]
pub struct TreeSymbol {
    pub name: String,
    pub kind: String,
    pub start_line: u32,
}

pub struct TreeReportArgs {
    pub depth: u32,
    pub no_deps: bool,
    pub focus: Option<String>,
    pub show_symbols: bool,
}

pub fn tree_report(db_path: &Path, args: &TreeReportArgs) -> anyhow::Result<TreeReport> {
    let connection = Connection::open(db_path)?;

    // 1. Query all indexed files with line counts and symbol counts
    let mut file_stats: BTreeMap<String, (Option<u32>, u32)> = BTreeMap::new();
    {
        let mut stmt = connection.prepare(
            "SELECT f.file_path, f.line_count, COUNT(s.symbol_id) as sym_count
             FROM indexed_files f
             LEFT JOIN symbols_v2 s ON f.file_path = s.file_path
             GROUP BY f.file_path
             ORDER BY f.file_path ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let fp: String = row.get(0)?;
            let lc: Option<u32> = row.get(1)?;
            let sc: u32 = row.get(2)?;
            Ok((fp, lc, sc))
        })?;
        for row in rows {
            let (fp, lc, sc) = row?;
            file_stats.insert(fp, (lc, sc));
        }
    }

    // 2. Query file-level dependency edges (unless --no-deps)
    let mut imports_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut used_by_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if !args.no_deps {
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
            imports_map.entry(from.clone()).or_default().push(to.clone());
            used_by_map.entry(to).or_default().push(from);
        }
    }

    // 3. Query symbols per file (if --symbols)
    let mut symbols_map: BTreeMap<String, Vec<TreeSymbol>> = BTreeMap::new();
    if args.show_symbols {
        let mut stmt = connection.prepare(
            "SELECT file_path, symbol, kind, start_line
             FROM symbols_v2
             ORDER BY file_path ASC, start_line ASC, symbol ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                TreeSymbol {
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    start_line: row.get(3)?,
                },
            ))
        })?;
        for row in rows {
            let (fp, sym) = row?;
            symbols_map.entry(fp).or_default().push(sym);
        }
    }

    // 4. Build tree from file paths
    let mut root = TreeNode {
        name: ".".to_string(),
        kind: TreeNodeKind::Directory,
        line_count: None,
        symbol_count: 0,
        children: Vec::new(),
        imports: Vec::new(),
        used_by: Vec::new(),
        total_files: 0,
        total_symbols: 0,
        symbols: Vec::new(),
    };

    // Apply focus filter
    let files_to_include: Vec<&String> = if let Some(ref focus) = args.focus {
        let focus_prefix = if focus.ends_with('/') {
            focus.clone()
        } else {
            format!("{focus}/")
        };
        file_stats
            .keys()
            .filter(|fp| fp.starts_with(&focus_prefix) || *fp == focus)
            .collect()
    } else {
        file_stats.keys().collect()
    };

    for file_path in &files_to_include {
        let (line_count, symbol_count) = file_stats[file_path.as_str()];
        let parts: Vec<&str> = file_path.split('/').collect();
        insert_path(
            &mut root,
            &parts,
            line_count,
            symbol_count,
            imports_map.get(file_path.as_str()).cloned().unwrap_or_default(),
            used_by_map.get(file_path.as_str()).cloned().unwrap_or_default(),
            symbols_map.get(file_path.as_str()).cloned().unwrap_or_default(),
        );
    }

    // 5. Aggregate directory stats
    aggregate_stats(&mut root);

    // 6. Apply depth truncation
    truncate_depth(&mut root, 0, args.depth);

    Ok(TreeReport { root })
}

fn insert_path(
    node: &mut TreeNode,
    parts: &[&str],
    line_count: Option<u32>,
    symbol_count: u32,
    imports: Vec<String>,
    used_by: Vec<String>,
    symbols: Vec<TreeSymbol>,
) {
    if parts.is_empty() {
        return;
    }
    if parts.len() == 1 {
        // Leaf file
        node.children.push(TreeNode {
            name: parts[0].to_string(),
            kind: TreeNodeKind::File,
            line_count,
            symbol_count,
            children: Vec::new(),
            imports,
            used_by,
            total_files: 1,
            total_symbols: symbol_count,
            symbols,
        });
        return;
    }

    // Find or create directory node
    let dir_name = parts[0];
    let child_pos = node.children.iter().position(|c| c.name == dir_name);
    let child = if let Some(pos) = child_pos {
        &mut node.children[pos]
    } else {
        node.children.push(TreeNode {
            name: dir_name.to_string(),
            kind: TreeNodeKind::Directory,
            line_count: None,
            symbol_count: 0,
            children: Vec::new(),
            imports: Vec::new(),
            used_by: Vec::new(),
            total_files: 0,
            total_symbols: 0,
            symbols: Vec::new(),
        });
        node.children.last_mut().unwrap()
    };

    insert_path(child, &parts[1..], line_count, symbol_count, imports, used_by, symbols);
}

fn aggregate_stats(node: &mut TreeNode) -> (u32, u32) {
    match node.kind {
        TreeNodeKind::File => (1, node.symbol_count),
        TreeNodeKind::Directory => {
            let mut total_files = 0u32;
            let mut total_symbols = 0u32;
            for child in &mut node.children {
                let (f, s) = aggregate_stats(child);
                total_files += f;
                total_symbols += s;
            }
            node.total_files = total_files;
            node.total_symbols = total_symbols;
            // Sort children: directories first, then files, all alphabetical
            node.children.sort_by(|a, b| {
                let a_is_dir = matches!(a.kind, TreeNodeKind::Directory);
                let b_is_dir = matches!(b.kind, TreeNodeKind::Directory);
                b_is_dir.cmp(&a_is_dir).then(a.name.cmp(&b.name))
            });
            (total_files, total_symbols)
        }
    }
}

fn truncate_depth(node: &mut TreeNode, current_depth: u32, max_depth: u32) {
    if current_depth >= max_depth {
        node.children.clear();
        return;
    }
    for child in &mut node.children {
        truncate_depth(child, current_depth + 1, max_depth);
    }
}

// --- Orient report ---

use super::diagnostics::{CircularReport, HealthReport, detect_circular_deps, health_report};
use super::{HotspotEntry, hotspots};

#[derive(Debug, Clone, Serialize)]
pub struct OrientReport {
    pub tree: TreeReport,
    pub health: HealthReport,
    pub hotspots: Vec<HotspotEntry>,
    pub circular: CircularReport,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub kind: RecommendationKind,
    pub message: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum RecommendationKind {
    StartExploring,
    CarefulAround,
    CycleWarning,
}

pub struct OrientReportArgs {
    pub depth: u32,
    pub top: u32,
}

pub fn orient_report(db_path: &Path, args: &OrientReportArgs) -> anyhow::Result<OrientReport> {
    let tree = tree_report(
        db_path,
        &TreeReportArgs {
            depth: args.depth,
            no_deps: false,
            focus: None,
            show_symbols: false,
        },
    )?;
    let health = health_report(db_path, args.top, 0)?;
    let hotspot_entries = hotspots(db_path, 10)?;
    let circular = detect_circular_deps(db_path, 10)?;

    let recommendations = generate_recommendations(&health, &hotspot_entries, &circular);

    Ok(OrientReport {
        tree,
        health,
        hotspots: hotspot_entries,
        circular,
        recommendations,
    })
}

fn generate_recommendations(
    health: &HealthReport,
    hotspot_entries: &[HotspotEntry],
    circular: &CircularReport,
) -> Vec<Recommendation> {
    let mut recs = Vec::new();

    // Entry point suggestion: file with most outbound edges (hotspot with highest fan_out)
    if let Some(entry) = hotspot_entries.iter().max_by_key(|h| h.fan_out) {
        recs.push(Recommendation {
            kind: RecommendationKind::StartExploring,
            message: format!(
                "Start exploring: {} (entry point, {} symbols)",
                entry.file_path,
                entry.fan_out
            ),
            file_path: Some(entry.file_path.clone()),
        });
    }

    // "Careful around" warning: files that are both large and high fan-in
    let large_files: std::collections::HashSet<&str> = health
        .largest_files
        .iter()
        .take(5)
        .map(|f| f.file_path.as_str())
        .collect();
    let mut warned_files: std::collections::HashSet<String> = std::collections::HashSet::new();
    for hotspot in hotspot_entries.iter().take(10) {
        if large_files.contains(hotspot.file_path.as_str())
            && hotspot.fan_in > 0
            && warned_files.insert(hotspot.file_path.clone())
        {
            recs.push(Recommendation {
                kind: RecommendationKind::CarefulAround,
                message: format!(
                    "Careful around: {} (large file, high fan-in)",
                    hotspot.file_path
                ),
                file_path: Some(hotspot.file_path.clone()),
            });
        }
    }

    // Cycle warnings
    if circular.total_cycles > 0 {
        recs.push(Recommendation {
            kind: RecommendationKind::CycleWarning,
            message: format!(
                "{} circular dependency cycle{} detected",
                circular.total_cycles,
                if circular.total_cycles == 1 { "" } else { "s" }
            ),
            file_path: None,
        });
    }

    recs
}
