use std::path::Path;
use std::process::Command;

pub fn changed_files_since(repo: &Path, since: &str) -> anyhow::Result<Vec<String>> {
    if since.starts_with('-') {
        anyhow::bail!(
            "changed_files_since: invalid revision '{}' (looks like a flag)",
            since
        );
    }
    let output = Command::new("git")
        .args(["diff", "--name-only", since, "HEAD"])
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let files = String::from_utf8(output.stdout)?
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    Ok(files)
}

pub fn unstaged_files(repo: &Path) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(repo)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let files = String::from_utf8(output.stdout)?
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    Ok(files)
}
