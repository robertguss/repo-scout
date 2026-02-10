use std::fs;
use std::path::Path;

use anyhow::Context;
use ignore::WalkBuilder;

#[derive(Debug)]
pub struct SourceFile {
    pub relative_path: String,
    pub bytes: Vec<u8>,
    pub content_hash: String,
}

pub fn discover_source_files(repo: &Path) -> anyhow::Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    let walker = WalkBuilder::new(repo).standard_filters(true).build();

    for entry in walker {
        let entry = entry.with_context(|| format!("failed to walk {}", repo.display()))?;
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();
        let relative = path
            .strip_prefix(repo)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read file for indexing: {}", path.display()))?;
        let content_hash = blake3::hash(&bytes).to_hex().to_string();

        files.push(SourceFile {
            relative_path: relative,
            bytes,
            content_hash,
        });
    }

    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::discover_source_files;
    use std::fs;

    #[cfg(unix)]
    fn with_unreadable_file(path: &std::path::Path, f: impl FnOnce()) {
        use std::os::unix::fs::PermissionsExt;

        let original = fs::metadata(path)
            .expect("fixture file metadata should load")
            .permissions();
        let mut unreadable = original.clone();
        unreadable.set_mode(0o000);
        fs::set_permissions(path, unreadable).expect("file should be made unreadable");
        f();
        fs::set_permissions(path, original).expect("file permissions should be restored");
    }

    #[test]
    fn discover_source_files_reports_walk_errors() {
        let repo = tempfile::tempdir().expect("temp dir should be created");
        let missing = repo.path().join("missing");
        let error = discover_source_files(&missing).expect_err("missing repo should fail walking");
        assert!(error.to_string().contains("failed to walk"));
    }

    #[test]
    #[cfg(unix)]
    fn discover_source_files_reports_read_errors() {
        let repo = tempfile::tempdir().expect("temp dir should be created");
        let unreadable = repo.path().join("secret.rs");
        fs::write(&unreadable, "fn hidden() {}\n").expect("fixture file should be written");

        with_unreadable_file(&unreadable, || {
            let error = discover_source_files(repo.path())
                .expect_err("unreadable file should fail content loading");
            assert!(
                error
                    .to_string()
                    .contains("failed to read file for indexing")
            );
        });
    }
}
