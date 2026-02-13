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
    #[command(about = "Index a repository into the local SQLite database")]
    Index(RepoArgs),
    #[command(about = "Show index status and health")]
    Status(RepoArgs),
    #[command(about = "Find symbol definitions by name")]
    Find(FindArgs),
    #[command(about = "Find all references to a symbol")]
    Refs(RefsArgs),
    #[command(about = "Show what depends on a symbol (callers, importers)")]
    Impact(QueryArgs),
    #[command(about = "Find code relevant to a task description")]
    Context(ContextArgs),
    #[command(about = "Find test files that cover a symbol")]
    TestsFor(TestsForArgs),
    #[command(about = "Suggest test commands after changing files")]
    VerifyPlan(VerifyPlanArgs),
    #[command(about = "Analyze blast radius of file changes")]
    DiffImpact(DiffImpactArgs),
    #[command(about = "Show symbol details: signature, call graph, source")]
    Explain(ExplainArgs),
    #[command(about = "Extract source code for a symbol")]
    Snippet(SnippetArgs),
    #[command(about = "Show file structure: signatures and definitions without bodies")]
    Outline(OutlineArgs),
    #[command(about = "Show whole-repo structural overview")]
    Summary(RepoArgs),
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
    #[arg(long = "max-results")]
    pub max_results: Option<u32>,
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
    #[arg(long = "max-results")]
    pub max_results: Option<u32>,
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
    pub budget: u32,
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
    #[arg(long = "changed-line")]
    pub changed_lines: Vec<String>,
    #[arg(long = "changed-symbol")]
    pub changed_symbols: Vec<String>,
    #[arg(long = "max-targeted")]
    pub max_targeted: Option<u32>,
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
    #[arg(long = "changed-symbol")]
    pub changed_symbols: Vec<String>,
    #[arg(long, default_value_t = 2)]
    pub max_distance: u32,
    #[arg(long = "max-results", default_value_t = 30)]
    pub max_results: u32,
    #[arg(long, default_value_t = false)]
    pub no_limit: bool,
    #[arg(long, default_value_t = false, conflicts_with = "exclude_tests")]
    pub include_tests: bool,
    #[arg(long, default_value_t = false, conflicts_with = "include_tests")]
    pub exclude_tests: bool,
    #[arg(long, default_value_t = false)]
    pub include_imports: bool,
    #[arg(long, default_value_t = false)]
    pub exclude_changed: bool,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct OutlineArgs {
    pub file: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SnippetArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 0)]
    pub context: u32,
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
