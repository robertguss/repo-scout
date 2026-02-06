use anyhow::Context;
use rusqlite::{Connection, OptionalExtension};

pub const SCHEMA_VERSION: i64 = 1;

pub fn bootstrap_schema(connection: &Connection) -> anyhow::Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS symbols (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS refs (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS indexed_files (
            file_path TEXT PRIMARY KEY,
            content_hash TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS text_occurrences (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL,
            line INTEGER NOT NULL,
            column INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS ast_definitions (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL,
            kind TEXT NOT NULL,
            line INTEGER NOT NULL,
            column INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS ast_references (
            id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL,
            line INTEGER NOT NULL,
            column INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_text_occurrences_symbol
            ON text_occurrences(symbol);
        CREATE INDEX IF NOT EXISTS idx_text_occurrences_file
            ON text_occurrences(file_path);
        CREATE INDEX IF NOT EXISTS idx_ast_definitions_symbol
            ON ast_definitions(symbol);
        CREATE INDEX IF NOT EXISTS idx_ast_references_symbol
            ON ast_references(symbol);
        "#,
    )?;

    connection.execute(
        "INSERT OR REPLACE INTO meta(key, value) VALUES('schema_version', ?1)",
        [SCHEMA_VERSION.to_string()],
    )?;

    Ok(())
}

pub fn read_schema_version(connection: &Connection) -> anyhow::Result<i64> {
    let value = connection
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    let Some(value) = value else {
        anyhow::bail!("schema_version missing in meta table");
    };

    let parsed = value
        .parse::<i64>()
        .with_context(|| format!("invalid schema_version value in database: {value}"))?;
    Ok(parsed)
}
