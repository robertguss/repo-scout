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
    Find(FindArgs),
    Refs(RefsArgs),
    Impact(QueryArgs),
    Context(ContextArgs),
    TestsFor(TestsForArgs),
    VerifyPlan(VerifyPlanArgs),
    DiffImpact(DiffImpactArgs),
    Explain(ExplainArgs),
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

#[derive(Debug, Args)]
pub struct FindArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub code_only: bool,
    #[arg(long, default_value_t = false)]
    pub exclude_tests: bool,
}

#[derive(Debug, Args)]
pub struct RefsArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub code_only: bool,
    #[arg(long, default_value_t = false)]
    pub exclude_tests: bool,
}

#[derive(Debug, Args)]
pub struct ContextArgs {
    #[arg(long)]
    pub task: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 1200)]
    pub budget: usize,
    #[arg(long, default_value_t = false)]
    pub code_only: bool,
    #[arg(long, default_value_t = false)]
    pub exclude_tests: bool,
}

#[derive(Debug, Args)]
pub struct TestsForArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub include_support: bool,
}

#[derive(Debug, Args)]
pub struct VerifyPlanArgs {
    #[arg(long = "changed-file", required = true)]
    pub changed_files: Vec<String>,
    #[arg(long = "max-targeted")]
    pub max_targeted: Option<usize>,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DiffImpactArgs {
    #[arg(long = "changed-file", required = true)]
    pub changed_files: Vec<String>,
    #[arg(long = "changed-line")]
    pub changed_lines: Vec<String>,
    #[arg(long, default_value_t = 2)]
    pub max_distance: u32,
    #[arg(long, default_value_t = true)]
    pub include_tests: bool,
    #[arg(long, default_value_t = false)]
    pub include_imports: bool,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub include_snippets: bool,
}
