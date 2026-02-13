use std::path::Path;

use rusqlite::{Connection, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BoundarySymbol {
    pub symbol: String,
    pub kind: String,
    pub is_public: bool,
    pub external_references: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoundaryReport {
    pub file_path: String,
    pub public_symbols: Vec<BoundarySymbol>,
    pub internal_symbols: Vec<BoundarySymbol>,
}

pub fn boundary_analysis(db_path: &Path, file_path: &str) -> anyhow::Result<BoundaryReport> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT s.symbol,
                s.kind,
                CASE
                    WHEN s.visibility IN ('public', 'pub', 'export') THEN 1
                    WHEN s.signature LIKE 'pub %' OR s.signature LIKE 'pub(%' THEN 1
                    ELSE 0
                END AS is_public,
                SUM(
                    CASE
                        WHEN caller.file_path IS NOT NULL AND caller.file_path != s.file_path THEN 1
                        ELSE 0
                    END
                ) AS external_refs
         FROM symbols_v2 s
         LEFT JOIN symbol_edges_v2 e ON e.to_symbol_id = s.symbol_id
         LEFT JOIN symbols_v2 caller ON caller.symbol_id = e.from_symbol_id
         WHERE s.file_path = ?1
         GROUP BY s.symbol_id
         ORDER BY is_public DESC, external_refs DESC, s.symbol ASC",
    )?;
    let rows = stmt.query_map(params![file_path], |row| {
        Ok(BoundarySymbol {
            symbol: row.get(0)?,
            kind: row.get(1)?,
            is_public: row.get::<_, u32>(2)? > 0,
            external_references: row.get(3)?,
        })
    })?;

    let mut public_symbols = Vec::new();
    let mut internal_symbols = Vec::new();
    for row in rows {
        let symbol = row?;
        if symbol.is_public {
            public_symbols.push(symbol);
        } else {
            internal_symbols.push(symbol);
        }
    }

    Ok(BoundaryReport {
        file_path: file_path.to_string(),
        public_symbols,
        internal_symbols,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractCheckReport {
    pub symbol: String,
    pub file_path: String,
    pub function_start_line: u32,
    pub function_end_line: u32,
    pub extract_start_line: u32,
    pub extract_end_line: u32,
    pub estimated_line_count: u32,
    pub signature: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

pub fn parse_line_range(input: &str) -> anyhow::Result<LineRange> {
    let mut parts = input.splitn(2, '-');
    let start_raw = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid --lines format: expected <start>-<end>"))?;
    let end_raw = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid --lines format: expected <start>-<end>"))?;
    let start = start_raw
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --lines start: '{start_raw}'"))?;
    let end = end_raw
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("invalid --lines end: '{end_raw}'"))?;
    if start == 0 || end == 0 {
        anyhow::bail!("invalid --lines range: values must be >= 1");
    }
    if start > end {
        anyhow::bail!("invalid --lines range: start must be <= end");
    }
    Ok(LineRange { start, end })
}

pub fn extract_check(
    db_path: &Path,
    symbol: &str,
    range: LineRange,
) -> anyhow::Result<ExtractCheckReport> {
    let connection = Connection::open(db_path)?;
    let mut stmt = connection.prepare(
        "SELECT file_path, start_line, end_line, signature
         FROM symbols_v2
         WHERE symbol = ?1 AND kind = 'function'
         ORDER BY file_path ASC, start_line ASC
         LIMIT 1",
    )?;
    let row = stmt
        .query_row(params![symbol], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, u32>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|_| anyhow::anyhow!("symbol '{symbol}' not found as a function"))?;

    let (file_path, function_start_line, function_end_line, signature) = row;
    if range.start < function_start_line || range.end > function_end_line {
        anyhow::bail!(
            "extract range {}-{} is outside function bounds {}-{}",
            range.start,
            range.end,
            function_start_line,
            function_end_line
        );
    }

    let mut warnings = Vec::new();
    if range.start == function_start_line && range.end == function_end_line {
        warnings.push("requested range covers entire function".to_string());
    }
    let estimated_line_count = range.end.saturating_sub(range.start).saturating_add(1);
    if estimated_line_count < 2 {
        warnings.push("range is very small; extraction may not be worthwhile".to_string());
    }

    Ok(ExtractCheckReport {
        symbol: symbol.to_string(),
        file_path,
        function_start_line,
        function_end_line,
        extract_start_line: range.start,
        extract_end_line: range.end,
        estimated_line_count,
        signature,
        warnings,
    })
}
