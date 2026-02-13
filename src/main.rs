mod cli;
mod git_utils;
mod indexer;
mod output;
mod query;
mod store;

use std::fs;

use clap::Parser;
use rusqlite::Connection;

use crate::cli::{Cli, Command};
use crate::indexer::index_repository;
use crate::query::{
    ChangedLineRange, DiffImpactChangedMode, DiffImpactImportMode, DiffImpactOptions,
    DiffImpactTestMode, ExplainMatch, ImpactMatch, QueryPathMode, QueryScope, QueryTestMode,
    VerifyPlanOptions, callees_of, callers_of, context_matches, context_matches_scoped,
    diff_impact_for_changed_files, explain_symbol, file_deps, find_call_path, find_matches_scoped,
    hotspots, impact_matches, outline_file, refs_matches_scoped, related_symbols,
    repo_entry_points, snippet_for_symbol, status_summary, suggest_similar_symbols,
    tests_for_symbol, verify_plan_for_changed_files,
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
        Command::Hotspots(args) => run_hotspots(args),
        Command::CallPath(args) => run_call_path(args),
        Command::Related(args) => run_related(args),
        Command::Health(args) => run_health(args),
        Command::Circular(args) => run_circular(args),
        Command::Tree(args) => run_tree(args),
        Command::Orient(args) => run_orient(args),
        Command::Anatomy(args) => run_anatomy(args),
        Command::Coupling(args) => run_coupling(args),
        Command::Dead(args) => run_dead(args),
        Command::TestGaps(args) => run_test_gaps(args),
        Command::Suggest(args) => run_suggest(args),
        Command::Boundary(args) => run_boundary(args),
        Command::ExtractCheck(args) => run_extract_check(args),
        Command::MoveCheck(args) => run_move_check(args),
        Command::RenameCheck(args) => run_rename_check(args),
        Command::SplitCheck(args) => run_split_check(args),
        Command::TestScaffold(args) => run_test_scaffold(args),
        Command::SafeSteps(args) => run_safe_steps(args),
        Command::VerifyRefactor(args) => run_verify_refactor(args),
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
    let symbol_query = parse_symbol_query(&args.symbol);
    let scope = query_scope_for_find_refs(args.code_only, args.exclude_tests, args.filters.scope);
    let mut matches = find_matches_scoped(&store.db_path, &symbol_query.lookup_symbol, &scope)?;
    filter_query_matches(&mut matches, &args.filters);
    apply_query_match_ranking_preferences(
        &mut matches,
        &symbol_query.preferred_file,
        &symbol_query.preferred_lang,
        args.filters.include_fixtures,
    );
    if let Some(max_results) = args.max_results {
        matches.truncate(u32_to_usize(max_results));
    }
    if args.json {
        output::print_query_json("find", &symbol_query.lookup_symbol, &matches)?;
    } else if args.compact {
        output::print_query_compact(&matches);
    } else {
        output::print_query("find", &symbol_query.lookup_symbol, &matches);
        if matches.is_empty() {
            let suggestions = suggest_similar_symbols(&store.db_path, &symbol_query.lookup_symbol)?;
            if !suggestions.is_empty() {
                output::print_did_you_mean(&suggestions);
            }
        }
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
///     compact: false,
///     filters: crate::cli::SymbolFilterArgs::default(),
/// };
/// let _ = run_refs(args);
/// ```
fn run_refs(args: crate::cli::RefsArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let symbol_query = parse_symbol_query(&args.symbol);
    let scope = query_scope_for_find_refs(args.code_only, args.exclude_tests, args.filters.scope);
    let mut matches = refs_matches_scoped(&store.db_path, &symbol_query.lookup_symbol, &scope)?;
    filter_query_matches(&mut matches, &args.filters);
    apply_query_match_ranking_preferences(
        &mut matches,
        &symbol_query.preferred_file,
        &symbol_query.preferred_lang,
        args.filters.include_fixtures,
    );
    if let Some(max_results) = args.max_results {
        matches.truncate(u32_to_usize(max_results));
    }
    if args.json {
        output::print_query_json("refs", &symbol_query.lookup_symbol, &matches)?;
    } else if args.compact {
        output::print_query_compact(&matches);
    } else {
        output::print_refs_grouped(&symbol_query.lookup_symbol, &matches);
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
/// let args = QueryArgs {
///     repo: ".".into(),
///     symbol: "my_crate::foo".into(),
///     json: false,
///     filters: crate::cli::SymbolFilterArgs::default(),
/// };
/// run_impact(args).unwrap();
/// ```
fn run_impact(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let symbol_query = parse_symbol_query(&args.symbol);
    let mut matches = impact_matches(&store.db_path, &symbol_query.lookup_symbol)?;
    filter_impact_matches(&mut matches, &args.filters);
    apply_impact_ranking_preferences(&mut matches, args.filters.include_fixtures);
    if args.json {
        output::print_impact_json(&symbol_query.lookup_symbol, &matches)?;
    } else {
        output::print_impact(&symbol_query.lookup_symbol, &matches);
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
        anyhow::bail!("no changed files: provide --changed-file, --since, or --unstaged");
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
        anyhow::bail!("no changed files: provide --changed-file, --since, or --unstaged");
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
    let symbol_query = parse_symbol_query(&args.symbol);
    let mut matches = explain_symbol(
        &store.db_path,
        &symbol_query.lookup_symbol,
        args.include_snippets,
    )?;
    filter_explain_matches(&mut matches, &args.filters);
    if args.json {
        output::print_explain_json(&symbol_query.lookup_symbol, args.include_snippets, &matches)?;
    } else if args.compact {
        output::print_explain_compact(&matches);
    } else {
        output::print_explain(&symbol_query.lookup_symbol, &matches);
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

fn run_hotspots(args: crate::cli::HotspotsArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let entries = hotspots(&store.db_path, args.limit)?;
    if args.json {
        output::print_hotspots_json(&entries)?;
    } else {
        output::print_hotspots(&entries);
    }
    Ok(())
}

fn run_call_path(args: crate::cli::CallPathArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let path = find_call_path(&store.db_path, &args.from, &args.to, args.max_depth)?;
    if args.json {
        output::print_call_path_json(&args.from, &args.to, &path)?;
    } else {
        output::print_call_path(&args.from, &args.to, &path);
    }
    Ok(())
}

fn run_related(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let symbol_query = parse_symbol_query(&args.symbol);
    let mut results = related_symbols(&store.db_path, &symbol_query.lookup_symbol)?;
    filter_related_symbols(&mut results, &args.filters);
    sort_related_symbols_by_path_preferences(&mut results, args.filters.include_fixtures);
    if args.json {
        output::print_related_json(&symbol_query.lookup_symbol, &results)?;
    } else {
        output::print_related(&symbol_query.lookup_symbol, &results);
    }
    Ok(())
}

fn run_callers(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let symbol_query = parse_symbol_query(&args.symbol);
    let mut matches = callers_of(&store.db_path, &symbol_query.lookup_symbol)?;
    filter_edge_matches(&mut matches, &args.filters);
    sort_edge_matches_by_path_preferences(&mut matches, args.filters.include_fixtures);
    if args.json {
        output::print_edges_json("callers", &symbol_query.lookup_symbol, &matches)?;
    } else {
        output::print_edges("callers", &symbol_query.lookup_symbol, &matches);
    }
    Ok(())
}

fn run_callees(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let symbol_query = parse_symbol_query(&args.symbol);
    let mut matches = callees_of(&store.db_path, &symbol_query.lookup_symbol)?;
    filter_edge_matches(&mut matches, &args.filters);
    sort_edge_matches_by_path_preferences(&mut matches, args.filters.include_fixtures);
    if args.json {
        output::print_edges_json("callees", &symbol_query.lookup_symbol, &matches)?;
    } else {
        output::print_edges("callees", &symbol_query.lookup_symbol, &matches);
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ParsedSymbolQuery {
    lookup_symbol: String,
    preferred_file: Option<String>,
    preferred_lang: Option<String>,
}

fn parse_symbol_query(raw_symbol: &str) -> ParsedSymbolQuery {
    let mut parsed = ParsedSymbolQuery {
        lookup_symbol: raw_symbol.to_string(),
        preferred_file: None,
        preferred_lang: None,
    };
    let Some((raw_prefix, raw_symbol_name)) = raw_symbol.rsplit_once("::") else {
        return parsed;
    };
    if raw_symbol_name.trim().is_empty() {
        return parsed;
    }

    let prefix = normalize_path(raw_prefix);
    let symbol_name = raw_symbol_name.trim();
    let maybe_lang_prefix = known_language_prefix(&prefix);
    let maybe_file_prefix = maybe_lang_prefix
        .map(|(_, rest)| rest)
        .or(Some(prefix.as_str()))
        .filter(|candidate| looks_like_file_path(candidate));
    if maybe_lang_prefix.is_none() && maybe_file_prefix.is_none() {
        return parsed;
    }

    parsed.lookup_symbol = symbol_name.to_string();
    if let Some((lang, _)) = maybe_lang_prefix {
        parsed.preferred_lang = Some(lang.to_string());
    }
    if let Some(file_prefix) = maybe_file_prefix {
        parsed.preferred_file = Some(normalize_path(file_prefix));
    }
    parsed
}

fn query_scope_for_find_refs(
    code_only: bool,
    exclude_tests: bool,
    scope: crate::cli::QueryScopeKind,
) -> QueryScope {
    let mut query_scope = QueryScope::from_flags(code_only, exclude_tests);
    match scope {
        crate::cli::QueryScopeKind::All => {}
        crate::cli::QueryScopeKind::Production => {
            query_scope.path_mode = QueryPathMode::CodeOnly;
            query_scope.test_mode = QueryTestMode::ExcludeTests;
        }
        crate::cli::QueryScopeKind::Tests => {
            query_scope.test_mode = QueryTestMode::IncludeTests;
        }
    }
    query_scope
}

fn include_path_by_scope(path: &str, scope: crate::cli::QueryScopeKind) -> bool {
    match scope {
        crate::cli::QueryScopeKind::All => true,
        crate::cli::QueryScopeKind::Production => {
            is_code_file_path(path) && !is_test_like_path(path)
        }
        crate::cli::QueryScopeKind::Tests => is_test_like_path(path),
    }
}

fn filter_query_matches(
    matches: &mut Vec<crate::query::QueryMatch>,
    filters: &crate::cli::SymbolFilterArgs,
) {
    matches.retain(|item| path_passes_filters(&item.file_path, filters));
}

fn filter_impact_matches(matches: &mut Vec<ImpactMatch>, filters: &crate::cli::SymbolFilterArgs) {
    matches.retain(|item| path_passes_filters(&item.file_path, filters));
}

fn filter_edge_matches(
    matches: &mut Vec<crate::query::EdgeMatch>,
    filters: &crate::cli::SymbolFilterArgs,
) {
    matches.retain(|item| path_passes_filters(&item.file_path, filters));
}

fn filter_related_symbols(
    matches: &mut Vec<crate::query::RelatedSymbol>,
    filters: &crate::cli::SymbolFilterArgs,
) {
    matches.retain(|item| path_passes_filters(&item.file_path, filters));
}

fn filter_explain_matches(matches: &mut Vec<ExplainMatch>, filters: &crate::cli::SymbolFilterArgs) {
    matches.retain(|item| {
        if !path_passes_filters(&item.file_path, filters) {
            return false;
        }
        filters
            .lang
            .as_deref()
            .map_or(true, |lang| item.language.eq_ignore_ascii_case(lang))
    });
}

fn path_passes_filters(path: &str, filters: &crate::cli::SymbolFilterArgs) -> bool {
    let normalized_path = normalize_path(path);
    if !include_path_by_scope(&normalized_path, filters.scope) {
        return false;
    }
    if filters
        .exclude_globs
        .iter()
        .any(|glob| path_matches_glob(&normalized_path, &normalize_path(glob)))
    {
        return false;
    }
    if let Some(file_filter) = filters.file.as_deref() {
        if normalized_path != normalize_path(file_filter) {
            return false;
        }
    }
    if let Some(lang_filter) = filters.lang.as_deref() {
        if !file_language(&normalized_path).eq_ignore_ascii_case(lang_filter) {
            return false;
        }
    }
    true
}

fn apply_query_match_ranking_preferences(
    matches: &mut [crate::query::QueryMatch],
    preferred_file: &Option<String>,
    preferred_lang: &Option<String>,
    include_fixtures: bool,
) {
    for item in matches.iter_mut() {
        let mut adjusted_score = item.score;
        if !include_fixtures {
            if is_fixture_path(&item.file_path) {
                adjusted_score -= 0.25;
            } else if is_test_like_path(&item.file_path) {
                adjusted_score -= 0.10;
            }
        }
        if preferred_file
            .as_deref()
            .is_some_and(|preferred| normalize_path(&item.file_path) == normalize_path(preferred))
        {
            adjusted_score += 0.30;
        }
        if preferred_lang
            .as_deref()
            .is_some_and(|preferred| file_language(&item.file_path).eq_ignore_ascii_case(preferred))
        {
            adjusted_score += 0.15;
        }
        item.score = adjusted_score.max(0.0);
    }

    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                fallback_path_class_rank(&left.file_path)
                    .cmp(&fallback_path_class_rank(&right.file_path)),
            )
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.why_matched.cmp(&right.why_matched))
    });
}

fn fallback_path_class_rank(file_path: &str) -> u8 {
    if is_code_file_path(file_path) && !is_test_like_path(file_path) {
        0
    } else if is_test_like_path(file_path) {
        1
    } else {
        2
    }
}

fn path_priority_rank(file_path: &str, include_fixtures: bool) -> u8 {
    if is_code_file_path(file_path) && !is_test_like_path(file_path) {
        0
    } else if is_test_like_path(file_path) {
        if !include_fixtures && is_fixture_path(file_path) {
            2
        } else {
            1
        }
    } else {
        3
    }
}

fn score_with_fixture_penalty(score: f64, file_path: &str, include_fixtures: bool) -> f64 {
    if include_fixtures {
        return score;
    }
    if is_fixture_path(file_path) {
        (score - 0.25).max(0.0)
    } else if is_test_like_path(file_path) {
        (score - 0.10).max(0.0)
    } else {
        score
    }
}

fn apply_impact_ranking_preferences(matches: &mut [ImpactMatch], include_fixtures: bool) {
    for item in matches.iter_mut() {
        item.score = score_with_fixture_penalty(item.score, &item.file_path, include_fixtures);
    }
    matches.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                path_priority_rank(&left.file_path, include_fixtures)
                    .cmp(&path_priority_rank(&right.file_path, include_fixtures)),
            )
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.relationship.cmp(&right.relationship))
    });
}

fn sort_edge_matches_by_path_preferences(
    matches: &mut [crate::query::EdgeMatch],
    include_fixtures: bool,
) {
    matches.sort_by(|left, right| {
        path_priority_rank(&left.file_path, include_fixtures)
            .cmp(&path_priority_rank(&right.file_path, include_fixtures))
            .then(left.file_path.cmp(&right.file_path))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.kind.cmp(&right.kind))
    });
}

fn sort_related_symbols_by_path_preferences(
    matches: &mut [crate::query::RelatedSymbol],
    include_fixtures: bool,
) {
    matches.sort_by(|left, right| {
        path_priority_rank(&left.file_path, include_fixtures)
            .cmp(&path_priority_rank(&right.file_path, include_fixtures))
            .then(left.file_path.cmp(&right.file_path))
            .then(left.symbol.cmp(&right.symbol))
            .then(left.relationship.cmp(&right.relationship))
            .then(left.kind.cmp(&right.kind))
    });
}

fn file_language(file_path: &str) -> &'static str {
    let normalized = normalize_path(file_path);
    if normalized.ends_with(".rs") {
        "rust"
    } else if normalized.ends_with(".ts") || normalized.ends_with(".tsx") {
        "typescript"
    } else if normalized.ends_with(".py") {
        "python"
    } else if normalized.ends_with(".go") {
        "go"
    } else {
        "unknown"
    }
}

fn is_code_file_path(file_path: &str) -> bool {
    let normalized = normalize_path(file_path);
    normalized.ends_with(".rs")
        || normalized.ends_with(".ts")
        || normalized.ends_with(".tsx")
        || normalized.ends_with(".py")
        || normalized.ends_with(".go")
}

fn is_test_like_path(file_path: &str) -> bool {
    let normalized = normalize_path(file_path);
    normalized.starts_with("tests/")
        || normalized.contains("/tests/")
        || std::path::Path::new(&normalized)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(is_test_like_file_name)
}

fn is_test_like_file_name(file_name: &str) -> bool {
    file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.ends_with("_tests.py")
        || (file_name.starts_with("test_") && file_name.ends_with(".py"))
        || file_name.ends_with(".test.ts")
        || file_name.ends_with(".test.tsx")
        || file_name.ends_with(".spec.ts")
        || file_name.ends_with(".spec.tsx")
}

fn is_fixture_path(file_path: &str) -> bool {
    let normalized = normalize_path(file_path);
    normalized.starts_with("tests/fixtures/") || normalized.contains("/tests/fixtures/")
}

fn path_matches_glob(path: &str, glob: &str) -> bool {
    let normalized_path = normalize_path(path);
    let normalized_glob = normalize_path(glob);
    if let Some(prefix) = normalized_glob.strip_suffix("/**") {
        let normalized_prefix = prefix.trim_end_matches('/');
        return normalized_path == normalized_prefix
            || normalized_path.starts_with(&format!("{normalized_prefix}/"));
    }
    if !normalized_glob.contains('*') {
        return normalized_path == normalized_glob;
    }

    let mut remaining_path = normalized_path.as_str();
    let mut first_segment = true;
    for segment in normalized_glob.split('*') {
        if segment.is_empty() {
            continue;
        }
        if first_segment {
            if !remaining_path.starts_with(segment) {
                return false;
            }
            remaining_path = &remaining_path[segment.len()..];
            first_segment = false;
            continue;
        }
        if let Some(index) = remaining_path.find(segment) {
            remaining_path = &remaining_path[(index + segment.len())..];
        } else {
            return false;
        }
    }
    normalized_glob.ends_with('*') || remaining_path.is_empty()
}

fn normalize_path(input: &str) -> String {
    let normalized = input.replace('\\', "/");
    normalized
        .trim()
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

fn known_language_prefix(prefix: &str) -> Option<(&str, &str)> {
    let (lang, rest) = prefix.split_once(':')?;
    match lang {
        "rust" | "python" | "typescript" | "go" => Some((lang, rest)),
        _ => None,
    }
}

fn looks_like_file_path(value: &str) -> bool {
    let normalized = normalize_path(value);
    normalized.contains('/')
        || normalized.ends_with(".rs")
        || normalized.ends_with(".py")
        || normalized.ends_with(".ts")
        || normalized.ends_with(".tsx")
        || normalized.ends_with(".go")
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

fn run_health(args: crate::cli::HealthArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let report =
        crate::query::diagnostics::health_report(&store.db_path, args.top, args.threshold)?;

    let baseline_path = args.repo.join(".repo-scout").join("health-baseline.json");
    if args.save_baseline {
        if let Some(parent) = baseline_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&report)?;
        fs::write(&baseline_path, serialized)?;
    }
    if args.diff {
        if baseline_path.exists() {
            let raw = fs::read_to_string(&baseline_path)?;
            let baseline: crate::query::diagnostics::HealthReport = serde_json::from_str(&raw)?;
            print_health_diff(&baseline, &report);
        } else {
            println!(
                "Health comparison: no baseline found at {}",
                baseline_path.display()
            );
        }
        return Ok(());
    }

    if args.json {
        output::print_health_json(&report)?;
    } else {
        let (show_files, show_functions) = if args.large_files && args.large_functions {
            (true, true)
        } else {
            (!args.large_functions, !args.large_files)
        };
        output::print_health(&report, show_files, show_functions);
    }
    Ok(())
}

fn run_tree(args: crate::cli::TreeArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let tree_args = crate::query::orientation::TreeReportArgs {
        depth: args.depth,
        no_deps: args.no_deps,
        focus: args.focus,
        show_symbols: args.symbols,
    };
    let report = crate::query::orientation::tree_report(&store.db_path, &tree_args)?;
    if args.json {
        output::print_tree_json(&report)?;
    } else {
        output::print_tree(&report);
    }
    Ok(())
}

fn run_orient(args: crate::cli::OrientArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let orient_args = crate::query::orientation::OrientReportArgs {
        depth: args.depth,
        top: args.top,
    };
    let report = crate::query::orientation::orient_report(&store.db_path, &orient_args)?;
    if args.json {
        output::print_orient_json(&report)?;
    } else {
        output::print_orient(&report);
    }
    Ok(())
}

fn run_circular(args: crate::cli::CircularArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let report = crate::query::diagnostics::detect_circular_deps(&store.db_path, args.max_length)?;
    if args.json {
        output::print_circular_json(&report)?;
    } else {
        output::print_circular(&report);
    }
    Ok(())
}

fn run_anatomy(_args: crate::cli::AnatomyArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let report = crate::query::diagnostics::file_anatomy(&store.db_path, &args.file)?;
    if args.json {
        output::print_anatomy_json(&report)?;
    } else {
        output::print_anatomy(&report);
    }
    Ok(())
}

fn run_coupling(_args: crate::cli::CouplingArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let entries = crate::query::diagnostics::coupling_report(&store.db_path, args.limit)?;
    if args.json {
        output::print_coupling_json(&entries)?;
    } else {
        output::print_coupling(&entries);
    }
    Ok(())
}

fn run_dead(_args: crate::cli::DeadArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let mut entries = crate::query::diagnostics::dead_symbols(&store.db_path)?;
    entries.retain(|entry| path_passes_filters(&entry.file_path, &args.filters));
    if args.json {
        output::print_dead_json(&entries)?;
    } else {
        output::print_dead(&entries);
    }
    Ok(())
}

fn run_test_gaps(_args: crate::cli::TestGapsArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let mut report = crate::query::diagnostics::test_gap_analysis(&store.db_path, &args.target)?;
    if let Some(min_risk) = args.min_risk {
        report.uncovered.retain(|entry| entry.risk >= min_risk);
    }
    if args.json {
        output::print_test_gaps_json(&report)?;
    } else {
        output::print_test_gaps(&report);
    }
    Ok(())
}

fn run_suggest(_args: crate::cli::SuggestArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let suggestions = crate::query::diagnostics::suggest_refactorings(
        &store.db_path,
        args.top,
        args.safe_only,
        args.min_score,
    )?;
    if args.json {
        output::print_suggest_json(&suggestions)?;
    } else {
        output::print_suggest(&suggestions);
    }
    Ok(())
}

fn run_boundary(_args: crate::cli::BoundaryArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let report = crate::query::planning::boundary_analysis(&store.db_path, &args.file)?;
    if args.json {
        output::print_boundary_json(&report)?;
    } else {
        output::print_boundary(&report, args.public_only);
    }
    Ok(())
}

fn run_extract_check(_args: crate::cli::ExtractCheckArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let range = crate::query::planning::parse_line_range(&args.lines)?;
    let report = crate::query::planning::extract_check(&store.db_path, &args.symbol, range)?;
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "extract-check",
            "symbol": report.symbol,
            "file_path": report.file_path,
            "function_start_line": report.function_start_line,
            "function_end_line": report.function_end_line,
            "extract_start_line": report.extract_start_line,
            "extract_end_line": report.extract_end_line,
            "estimated_line_count": report.estimated_line_count,
            "signature": report.signature,
            "warnings": report.warnings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "Extract analysis for {} lines {}-{}:",
            report.symbol, report.extract_start_line, report.extract_end_line
        );
        println!("  File: {}", report.file_path);
        println!(
            "  Function bounds: {}-{}",
            report.function_start_line, report.function_end_line
        );
        println!(
            "  Estimated extracted size: {} lines",
            report.estimated_line_count
        );
        if !report.warnings.is_empty() {
            println!("  Warnings:");
            for warning in report.warnings {
                println!("    - {warning}");
            }
        }
    }
    Ok(())
}

fn run_move_check(_args: crate::cli::MoveCheckArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let refs = refs_matches_scoped(&store.db_path, &args.symbol, &QueryScope::default())?;
    let impact = impact_matches(&store.db_path, &args.symbol)?;
    let tests = tests_for_symbol(&store.db_path, &args.symbol, false)?;
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "move-check",
            "symbol": args.symbol,
            "destination": args.to,
            "reference_count": refs.len(),
            "impact_count": impact.len(),
            "test_count": tests.len(),
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Move check for {} -> {}", args.symbol, args.to);
        println!("  references to update: {}", refs.len());
        println!("  impacted dependents: {}", impact.len());
        println!("  associated tests: {}", tests.len());
    }
    Ok(())
}

fn run_rename_check(_args: crate::cli::RenameCheckArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let refs = refs_matches_scoped(&store.db_path, &args.symbol, &QueryScope::default())?;
    let connection = Connection::open(&store.db_path)?;
    let text_hits: u32 = connection.query_row(
        "SELECT COUNT(*) FROM text_occurrences WHERE symbol = ?1",
        [args.symbol.as_str()],
        |row| row.get(0),
    )?;
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "rename-check",
            "symbol": args.symbol,
            "new_name": args.to,
            "ast_reference_count": refs.len(),
            "text_occurrence_count": text_hits,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Rename check for {} -> {}", args.symbol, args.to);
        println!("  AST references: {}", refs.len());
        println!("  text occurrences: {text_hits}");
    }
    Ok(())
}

fn run_split_check(_args: crate::cli::SplitCheckArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let anatomy = crate::query::diagnostics::file_anatomy(&store.db_path, &args.file)?;
    let mode = if args.auto {
        "auto"
    } else if args.groups.is_some() {
        "manual"
    } else {
        "none"
    };
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "split-check",
            "file": args.file,
            "mode": mode,
            "symbol_count": anatomy.total_symbols,
            "function_count": anatomy.function_count,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Split check for {} ({mode})", args.file);
        println!("  symbols: {}", anatomy.total_symbols);
        println!("  functions: {}", anatomy.function_count);
    }
    Ok(())
}

fn run_test_scaffold(_args: crate::cli::TestScaffoldArgs) -> anyhow::Result<()> {
    let args = _args;
    let store = ensure_store(&args.repo)?;
    let explain = explain_symbol(&store.db_path, &args.symbol, false)?;
    let tests = tests_for_symbol(&store.db_path, &args.symbol, true)?;
    let signature = explain
        .first()
        .and_then(|entry| entry.signature.clone())
        .unwrap_or_else(|| "<unknown>".to_string());
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "test-scaffold",
            "symbol": args.symbol,
            "signature": signature,
            "existing_tests": tests,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Test scaffold for {}:", args.symbol);
        println!("  signature: {signature}");
        println!("  existing tests: {}", tests.len());
    }
    Ok(())
}

fn run_safe_steps(_args: crate::cli::SafeStepsArgs) -> anyhow::Result<()> {
    let args = _args;
    let action = match args.action {
        crate::cli::SafeStepsAction::Extract => "extract",
        crate::cli::SafeStepsAction::Move => "move",
        crate::cli::SafeStepsAction::Rename => "rename",
        crate::cli::SafeStepsAction::Split => "split",
    };
    let steps = vec![
        format!("Step 1: Stage the {action} change for {}", args.symbol),
        "Step 2: Run targeted checks".to_string(),
        "Step 3: Run full test suite".to_string(),
    ];
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "safe-steps",
            "symbol": args.symbol,
            "action": action,
            "steps": steps,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!("Safe refactoring steps for {} ({action}):", args.symbol);
        for step in steps {
            println!("  {step}");
        }
    }
    Ok(())
}

fn run_verify_refactor(_args: crate::cli::VerifyRefactorArgs) -> anyhow::Result<()> {
    let args = _args;
    let report = crate::query::verification::verify_refactor_report(
        &args.repo,
        &args.before,
        args.after.as_deref(),
    )?;
    if args.strict && (!report.warnings.is_empty() || !report.changed_files.is_empty()) {
        anyhow::bail!(
            "verify-refactor strict mode failed: {}",
            if !report.warnings.is_empty() {
                report.warnings.join("; ")
            } else {
                format!("{} changed file(s) detected", report.changed_files.len())
            }
        );
    }
    if args.json {
        let payload = serde_json::json!({
            "schema_version": output::JSON_SCHEMA_VERSION_V2,
            "command": "verify-refactor",
            "before": report.before,
            "after": report.after,
            "changed_files": report.changed_files,
            "warnings": report.warnings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "Refactoring verification ({} -> {}):",
            report.before, report.after
        );
        println!("  Changed files: {}", report.changed_files.len());
        if !report.changed_files.is_empty() {
            for file in &report.changed_files {
                println!("    - {file}");
            }
        }
        if !report.warnings.is_empty() {
            println!("  Warnings:");
            for warning in report.warnings {
                println!("    - {warning}");
            }
        } else if report.changed_files.is_empty() {
            println!("  No discrepancies detected.");
        }
    }
    Ok(())
}

fn print_health_diff(
    baseline: &crate::query::diagnostics::HealthReport,
    current: &crate::query::diagnostics::HealthReport,
) {
    let baseline_file = baseline.largest_files.first().map(|entry| entry.line_count);
    let current_file = current.largest_files.first().map(|entry| entry.line_count);
    let baseline_fn = baseline
        .largest_functions
        .first()
        .map(|entry| entry.line_count);
    let current_fn = current
        .largest_functions
        .first()
        .map(|entry| entry.line_count);

    println!("Health comparison:");
    println!(
        "  Largest file line count: {} -> {}",
        baseline_file
            .map(|value| value.to_string())
            .unwrap_or_else(|| "?".to_string()),
        current_file
            .map(|value| value.to_string())
            .unwrap_or_else(|| "?".to_string())
    );
    println!(
        "  Largest function line count: {} -> {}",
        baseline_fn
            .map(|value| value.to_string())
            .unwrap_or_else(|| "?".to_string()),
        current_fn
            .map(|value| value.to_string())
            .unwrap_or_else(|| "?".to_string())
    );
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
        apply_impact_ranking_preferences, normalize_changed_file, parse_changed_line_spec,
        run_call_path, run_callees, run_callers, run_context, run_deps, run_diff_impact,
        run_explain, run_find, run_hotspots, run_impact, run_index, run_outline, run_refs,
        run_related, run_snippet, run_status, run_summary_cmd, run_tests_for, run_verify_plan,
        sort_edge_matches_by_path_preferences, sort_related_symbols_by_path_preferences,
    };
    use crate::cli::{
        CallPathArgs, ContextArgs, DepsArgs, DiffImpactArgs, ExplainArgs, FindArgs, HotspotsArgs,
        OutlineArgs, QueryArgs, RefsArgs, RepoArgs, SnippetArgs, SymbolFilterArgs, TestsForArgs,
        VerifyPlanArgs,
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
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("find json should succeed");
        run_find(FindArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            code_only: false,
            exclude_tests: false,
            max_results: None,
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("find text should succeed");

        run_refs(RefsArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            code_only: false,
            exclude_tests: false,
            max_results: Some(10),
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("refs json should succeed");
        run_refs(RefsArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            code_only: true,
            exclude_tests: false,
            max_results: None,
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("refs text should succeed");

        run_impact(QueryArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("impact json should succeed");
        run_impact(QueryArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            filters: SymbolFilterArgs::default(),
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
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("explain json should succeed");
        run_explain(ExplainArgs {
            symbol: "run_find".to_string(),
            repo: repo_path.clone(),
            json: false,
            include_snippets: false,
            compact: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("explain text should succeed");

        // Snippet (json + text)
        run_snippet(SnippetArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: true,
            context: 0,
        })
        .expect("snippet json");
        run_snippet(SnippetArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            context: 2,
        })
        .expect("snippet text");

        // Outline (json + text)
        run_outline(OutlineArgs {
            file: "src/lib.rs".into(),
            repo: repo_path.clone(),
            json: true,
        })
        .expect("outline json");
        run_outline(OutlineArgs {
            file: "src/lib.rs".into(),
            repo: repo_path.clone(),
            json: false,
        })
        .expect("outline text");

        // Summary
        run_summary_cmd(RepoArgs {
            repo: repo_path.clone(),
        })
        .expect("summary");

        // Deps (json + text)
        run_deps(DepsArgs {
            file: "src/lib.rs".into(),
            repo: repo_path.clone(),
            json: true,
        })
        .expect("deps json");
        run_deps(DepsArgs {
            file: "src/lib.rs".into(),
            repo: repo_path.clone(),
            json: false,
        })
        .expect("deps text");

        // Hotspots (json + text)
        run_hotspots(HotspotsArgs {
            repo: repo_path.clone(),
            json: true,
            limit: 5,
        })
        .expect("hotspots json");
        run_hotspots(HotspotsArgs {
            repo: repo_path.clone(),
            json: false,
            limit: 5,
        })
        .expect("hotspots text");

        // Callers (json + text)
        run_callers(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("callers json");
        run_callers(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("callers text");

        // Callees (json + text)
        run_callees(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("callees json");
        run_callees(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("callees text");

        // Call-path (json + text)
        run_call_path(CallPathArgs {
            from: "run_find".into(),
            to: "run_refs".into(),
            repo: repo_path.clone(),
            json: true,
            max_depth: 5,
        })
        .expect("call-path json");
        run_call_path(CallPathArgs {
            from: "run_find".into(),
            to: "run_refs".into(),
            repo: repo_path.clone(),
            json: false,
            max_depth: 5,
        })
        .expect("call-path text");

        // Related (json + text)
        run_related(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("related json");
        run_related(QueryArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            filters: SymbolFilterArgs::default(),
        })
        .expect("related text");

        // Explain compact
        run_explain(ExplainArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            include_snippets: false,
            compact: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("explain compact");

        // Find compact
        run_find(FindArgs {
            symbol: "run_find".into(),
            repo: repo_path.clone(),
            json: false,
            code_only: false,
            exclude_tests: false,
            max_results: None,
            compact: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("find compact");

        // Refs compact
        run_refs(RefsArgs {
            symbol: "run_find".into(),
            repo: repo_path,
            json: false,
            code_only: false,
            exclude_tests: false,
            max_results: None,
            compact: true,
            filters: SymbolFilterArgs::default(),
        })
        .expect("refs compact");
    }

    #[test]
    fn ranking_helpers_penalize_fixture_and_test_paths_by_default() {
        let mut impact_rows = vec![
            crate::query::ImpactMatch {
                symbol: "fixture_user".into(),
                kind: "function".into(),
                file_path: "tests/fixtures/sample.rs".into(),
                line: 1,
                column: 1,
                distance: 1,
                relationship: "called_by".into(),
                confidence: "graph_likely".into(),
                score: 0.95,
            },
            crate::query::ImpactMatch {
                symbol: "test_user".into(),
                kind: "function".into(),
                file_path: "tests/user_test.rs".into(),
                line: 1,
                column: 1,
                distance: 1,
                relationship: "called_by".into(),
                confidence: "graph_likely".into(),
                score: 0.93,
            },
            crate::query::ImpactMatch {
                symbol: "prod_user".into(),
                kind: "function".into(),
                file_path: "src/lib.rs".into(),
                line: 1,
                column: 1,
                distance: 1,
                relationship: "called_by".into(),
                confidence: "graph_likely".into(),
                score: 0.90,
            },
        ];
        apply_impact_ranking_preferences(&mut impact_rows, false);
        assert_eq!(impact_rows[0].file_path, "src/lib.rs");

        let mut include_fixture_rows = impact_rows.clone();
        include_fixture_rows[0].score = 0.95;
        include_fixture_rows[0].file_path = "tests/fixtures/sample.rs".into();
        include_fixture_rows[1].score = 0.93;
        include_fixture_rows[1].file_path = "tests/user_test.rs".into();
        include_fixture_rows[2].score = 0.90;
        include_fixture_rows[2].file_path = "src/lib.rs".into();
        apply_impact_ranking_preferences(&mut include_fixture_rows, true);
        assert_eq!(
            include_fixture_rows[0].file_path,
            "tests/fixtures/sample.rs"
        );
    }

    #[test]
    fn ranking_helpers_sort_edges_and_related_with_fixture_penalty() {
        let mut edges = vec![
            crate::query::EdgeMatch {
                file_path: "tests/fixtures/sample.rs".into(),
                symbol: "fixture_user".into(),
                kind: "function".into(),
                line: 1,
                column: 1,
                confidence: 0.95,
            },
            crate::query::EdgeMatch {
                file_path: "tests/user_test.rs".into(),
                symbol: "test_user".into(),
                kind: "function".into(),
                line: 1,
                column: 1,
                confidence: 0.95,
            },
            crate::query::EdgeMatch {
                file_path: "src/lib.rs".into(),
                symbol: "prod_user".into(),
                kind: "function".into(),
                line: 1,
                column: 1,
                confidence: 0.95,
            },
        ];
        sort_edge_matches_by_path_preferences(&mut edges, false);
        assert_eq!(edges[0].file_path, "src/lib.rs");
        assert_eq!(edges[2].file_path, "tests/fixtures/sample.rs");

        let mut related = vec![
            crate::query::RelatedSymbol {
                symbol: "fixture_sibling".into(),
                file_path: "tests/fixtures/sample.rs".into(),
                kind: "function".into(),
                relationship: "sibling".into(),
            },
            crate::query::RelatedSymbol {
                symbol: "prod_sibling".into(),
                file_path: "src/lib.rs".into(),
                kind: "function".into(),
                relationship: "sibling".into(),
            },
        ];
        sort_related_symbols_by_path_preferences(&mut related, false);
        assert_eq!(related[0].file_path, "src/lib.rs");
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
