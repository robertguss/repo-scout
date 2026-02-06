use anyhow::Context;
use rusqlite::{Connection, OptionalExtension};

pub const SCHEMA_VERSION: i64 = 3;

/// Creates the database schema and records the current schema version.
///
/// This creates the necessary tables (meta, symbols, refs, indexed_files,
/// text_occurrences, ast_definitions, ast_references, symbols_v2,
/// symbol_edges_v2) and their associated indices if they do not already
/// exist, then writes `SCHEMA_VERSION` into the `meta` table under the key
/// `schema_version`.
///
/// # Examples
///
/// ```
/// use rusqlite::Connection;
/// # use anyhow::Result;
/// # fn run() -> Result<()> {
/// let conn = Connection::open_in_memory()?;
/// bootstrap_schema(&conn)?;
/// let value: String = conn.query_row(
///     "SELECT value FROM meta WHERE key = 'schema_version'",
///     [],
///     |row| row.get(0),
/// )?;
/// assert_eq!(value.parse::<i64>().unwrap(), SCHEMA_VERSION);
/// # Ok(()) }
/// # run().unwrap();
/// ```
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
        CREATE TABLE IF NOT EXISTS symbols_v2 (
            symbol_id INTEGER PRIMARY KEY,
            file_path TEXT NOT NULL,
            symbol TEXT NOT NULL,
            kind TEXT NOT NULL,
            language TEXT NOT NULL DEFAULT 'unknown',
            qualified_symbol TEXT,
            container TEXT,
            start_line INTEGER NOT NULL,
            start_column INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            end_column INTEGER NOT NULL,
            signature TEXT,
            UNIQUE(file_path, symbol, kind, start_line, start_column)
        );
        CREATE TABLE IF NOT EXISTS symbol_edges_v2 (
            edge_id INTEGER PRIMARY KEY,
            from_symbol_id INTEGER NOT NULL,
            to_symbol_id INTEGER NOT NULL,
            edge_kind TEXT NOT NULL,
            confidence REAL NOT NULL,
            provenance TEXT NOT NULL DEFAULT 'ast_definition',
            UNIQUE(from_symbol_id, to_symbol_id, edge_kind)
        );
        CREATE INDEX IF NOT EXISTS idx_text_occurrences_symbol
            ON text_occurrences(symbol);
        CREATE INDEX IF NOT EXISTS idx_text_occurrences_file
            ON text_occurrences(file_path);
        CREATE INDEX IF NOT EXISTS idx_ast_definitions_symbol
            ON ast_definitions(symbol);
        CREATE INDEX IF NOT EXISTS idx_ast_references_symbol
            ON ast_references(symbol);
        CREATE INDEX IF NOT EXISTS idx_symbols_v2_symbol
            ON symbols_v2(symbol);
        CREATE INDEX IF NOT EXISTS idx_symbols_v2_file
            ON symbols_v2(file_path);
        CREATE INDEX IF NOT EXISTS idx_edges_v2_from_kind
            ON symbol_edges_v2(from_symbol_id, edge_kind);
        CREATE INDEX IF NOT EXISTS idx_edges_v2_to_kind
            ON symbol_edges_v2(to_symbol_id, edge_kind);
        "#,
    )?;
    migrate_schema_v3(connection)?;

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

fn migrate_schema_v3(connection: &Connection) -> anyhow::Result<()> {
    ensure_column_exists(
        connection,
        "symbols_v2",
        "language",
        "ALTER TABLE symbols_v2 ADD COLUMN language TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    ensure_column_exists(
        connection,
        "symbols_v2",
        "qualified_symbol",
        "ALTER TABLE symbols_v2 ADD COLUMN qualified_symbol TEXT",
    )?;
    ensure_column_exists(
        connection,
        "symbol_edges_v2",
        "provenance",
        "ALTER TABLE symbol_edges_v2 ADD COLUMN provenance TEXT NOT NULL DEFAULT 'ast_definition'",
    )?;

    connection.execute_batch(
        r#"
        UPDATE symbols_v2
        SET language = CASE
            WHEN file_path LIKE '%.rs' THEN 'rust'
            WHEN file_path LIKE '%.ts' OR file_path LIKE '%.tsx' THEN 'typescript'
            WHEN file_path LIKE '%.py' THEN 'python'
            ELSE 'unknown'
        END
        WHERE language IS NULL OR language = '';

        UPDATE symbols_v2
        SET qualified_symbol = language || ':' || file_path || '::' || symbol
        WHERE qualified_symbol IS NULL OR qualified_symbol = '';

        UPDATE symbol_edges_v2
        SET provenance = CASE edge_kind
            WHEN 'calls' THEN 'call_resolution'
            WHEN 'imports' THEN 'import_resolution'
            WHEN 'contains' THEN 'ast_definition'
            WHEN 'implements' THEN 'ast_reference'
            ELSE 'ast_reference'
        END
        WHERE provenance IS NULL OR provenance = '' OR provenance = 'ast';

        CREATE INDEX IF NOT EXISTS idx_symbols_v2_language_symbol
            ON symbols_v2(language, symbol);
        CREATE INDEX IF NOT EXISTS idx_symbols_v2_qualified_symbol
            ON symbols_v2(qualified_symbol);
        "#,
    )?;

    Ok(())
}

fn ensure_column_exists(
    connection: &Connection,
    table: &str,
    column: &str,
    alter_sql: &str,
) -> anyhow::Result<()> {
    if !table_has_column(connection, table, column)? {
        connection.execute(alter_sql, [])?;
    }
    Ok(())
}

fn table_has_column(connection: &Connection, table: &str, column: &str) -> anyhow::Result<bool> {
    let query = format!("PRAGMA table_info({table})");
    let mut statement = connection.prepare(&query)?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column {
            return Ok(true);
        }
    }

    Ok(false)
}
