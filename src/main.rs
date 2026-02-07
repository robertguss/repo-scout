mod cli;
mod indexer;
mod output;
mod query;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::indexer::index_repository;
use crate::query::{
    ChangedLineRange, DiffImpactOptions, QueryScope, context_matches, context_matches_scoped,
    diff_impact_for_changed_files,
    explain_symbol, find_matches_scoped, impact_matches, refs_matches_scoped, tests_for_symbol,
    verify_plan_for_changed_files,
};
use crate::store::ensure_store;

/// Program entry point that runs the CLI and exits on failure.
///
/// If `run()` returns an error, the error is printed to standard error using
/// pretty formatting and the process terminates with exit code 1.
///
/// # Examples
///
/// ```ignore
/// // Typical binary entry: just call `main()`
/// main();
/// ```
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

/// Parses command-line arguments and dispatches to the selected command handler.
///
/// Parses program arguments into a `Cli` and invokes the corresponding command implementation
/// (index, status, find, refs, impact, context, tests-for, or verify-plan).
///
/// # Returns
///
/// `Ok(())` if the selected command completes successfully, an `Err` if the command fails.
///
/// # Examples
///
/// ```no_run
/// // Dispatch based on current process arguments.
/// let _ = crate::run();
/// ```
fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Index(args) => run_index(args),
        Command::Status(args) => run_status(args),
        Command::Find(args) => run_find(args),
        Command::Refs(args) => run_refs(args),
        Command::Impact(args) => run_impact(args),
        Command::Context(args) => run_context(args),
        Command::TestsFor(args) => run_tests_for(args),
        Command::VerifyPlan(args) => run_verify_plan(args),
        Command::DiffImpact(args) => run_diff_impact(args),
        Command::Explain(args) => run_explain(args),
    }
}

fn run_index(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let summary = index_repository(&args.repo, &store.db_path)?;
    output::print_index(
        &store.db_path,
        store.schema_version,
        summary.indexed_files,
        summary.skipped_files,
    );
    Ok(())
}

fn run_status(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    output::print_status(&store.db_path, store.schema_version);
    Ok(())
}

fn run_find(args: crate::cli::FindArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = find_matches_scoped(
        &store.db_path,
        &args.symbol,
        &crate::query::QueryScope {
            code_only: args.code_only,
            exclude_tests: args.exclude_tests,
        },
    )?;
    if args.json {
        output::print_query_json("find", &args.symbol, &matches)?;
    } else {
        output::print_query("find", &args.symbol, &matches);
    }
    Ok(())
}

/// Run the "refs" query for a symbol and print the results.
///
/// Ensures the repository store exists, obtains references for `args.symbol` from the store,
/// and prints the results as JSON when `args.json` is true or as human-readable output otherwise.
///
/// # Returns
///
/// `Ok(())` on success; an error if ensuring the store, querying references, or printing the results fails.
///
/// # Examples
///
/// ```
/// let args = crate::cli::RefsArgs {
///     repo: "/path/to/repo".into(),
///     symbol: "my::Symbol".into(),
///     json: false,
///     code_only: false,
///     exclude_tests: false,
/// };
/// let _ = run_refs(args);
/// ```
fn run_refs(args: crate::cli::RefsArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = refs_matches_scoped(
        &store.db_path,
        &args.symbol,
        &crate::query::QueryScope {
            code_only: args.code_only,
            exclude_tests: args.exclude_tests,
        },
    )?;
    if args.json {
        output::print_query_json("refs", &args.symbol, &matches)?;
    } else {
        output::print_query("refs", &args.symbol, &matches);
    }
    Ok(())
}

/// Runs an impact query for a symbol and prints the results.
///
/// Ensures the repository store exists, computes impact matches for `args.symbol`,
/// and prints the output as JSON when `args.json` is true or as human-readable text otherwise.
///
/// # Errors
///
/// Returns an error if the store cannot be initialized or if computing or printing matches fails.
///
/// # Examples
///
/// ```no_run
/// use crate::cli::QueryArgs;
///
/// let args = QueryArgs { repo: ".".into(), symbol: "my_crate::foo".into(), json: false };
/// run_impact(args).unwrap();
/// ```
fn run_impact(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = impact_matches(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_impact_json(&args.symbol, &matches)?;
    } else {
        output::print_impact(&args.symbol, &matches);
    }
    Ok(())
}

/// Query matches relevant to a task within a repository and print the results.
///
/// The function ensures the repository store is available, retrieves context matches for the
/// provided task and budget, and prints the matches either as JSON (when `args.json` is true)
/// or as formatted human-readable output.
///
/// # Parameters
///
/// - `args`: CLI arguments containing the repository root, the task to query, the numeric budget,
///   and the `json` flag controlling output format.
///
/// # Returns
///
/// `Ok(())` on success, `Err` if the store cannot be accessed or the query or output formatting fails.
///
/// # Examples
///
/// ```
/// use crate::cli::ContextArgs;
///
/// // Construct CLI-like args and run the command handler.
/// let args = ContextArgs {
///     repo: ".".into(),
///     task: "build".into(),
///     budget: 10,
///     json: false,
///     code_only: false,
///     exclude_tests: false,
/// };
/// let _ = crate::run_context(args);
/// ```
fn run_context(args: crate::cli::ContextArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let scope = QueryScope {
        code_only: args.code_only,
        exclude_tests: args.exclude_tests,
    };
    let matches = if scope == QueryScope::default() {
        context_matches(&store.db_path, &args.task, args.budget)?
    } else {
        context_matches_scoped(&store.db_path, &args.task, args.budget, &scope)?
    };
    if args.json {
        output::print_context_json(&args.task, args.budget, &matches)?;
    } else {
        output::print_context(&args.task, args.budget, &matches);
    }
    Ok(())
}

/// Query test targets that reference a symbol and print the results.
///
/// Ensures the repository store exists, retrieves test targets for `args.symbol`, and prints them
/// as JSON when `args.json` is true or as human-readable output otherwise.
///
/// # Examples
///
/// ```no_run
/// use crate::cli::QueryArgs;
///
/// let args = QueryArgs {
///     repo: "/path/to/repo".into(),
///     symbol: "my_crate::module::Symbol".into(),
///     json: false,
///     ..Default::default()
/// };
///
/// let _ = crate::run_tests_for(args);
/// ```
fn run_tests_for(args: crate::cli::TestsForArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let targets = tests_for_symbol(&store.db_path, &args.symbol, args.include_support)?;
    if args.json {
        output::print_tests_for_json(&args.symbol, &targets)?;
    } else {
        output::print_tests_for(&args.symbol, &targets);
    }
    Ok(())
}

/// Computes a verification plan for the given changed files and prints the results.
///
/// Ensures the repository store exists, normalizes and deduplicates the provided changed-file
/// paths, computes the verification steps for those files, and prints the plan either as JSON
/// (when `args.json` is true) or in a human-readable format.
///
/// # Examples
///
/// ```
/// use crate::cli::VerifyPlanArgs;
///
/// let args = VerifyPlanArgs {
///     repo: ".".into(),
///     changed_files: vec!["src/lib.rs".into()],
///     json: false,
/// };
/// let _ = run_verify_plan(args);
/// ```
///
/// # Returns
///
/// `Ok(())` on success, or an error if the store cannot be accessed or the verification plan
/// cannot be computed or printed.
fn run_verify_plan(args: crate::cli::VerifyPlanArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let mut changed_files = args
        .changed_files
        .iter()
        .map(|path| normalize_changed_file(&args.repo, path))
        .collect::<Vec<_>>();
    changed_files.sort();
    changed_files.dedup();

    let steps = verify_plan_for_changed_files(&store.db_path, &changed_files, args.max_targeted)?;
    if args.json {
        output::print_verify_plan_json(&changed_files, &steps)?;
    } else {
        output::print_verify_plan(&changed_files, &steps);
    }
    Ok(())
}

fn run_diff_impact(args: crate::cli::DiffImpactArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let mut changed_files = args
        .changed_files
        .iter()
        .map(|path| normalize_changed_file(&args.repo, path))
        .collect::<Vec<_>>();
    changed_files.sort();
    changed_files.dedup();

    let mut changed_lines = args
        .changed_lines
        .iter()
        .map(|spec| parse_changed_line_spec(&args.repo, spec))
        .collect::<anyhow::Result<Vec<_>>>()?;
    changed_lines.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.end_line.cmp(&right.end_line))
    });
    changed_lines.dedup_by(|left, right| {
        left.file_path == right.file_path
            && left.start_line == right.start_line
            && left.end_line == right.end_line
    });

    let options = DiffImpactOptions {
        max_distance: args.max_distance,
        include_tests: args.include_tests,
        include_imports: args.include_imports,
        changed_lines,
    };
    let matches = diff_impact_for_changed_files(&store.db_path, &changed_files, &options)?;
    if args.json {
        output::print_diff_impact_json(
            &changed_files,
            options.max_distance,
            options.include_tests,
            &matches,
        )?;
    } else {
        output::print_diff_impact(
            &changed_files,
            options.max_distance,
            options.include_tests,
            &matches,
        );
    }
    Ok(())
}

fn parse_changed_line_spec(
    repo_root: &std::path::Path,
    raw_spec: &str,
) -> anyhow::Result<ChangedLineRange> {
    let mut segments = raw_spec.rsplitn(3, ':');
    let last = segments
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid --changed-line '{raw_spec}'"))?;
    let second = segments
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid --changed-line '{raw_spec}'"))?;
    let third = segments.next();

    let (path_part, start_part, end_part) = if second.parse::<u32>().is_ok() {
        (third.unwrap_or("").to_string(), second, last)
    } else {
        let mut path_part = String::new();
        if let Some(prefix) = third {
            path_part.push_str(prefix);
            path_part.push(':');
        }
        path_part.push_str(second);
        (path_part, last, last)
    };

    let start_line = start_part.parse::<u32>().ok();
    let end_line = end_part.parse::<u32>().ok();
    let Some(start_line) = start_line else {
        anyhow::bail!(
            "invalid --changed-line '{raw_spec}': expected format path:start[:end] with positive line numbers"
        );
    };
    let Some(end_line) = end_line else {
        anyhow::bail!(
            "invalid --changed-line '{raw_spec}': expected format path:start[:end] with positive line numbers"
        );
    };
    if start_line == 0 || end_line == 0 || end_line < start_line {
        anyhow::bail!(
            "invalid --changed-line '{raw_spec}': expected format path:start[:end] with start <= end and both >= 1"
        );
    }
    if path_part.trim().is_empty() {
        anyhow::bail!(
            "invalid --changed-line '{raw_spec}': expected format path:start[:end] with a non-empty path"
        );
    }

    Ok(ChangedLineRange {
        file_path: normalize_changed_file(repo_root, &path_part),
        start_line,
        end_line,
    })
}

fn run_explain(args: crate::cli::ExplainArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = explain_symbol(&store.db_path, &args.symbol, args.include_snippets)?;
    if args.json {
        output::print_explain_json(&args.symbol, args.include_snippets, &matches)?;
    } else {
        output::print_explain(&args.symbol, &matches);
    }
    Ok(())
}

/// Normalize a changed-file path into a repository-relative, forward-slash string.
///
/// The returned string has any leading "./" removed and all backslashes replaced with
/// forward slashes. If `changed_file` is an absolute path that is inside `repo_root`,
/// the result will be relative to `repo_root`; otherwise the original path is used
/// (converted to a string and normalized).
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// // absolute path inside repo becomes relative
/// let repo = Path::new("/repo");
/// assert_eq!(
///     crate::normalize_changed_file(repo, "/repo/src/lib.rs"),
///     "src/lib.rs"
/// );
///
/// // relative path keeps relative form and normalizes separators / leading ./
/// assert_eq!(
///     crate::normalize_changed_file(repo, "./src\\main.rs"),
///     "src/main.rs"
/// );
/// ```
fn normalize_changed_file(repo_root: &std::path::Path, changed_file: &str) -> String {
    let candidate = std::path::PathBuf::from(changed_file);
    let absolute_repo_root = std::fs::canonicalize(repo_root).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(repo_root))
            .unwrap_or_else(|_| repo_root.to_path_buf())
    });
    let normalized = if candidate.is_absolute() {
        let canonical_candidate = std::fs::canonicalize(&candidate).ok();
        canonical_candidate
            .as_deref()
            .and_then(|path| path.strip_prefix(&absolute_repo_root).ok())
            .map(|path| path.to_path_buf())
            .or_else(|| {
                candidate
                    .strip_prefix(&absolute_repo_root)
                    .ok()
                    .map(|path| path.to_path_buf())
            })
            .or_else(|| {
                candidate
                    .strip_prefix(repo_root)
                    .ok()
                    .map(|path| path.to_path_buf())
            })
            .unwrap_or(candidate)
    } else {
        candidate
    };

    normalized
        .to_string_lossy()
        .trim_start_matches("./")
        .replace('\\', "/")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_changed_file, parse_changed_line_spec};
    use std::path::Path;

    #[test]
    fn parse_changed_line_spec_accepts_windows_drive_path_without_end() {
        let parsed = parse_changed_line_spec(Path::new("."), r"C:\repo\src\lib.rs:12")
            .expect("windows path with start line should parse");
        assert_eq!(
            parsed.file_path,
            normalize_changed_file(Path::new("."), r"C:\repo\src\lib.rs")
        );
        assert_eq!(parsed.start_line, 12);
        assert_eq!(parsed.end_line, 12);
    }

    #[test]
    fn parse_changed_line_spec_accepts_windows_drive_path_with_end() {
        let parsed = parse_changed_line_spec(Path::new("."), r"C:\repo\src\lib.rs:12:24")
            .expect("windows path with start/end lines should parse");
        assert_eq!(
            parsed.file_path,
            normalize_changed_file(Path::new("."), r"C:\repo\src\lib.rs")
        );
        assert_eq!(parsed.start_line, 12);
        assert_eq!(parsed.end_line, 24);
    }
}
