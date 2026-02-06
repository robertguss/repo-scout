use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

pub fn repo_scout_cmd() -> Command {
    let env_candidates = [
        "CARGO_BIN_EXE_codex-5-3",
        "CARGO_BIN_EXE_codex_5_3",
        "CARGO_BIN_EXE_repo-scout",
        "CARGO_BIN_EXE_repo_scout",
    ];

    for name in env_candidates {
        if let Some(path) = std::env::var_os(name).map(PathBuf::from) {
            if path.is_file() {
                return Command::new(path);
            }
        }
    }

    let mut target_dir = std::env::current_exe().expect("test executable path should be available");
    target_dir.pop();
    if target_dir.ends_with("deps") {
        target_dir.pop();
    }

    let bin_candidates = ["codex-5-3", "repo-scout"];
    for bin_name in bin_candidates {
        let path = target_dir.join(format!("{bin_name}{}", std::env::consts::EXE_SUFFIX));
        if path.is_file() {
            return Command::new(path);
        }
    }

    panic!(
        "repo-scout binary not found via CARGO_BIN_EXE_* or {}",
        target_dir.display()
    );
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
