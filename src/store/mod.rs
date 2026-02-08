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

#[cfg(test)]
mod tests {
    use super::{ensure_store, index_db_path, is_corruption_error, with_corruption_hint};
    use crate::store::schema::SCHEMA_VERSION;
    use anyhow::anyhow;
    use rusqlite::ffi::{Error as SqliteFfiError, ErrorCode};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn sqlite_failure_error(code: ErrorCode) -> anyhow::Error {
        anyhow::Error::new(rusqlite::Error::SqliteFailure(
            SqliteFfiError {
                code,
                extended_code: 0,
            },
            None,
        ))
    }

    #[test]
    fn index_db_path_is_repo_relative() {
        let repo = tempdir().expect("temp dir should be created");
        assert_eq!(
            index_db_path(repo.path()),
            repo.path().join(".repo-scout").join("index.db")
        );
    }

    #[test]
    fn ensure_store_bootstraps_database_and_schema() {
        let repo = tempdir().expect("temp dir should be created");

        let metadata = ensure_store(repo.path()).expect("store bootstrap should succeed");
        assert_eq!(metadata.db_path, index_db_path(repo.path()));
        assert_eq!(metadata.schema_version, SCHEMA_VERSION);
        assert!(
            metadata.db_path.exists(),
            "index database should exist after bootstrap"
        );

        let second = ensure_store(repo.path()).expect("second bootstrap should succeed");
        assert_eq!(second.db_path, metadata.db_path);
        assert_eq!(second.schema_version, metadata.schema_version);
    }

    #[test]
    fn ensure_store_reports_corruption_hint_for_invalid_index_file() {
        let repo = tempdir().expect("temp dir should be created");
        let index_path = index_db_path(repo.path());
        fs::create_dir_all(index_path.parent().expect("index path should have parent"))
            .expect("index directory should be created");
        fs::write(&index_path, b"not a sqlite database")
            .expect("fixture index file should be written");

        let error = ensure_store(repo.path()).expect_err("invalid sqlite file should fail");
        let message = format!("{error:#}");
        assert!(message.contains("appears corrupted"));
        assert!(message.contains(&index_path.display().to_string()));
        assert!(message.contains("repo-scout index --repo <path>"));
    }

    #[test]
    fn corruption_detection_matches_sqlite_codes() {
        let corrupt = sqlite_failure_error(ErrorCode::DatabaseCorrupt);
        assert!(is_corruption_error(&corrupt));

        let not_db = sqlite_failure_error(ErrorCode::NotADatabase);
        assert!(is_corruption_error(&not_db));

        let non_corrupt = anyhow!("plain error");
        assert!(!is_corruption_error(&non_corrupt));
    }

    #[test]
    fn with_corruption_hint_only_rewrites_corrupt_errors() {
        let plain = anyhow!("plain error");
        let plain_rendered = with_corruption_hint(plain, Path::new("index.db")).to_string();
        assert_eq!(plain_rendered, "plain error");

        let corrupt = sqlite_failure_error(ErrorCode::DatabaseCorrupt);
        let hinted = with_corruption_hint(corrupt, Path::new("index.db")).to_string();
        assert!(hinted.contains("index database appears corrupted at index.db"));
    }
}
