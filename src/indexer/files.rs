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
