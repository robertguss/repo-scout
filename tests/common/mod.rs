use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

pub fn repo_scout_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!()
}

pub fn temp_repo() -> TempDir {
    tempfile::tempdir().expect("temporary repo should be created")
}

#[allow(dead_code)]
pub fn write_file(root: &Path, relative_path: &str, contents: &str) -> PathBuf {
    let full_path = root.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("parent directories should be created");
    }
    fs::write(&full_path, contents).expect("fixture file should be written");
    full_path
}
