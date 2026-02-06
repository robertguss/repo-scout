use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "repo-scout")]
#[command(about = "Hybrid repository indexing and query CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Index(RepoArgs),
    Status(RepoArgs),
    Find(QueryArgs),
    Refs(QueryArgs),
}

#[derive(Debug, Args)]
pub struct RepoArgs {
    #[arg(long)]
    pub repo: PathBuf,
}

#[derive(Debug, Args)]
pub struct QueryArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}
