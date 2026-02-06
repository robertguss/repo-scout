use std::path::Path;

use rusqlite::{Connection, params};

#[derive(Debug, Clone)]
pub struct QueryMatch {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub symbol: String,
}

pub fn text_matches(db_path: &Path, symbol: &str) -> anyhow::Result<Vec<QueryMatch>> {
    let connection = Connection::open(db_path)?;
    let mut statement = connection.prepare(
        "SELECT file_path, line, column, symbol
         FROM text_occurrences
         WHERE symbol = ?1
         ORDER BY file_path ASC, line ASC, column ASC",
    )?;
    let rows = statement.query_map(params![symbol], |row| {
        Ok(QueryMatch {
            file_path: row.get(0)?,
            line: row.get::<_, i64>(1)? as u32,
            column: row.get::<_, i64>(2)? as u32,
            symbol: row.get(3)?,
        })
    })?;

    let mut matches = Vec::new();
    for row in rows {
        matches.push(row?);
    }
    Ok(matches)
}
