use std::path::Path;
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VerifyRefactorReport {
    pub before: String,
    pub after: String,
    pub changed_files: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn verify_refactor_report(
    repo_root: &Path,
    before: &str,
    after: Option<&str>,
) -> anyhow::Result<VerifyRefactorReport> {
    let after_value = after.unwrap_or("working-tree").to_string();
    let mut warnings = Vec::new();
    let mut changed_files = Vec::new();

    if let Some(after_ref) = after {
        let output = Command::new("git")
            .args(["diff", "--name-only", before, after_ref])
            .current_dir(repo_root)
            .output();
        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8(output.stdout)?;
                changed_files = stdout
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
            }
            Ok(output) => {
                warnings.push(format!(
                    "unable to compute git diff between '{before}' and '{after_ref}': {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Err(error) => {
                warnings.push(format!(
                    "unable to invoke git diff between '{before}' and '{after_ref}': {error}"
                ));
            }
        }
    } else {
        warnings.push("after reference not provided; comparing against working tree".to_string());
    }

    Ok(VerifyRefactorReport {
        before: before.to_string(),
        after: after_value,
        changed_files,
        warnings,
    })
}
