use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use rusqlite::{Connection, ffi::ErrorCode};

pub mod schema;

#[derive(Debug)]
pub struct StoreMetadata {
    pub db_path: PathBuf,
    pub schema_version: i64,
}

pub fn ensure_store(repo: &Path) -> anyhow::Result<StoreMetadata> {
    let db_path = index_db_path(repo);
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create index directory for {}",
                parent.to_string_lossy()
            )
        })?;
    }

    let connection = Connection::open(&db_path)
        .with_context(|| format!("failed to open sqlite database {}", db_path.display()))
        .map_err(|error| with_corruption_hint(error, &db_path))?;
    schema::bootstrap_schema(&connection).map_err(|error| with_corruption_hint(error, &db_path))?;
    let schema_version = schema::read_schema_version(&connection)
        .map_err(|error| with_corruption_hint(error, &db_path))?;

    Ok(StoreMetadata {
        db_path,
        schema_version,
    })
}

fn index_db_path(repo: &Path) -> PathBuf {
    repo.join(".repo-scout").join("index.db")
}

fn with_corruption_hint(error: anyhow::Error, db_path: &Path) -> anyhow::Error {
    if is_corruption_error(&error) {
        return anyhow!(
            "index database appears corrupted at {}. delete this file and rerun `repo-scout index --repo <path>`.",
            db_path.display()
        );
    }
    error
}

fn is_corruption_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        let Some(sqlite_error) = cause.downcast_ref::<rusqlite::Error>() else {
            return false;
        };

        matches!(
            sqlite_error.sqlite_error_code(),
            Some(ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase)
        )
    })
}
