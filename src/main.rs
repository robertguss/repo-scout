mod cli;
mod indexer;
mod output;
mod query;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::indexer::index_repository;
use crate::query::{
    context_matches, find_matches, impact_matches, refs_matches, tests_for_symbol,
    verify_plan_for_changed_files,
};
use crate::store::ensure_store;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

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

fn run_find(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = find_matches(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_query_json("find", &args.symbol, &matches)?;
    } else {
        output::print_query("find", &args.symbol, &matches);
    }
    Ok(())
}

fn run_refs(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = refs_matches(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_query_json("refs", &args.symbol, &matches)?;
    } else {
        output::print_query("refs", &args.symbol, &matches);
    }
    Ok(())
}

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

fn run_context(args: crate::cli::ContextArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let matches = context_matches(&store.db_path, &args.task, args.budget)?;
    if args.json {
        output::print_context_json(&args.task, args.budget, &matches)?;
    } else {
        output::print_context(&args.task, args.budget, &matches);
    }
    Ok(())
}

fn run_tests_for(args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let targets = tests_for_symbol(&store.db_path, &args.symbol)?;
    if args.json {
        output::print_tests_for_json(&args.symbol, &targets)?;
    } else {
        output::print_tests_for(&args.symbol, &targets);
    }
    Ok(())
}

fn run_verify_plan(args: crate::cli::VerifyPlanArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    let mut changed_files = args
        .changed_files
        .iter()
        .map(|path| normalize_changed_file(&args.repo, path))
        .collect::<Vec<_>>();
    changed_files.sort();
    changed_files.dedup();

    let steps = verify_plan_for_changed_files(&store.db_path, &changed_files)?;
    if args.json {
        output::print_verify_plan_json(&changed_files, &steps)?;
    } else {
        output::print_verify_plan(&changed_files, &steps);
    }
    Ok(())
}

fn normalize_changed_file(repo_root: &std::path::Path, changed_file: &str) -> String {
    let candidate = std::path::PathBuf::from(changed_file);
    let absolute_repo_root = std::fs::canonicalize(repo_root).unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|cwd| cwd.join(repo_root))
            .unwrap_or_else(|_| repo_root.to_path_buf())
    });
    let normalized = if candidate.is_absolute() {
        candidate
            .strip_prefix(&absolute_repo_root)
            .map(|path| path.to_path_buf())
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
