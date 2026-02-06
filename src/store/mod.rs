use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use rusqlite::Connection;

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
        .with_context(|| format!("failed to open sqlite database {}", db_path.display()))?;
    schema::bootstrap_schema(&connection)?;
    let schema_version = schema::read_schema_version(&connection)?;

    Ok(StoreMetadata {
        db_path,
        schema_version,
    })
}

fn index_db_path(repo: &Path) -> PathBuf {
    repo.join(".repo-scout").join("index.db")
}
