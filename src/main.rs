mod cli;
mod output;
mod store;

use clap::Parser;

use crate::cli::{Cli, Command};
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
        Command::Find(args) => run_query("find", args),
        Command::Refs(args) => run_query("refs", args),
    }
}

fn run_index(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    output::print_index(&store.db_path, store.schema_version, 0);
    Ok(())
}

fn run_status(args: crate::cli::RepoArgs) -> anyhow::Result<()> {
    let store = ensure_store(&args.repo)?;
    output::print_status(&store.db_path, store.schema_version);
    Ok(())
}

fn run_query(command: &str, args: crate::cli::QueryArgs) -> anyhow::Result<()> {
    let _ = ensure_store(&args.repo)?;
    output::print_query(command, &args.symbol, 0);
    Ok(())
}
