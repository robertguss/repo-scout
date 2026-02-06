mod cli;
mod indexer;
mod output;
mod query;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::indexer::index_repository;
use crate::query::{context_matches, find_matches, impact_matches, refs_matches};
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
