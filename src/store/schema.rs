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
