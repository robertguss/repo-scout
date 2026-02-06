use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

pub mod files;
pub mod text;

#[derive(Debug)]
pub struct IndexSummary {
    pub indexed_files: usize,
    pub skipped_files: usize,
}

pub fn index_repository(repo: &Path, db_path: &Path) -> anyhow::Result<IndexSummary> {
    let mut connection = Connection::open(db_path)?;
    let source_files = files::discover_source_files(repo)?;

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

        let tx = connection.transaction()?;
        tx.execute(
            "DELETE FROM text_occurrences WHERE file_path = ?1",
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
