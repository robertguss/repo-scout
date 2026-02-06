use std::collections::HashSet;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

pub mod files;
pub mod rust_ast;
pub mod text;

#[derive(Debug)]
pub struct IndexSummary {
    pub indexed_files: usize,
    pub skipped_files: usize,
}

pub fn index_repository(repo: &Path, db_path: &Path) -> anyhow::Result<IndexSummary> {
    let mut connection = Connection::open(db_path)?;
    let source_files = files::discover_source_files(repo)?;
    let live_paths: HashSet<String> = source_files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect();

    prune_stale_file_rows(&mut connection, &live_paths)?;

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
        let ast_items = if file.relative_path.ends_with(".rs") {
            text_content
                .map(rust_ast::extract_rust_items)
                .transpose()?
                .unwrap_or_default()
        } else {
            (Vec::new(), Vec::new())
        };

        let tx = connection.transaction()?;
        tx.execute(
            "DELETE FROM text_occurrences WHERE file_path = ?1",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM ast_definitions WHERE file_path = ?1",
            [&file.relative_path],
        )?;
        tx.execute(
            "DELETE FROM ast_references WHERE file_path = ?1",
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

        for definition in ast_items.0 {
            tx.execute(
                "INSERT INTO ast_definitions(file_path, symbol, kind, line, column)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    file.relative_path,
                    definition.symbol,
                    definition.kind,
                    i64::from(definition.line),
                    i64::from(definition.column)
                ],
            )?;
        }

        for reference in ast_items.1 {
            tx.execute(
                "INSERT INTO ast_references(file_path, symbol, line, column)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    file.relative_path,
                    reference.symbol,
                    i64::from(reference.line),
                    i64::from(reference.column)
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

fn prune_stale_file_rows(
    connection: &mut Connection,
    live_paths: &HashSet<String>,
) -> anyhow::Result<()> {
    let stale_paths = {
        let mut statement =
            connection.prepare("SELECT file_path FROM indexed_files ORDER BY file_path ASC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;

        let mut stale_paths = Vec::new();
        for row in rows {
            let path = row?;
            if !live_paths.contains(&path) {
                stale_paths.push(path);
            }
        }
        stale_paths
    };

    if stale_paths.is_empty() {
        return Ok(());
    }

    let tx = connection.transaction()?;
    for path in stale_paths {
        tx.execute("DELETE FROM text_occurrences WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM ast_definitions WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM ast_references WHERE file_path = ?1", [&path])?;
        tx.execute("DELETE FROM indexed_files WHERE file_path = ?1", [&path])?;
    }
    tx.commit()?;

    Ok(())
}
