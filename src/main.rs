mod cli;
mod git_utils;
mod indexer;
mod output;
mod query;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::indexer::index_repository;
use crate::query::{
    ChangedLineRange, DiffImpactChangedMode, DiffImpactImportMode, DiffImpactOptions,
    DiffImpactTestMode, QueryScope, VerifyPlanOptions, context_matches, context_matches_scoped,
    diff_impact_for_changed_files, explain_symbol, find_matches_scoped, impact_matches,
    callees_of, callers_of, file_deps, outline_file, refs_matches_scoped, repo_entry_points,
    snippet_for_symbol, status_summary, tests_for_symbol, verify_plan_for_changed_files,
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
        Command::Snippet(args) => run_snippet(args),
        Command::Outline(args) => run_outline(args),
        Command::Summary(args) => run_summary_cmd(args),
        Command::Callers(args) => run_callers(args),
        Command::Callees(args) => run_callees(args),
        Command::Deps(args) => run_deps(args),
    }
}

fn run_index(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let summary = index_repository(&args.repo, &store.db_path)?;
    output::print_index(
        &store.db_path,
        store.schema_version,
        summary.indexed_files,
        summary.non_source_files,
    );
    Ok(())
}

fn run_status(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let summary = status_summary(&store.db_path)?;
    output::print_status(&store.db_path, store.schema_version, &summary);
    Ok(())
}

fn run_find(args: crate::cli::FindArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let scope = QueryScope::from_flags(args.code_only, args.exclude_tests);
    let mut matches = find_matches_scoped(&store.db_path, &args.symbol, &scope)?;
    if let Some(max_results) = args.max_results {
        matches.truncate(u32_to_usize(max_results));
    }
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
/// `Ok(())` on success.
/// Returns an error when store setup, query execution, or printing fails.
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
///     max_results: None,
/// };
/// let _ = run_refs(args);
/// ```
fn run_refs(args: crate::cli::RefsArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let scope = QueryScope::from_flags(args.code_only, args.exclude_tests);
    let mut matches = refs_matches_scoped(&store.db_path, &args.symbol, &scope)?;
    if let Some(max_results) = args.max_results {
        matches.truncate(u32_to_usize(max_results));
    }
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
/// `Ok(())` on success.
/// Returns `Err` if the store cannot be accessed or output generation fails.
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
    let scope = QueryScope::from_flags(args.code_only, args.exclude_tests);
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
///     changed_lines: vec![],
///     changed_symbols: vec![],
///     max_targeted: None,
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
    if let Some(ref since) = args.since {
        let git_files = git_utils::changed_files_since(&args.repo, since)?;
        changed_files.extend(git_files);
    }
    if args.unstaged {
        let unstaged = git_utils::unstaged_files(&args.repo)?;
        changed_files.extend(unstaged);
    }
    if changed_files.is_empty() {
        anyhow::bail!(
            "no changed files: provide --changed-file, --since, or --unstaged"
        );
    }
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
    let mut changed_symbols = args.changed_symbols.clone();
    changed_symbols.sort();
    changed_symbols.dedup();

    let options = VerifyPlanOptions {
        max_targeted: args.max_targeted,
        changed_lines,
        changed_symbols,
    };
    let steps = verify_plan_for_changed_files(&store.db_path, &changed_files, &options)?;
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
    if let Some(ref since) = args.since {
        let git_files = git_utils::changed_files_since(&args.repo, since)?;
        changed_files.extend(git_files);
    }
    if args.unstaged {
        let unstaged = git_utils::unstaged_files(&args.repo)?;
        changed_files.extend(unstaged);
    }
    if changed_files.is_empty() {
        anyhow::bail!(
            "no changed files: provide --changed-file, --since, or --unstaged"
        );
    }
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
    let mut changed_symbols = args.changed_symbols.clone();
    changed_symbols.sort();
    changed_symbols.dedup();
    let test_mode = if args.exclude_tests {
        DiffImpactTestMode::ExcludeTests
    } else {
        DiffImpactTestMode::IncludeTests
    };

    let options = DiffImpactOptions {
        max_distance: args.max_distance,
        test_mode,
        import_mode: if args.include_imports {
            DiffImpactImportMode::IncludeImports
        } else {
            DiffImpactImportMode::ExcludeImports
        },
        changed_lines,
        changed_symbols,
        changed_mode: if args.exclude_changed {
            DiffImpactChangedMode::ExcludeChanged
        } else {
            DiffImpactChangedMode::IncludeChanged
        },
        max_results: if args.no_limit {
            None
        } else {
            Some(args.max_results)
        },
    };
    let matches = diff_impact_for_changed_files(&store.db_path, &changed_files, &options)?;
    let include_tests = matches!(options.test_mode, DiffImpactTestMode::IncludeTests);
    if args.json {
        let print_result = output::print_diff_impact_json(
            &changed_files,
            options.max_distance,
            include_tests,
            &matches,
        );
        print_result?;
    } else {
        output::print_diff_impact(
            &changed_files,
            options.max_distance,
            include_tests,
            &matches,
        );
    }
    Ok(())
}

#[must_use]
fn u32_to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
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
            "invalid --changed-line '{}': expected format {}",
            raw_spec,
            "path:start[:end] with positive line numbers"
        );
    };
    let Some(end_line) = end_line else {
        anyhow::bail!(
            "invalid --changed-line '{}': expected format {}",
            raw_spec,
            "path:start[:end] with positive line numbers"
        );
    };
    if start_line == 0 || end_line == 0 || end_line < start_line {
        anyhow::bail!(
            "invalid --changed-line '{}': expected format {}",
            raw_spec,
            "path:start[:end] with start <= end and both >= 1"
        );
    }
    if path_part.trim().is_empty() {
        anyhow::bail!(
            "invalid --changed-line '{}': expected format {}",
            raw_spec,
            "path:start[:end] with a non-empty path"
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

fn run_outline(args: crate::cli::OutlineArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let entries = outline_file(&store.db_path, &args.file)?;
    if args.json {
        output::print_outline_json(&args.file, &entries)?;
    } else {
        output::print_outline(&args.file, &entries);
    }
    Ok(())
}

fn run_deps(args: crate::cli::DepsArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let deps = file_deps(&store.db_path, &args.file)?;
    if args.json {
        output::print_deps_json(&args.file, &deps)?;
    } else {
        output::print_deps(&args.file, &deps);
    }
    Ok(())
}

fn run_callers(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = callers_of(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_edges_json("callers", &args.symbol, &matches)?;
    } else {
        output::print_edges("callers", &args.symbol, &matches);
    }
    Ok(())
}

fn run_callees(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = callees_of(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_edges_json("callees", &args.symbol, &matches)?;
    } else {
        output::print_edges("callees", &args.symbol, &matches);
    }
    Ok(())
}

fn run_summary_cmd(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let summary = status_summary(&store.db_path)?;
    let entry_points = repo_entry_points(&store.db_path)?;
    output::print_summary(&summary, &entry_points);
    Ok(())
}

fn run_snippet(args: crate::cli::SnippetArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = snippet_for_symbol(&store.db_path, &args.symbol, args.context)?;
    if args.json {
        output::print_snippet_json(&args.symbol, &matches)?;
    } else {
        output::print_snippet(&args.symbol, &matches);
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
    use super::{
        normalize_changed_file, parse_changed_line_spec, run_context, run_diff_impact, run_explain,
        run_find, run_impact, run_index, run_refs, run_status, run_tests_for, run_verify_plan,
    };
    use crate::cli::{
        ContextArgs, DiffImpactArgs, ExplainArgs, FindArgs, QueryArgs, RefsArgs, RepoArgs,
        TestsForArgs, VerifyPlanArgs,
    };
    use std::path::Path;
    use tempfile::TempDir;

    #[cfg(unix)]
    fn create_symlink(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(src, dst)
    }

    fn write_file(path: &Path, content: &str) {
        let parent = path.parent().expect("fixture file should have a parent");
        std::fs::create_dir_all(parent).expect("fixture parent directory should be created");
        std::fs::write(path, content).expect("fixture file should be written");
    }

    fn fixture_repo() -> TempDir {
        let repo = tempfile::tempdir().expect("temp repo should be created");
        write_file(
            &repo.path().join("src/lib.rs"),
            r#"
pub fn run_find() {}
pub fn run_refs() { run_find(); }
"#,
        );
        write_file(
            &repo.path().join("tests/lib_test.rs"),
            r#"
#[test]
fn integration_check() {
    crate::run_find();
}
"#,
        );
        repo
    }

    #[test]
    fn command_handlers_cover_json_scope_and_changed_line_paths() {
        let repo = fixture_repo();
        let repo_path = repo.path().to_path_buf();

        run_index(RepoArgs {
            repo: repo_path.clone(),
        })
        .expect("index should succeed");
        run_status(RepoArgs {
            repo: repo_path.clone(),
        })
        .expect("status should succeed");

        run_find(FindArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            code_only: true,
            exclude_tests: true,
            max_results: Some(1),
        })
        .expect("find json should succeed");
        run_find(FindArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            code_only: false,
            exclude_tests: false,
            max_results: None,
        })
        .expect("find text should succeed");

        run_refs(RefsArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            code_only: false,
            exclude_tests: false,
            max_results: Some(10),
        })
        .expect("refs json should succeed");
        run_refs(RefsArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            code_only: true,
            exclude_tests: false,
            max_results: None,
        })
        .expect("refs text should succeed");

        run_impact(QueryArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
        })
        .expect("impact json should succeed");
        run_impact(QueryArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
        })
        .expect("impact text should succeed");

        run_context(ContextArgs {
            task: "run find references".to_string(),
            repo: repo_path.clone(),
            json: true,
            budget: 16,
            code_only: false,
            exclude_tests: false,
        })
        .expect("context json with default scope should succeed");
        run_context(ContextArgs {
            task: "run find references".to_string(),
            repo: repo_path.clone(),
            json: false,
            budget: 16,
            code_only: true,
            exclude_tests: true,
        })
        .expect("context text with scoped query should succeed");

        run_tests_for(TestsForArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            include_support: true,
        })
        .expect("tests-for json should succeed");
        run_tests_for(TestsForArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            include_support: false,
        })
        .expect("tests-for text should succeed");

        let changed_lines = vec![
            "src/lib.rs:4:4".to_string(),
            "src/lib.rs:2:2".to_string(),
            "src/lib.rs:4:4".to_string(),
        ];

        run_verify_plan(VerifyPlanArgs {
            changed_files: vec![
                "tests/lib_test.rs".to_string(),
                "src/lib.rs".to_string(),
                "src/lib.rs".to_string(),
            ],
            changed_lines: changed_lines.clone(),
            changed_symbols: vec![
                "run_find".to_string(),
                "run_refs".to_string(),
                "run_find".to_string(),
            ],
            since: None,
            unstaged: false,
            max_targeted: Some(3),
            repo: repo_path.clone(),
            json: true,
        })
        .expect("verify-plan json should succeed");
        run_verify_plan(VerifyPlanArgs {
            changed_files: vec!["src/lib.rs".to_string()],
            changed_lines: changed_lines.clone(),
            changed_symbols: vec!["run_find".to_string()],
            since: None,
            unstaged: false,
            max_targeted: None,
            repo: repo_path.clone(),
            json: false,
        })
        .expect("verify-plan text should succeed");

        run_diff_impact(DiffImpactArgs {
            changed_files: vec!["src/lib.rs".to_string(), "src/lib.rs".to_string()],
            changed_lines: changed_lines.clone(),
            changed_symbols: vec!["run_find".to_string(), "run_find".to_string()],
            since: None,
            unstaged: false,
            max_distance: 2,
            max_results: 20,
            no_limit: false,
            include_tests: false,
            exclude_tests: true,
            include_imports: true,
            exclude_changed: true,
            repo: repo_path.clone(),
            json: true,
        })
        .expect("diff-impact json should succeed");
        run_diff_impact(DiffImpactArgs {
            changed_files: vec!["src/lib.rs".to_string()],
            changed_lines,
            changed_symbols: vec!["run_find".to_string()],
            since: None,
            unstaged: false,
            max_distance: 1,
            max_results: 30,
            no_limit: true,
            include_tests: true,
            exclude_tests: false,
            include_imports: false,
            exclude_changed: false,
            repo: repo_path.clone(),
            json: false,
        })
        .expect("diff-impact text should succeed");

        run_explain(ExplainArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            include_snippets: true,
        })
        .expect("explain json should succeed");
        run_explain(ExplainArgs {
            symbol: "run_find".to_string(),
            repo: repo_path,
            json: false,
            include_snippets: false,
        })
        .expect("explain text should succeed");
    }

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

    #[test]
    fn parse_changed_line_spec_rejects_non_positive_or_descending_ranges() {
        let zero_start = parse_changed_line_spec(Path::new("."), "src/lib.rs:0:1")
            .expect_err("zero start line should fail");
        assert!(
            zero_start
                .to_string()
                .contains("start <= end and both >= 1")
        );

        let descending = parse_changed_line_spec(Path::new("."), "src/lib.rs:8:2")
            .expect_err("descending range should fail");
        assert!(
            descending
                .to_string()
                .contains("start <= end and both >= 1")
        );
    }

    #[test]
    fn parse_changed_line_spec_rejects_empty_path() {
        let error =
            parse_changed_line_spec(Path::new("."), ":12:24").expect_err("empty path should fail");
        assert!(error.to_string().contains("non-empty path"));
    }

    #[test]
    fn parse_changed_line_spec_rejects_non_numeric_end_line() {
        let error = parse_changed_line_spec(Path::new("."), "src/lib.rs:12:not-a-number")
            .expect_err("non-numeric end line should fail");
        assert!(
            error
                .to_string()
                .contains("path:start[:end] with positive line numbers")
        );
    }

    #[test]
    fn normalize_changed_file_trims_prefix_and_normalizes_separators() {
        let normalized = normalize_changed_file(Path::new("."), "./src\\main.rs");
        assert_eq!(normalized, "src/main.rs");
    }

    #[test]
    fn normalize_changed_file_uses_current_dir_fallback_for_missing_repo_root() {
        let repo_root = Path::new("target/repo-scout-tests/missing-repo-root");
        let absolute_candidate = std::env::current_dir()
            .expect("current dir should be available")
            .join(repo_root)
            .join("src/lib.rs");
        let normalized = normalize_changed_file(repo_root, &absolute_candidate.to_string_lossy());
        assert_eq!(normalized, "src/lib.rs");
    }

    #[test]
    #[cfg(unix)]
    fn normalize_changed_file_uses_literal_repo_prefix_when_canonical_prefix_differs() {
        let sandbox = tempfile::tempdir().expect("sandbox should be created");
        let real_root = sandbox.path().join("real");
        std::fs::create_dir_all(&real_root).expect("real root should be created");
        let link_root = sandbox.path().join("repo-link");
        create_symlink(&real_root, &link_root).expect("symlink should be created");

        let missing_candidate = link_root.join("src/missing.rs");
        let normalized = normalize_changed_file(&link_root, &missing_candidate.to_string_lossy());
        assert_eq!(normalized, "src/missing.rs");
    }
}
