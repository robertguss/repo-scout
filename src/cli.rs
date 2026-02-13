use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    #[command(about = "Show what calls a symbol")]
    Callers(QueryArgs),
    #[command(about = "Show what a symbol calls")]
    Callees(QueryArgs),
    #[command(about = "Show file-level dependency graph")]
    Deps(DepsArgs),
    #[command(about = "Show most-connected symbols (hotspots)")]
    Hotspots(HotspotsArgs),
    #[command(name = "call-path", about = "Find call path between two symbols")]
    CallPath(CallPathArgs),
    #[command(about = "Show structurally related symbols")]
    Related(QueryArgs),
    #[command(about = "Show code health: largest files and functions")]
    Health(HealthArgs),
    #[command(about = "Detect circular file-level dependencies")]
    Circular(CircularArgs),
    #[command(about = "Show repository file tree with stats and dependencies")]
    Tree(TreeArgs),
    #[command(about = "Orientation report: structure, health, hotspots, cycles, recommendations")]
    Orient(OrientArgs),
    #[command(about = "Analyze single-file structure and cohesion for refactoring")]
    Anatomy(AnatomyArgs),
    #[command(about = "Show strongest bidirectional file coupling")]
    Coupling(CouplingArgs),
    #[command(about = "Find symbols that appear unreferenced")]
    Dead(DeadArgs),
    #[command(
        name = "test-gaps",
        about = "Assess test coverage gaps for file or symbol"
    )]
    TestGaps(TestGapsArgs),
    #[command(about = "Prioritized refactoring recommendations")]
    Suggest(SuggestArgs),
    #[command(about = "Show public API boundary for a file")]
    Boundary(BoundaryArgs),
    #[command(
        name = "extract-check",
        about = "Pre-flight analysis for extracting a code range"
    )]
    ExtractCheck(ExtractCheckArgs),
    #[command(name = "move-check", about = "Pre-flight analysis for moving a symbol")]
    MoveCheck(MoveCheckArgs),
    #[command(name = "rename-check", about = "Preview impacts of renaming a symbol")]
    RenameCheck(RenameCheckArgs),
    #[command(
        name = "split-check",
        about = "Pre-flight analysis for splitting a file"
    )]
    SplitCheck(SplitCheckArgs),
    #[command(
        name = "test-scaffold",
        about = "Structured test setup guidance for a symbol"
    )]
    TestScaffold(TestScaffoldArgs),
    #[command(
        name = "safe-steps",
        about = "Generate safe incremental refactoring steps"
    )]
    SafeSteps(SafeStepsArgs),
    #[command(
        name = "verify-refactor",
        about = "Verify refactor completeness between snapshots"
    )]
    VerifyRefactor(VerifyRefactorArgs),
}

#[derive(Debug, Args)]
pub struct HealthArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long, default_value_t = 20)]
    pub top: u32,
    #[arg(long, default_value_t = 0)]
    pub threshold: u32,
    #[arg(long, default_value_t = false)]
    pub large_files: bool,
    #[arg(long, default_value_t = false)]
    pub large_functions: bool,
    #[arg(long = "save-baseline", default_value_t = false)]
    pub save_baseline: bool,
    #[arg(long, default_value_t = false)]
    pub diff: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct TreeArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long, default_value_t = 3)]
    pub depth: u32,
    #[arg(long = "no-deps", default_value_t = false)]
    pub no_deps: bool,
    #[arg(long)]
    pub focus: Option<String>,
    #[arg(long, default_value_t = false)]
    pub symbols: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct OrientArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long, default_value_t = 2)]
    pub depth: u32,
    #[arg(long, default_value_t = 5)]
    pub top: u32,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct CircularArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long = "max-length", default_value_t = 10)]
    pub max_length: u32,
    #[arg(long)]
    pub json: bool,
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
    #[command(flatten)]
    pub filters: SymbolFilterArgs,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum QueryScopeKind {
    #[default]
    All,
    Production,
    Tests,
}

#[derive(Debug, Args, Default)]
pub struct SymbolFilterArgs {
    #[arg(long, value_enum, default_value_t = QueryScopeKind::All)]
    pub scope: QueryScopeKind,
    #[arg(long = "exclude-glob")]
    pub exclude_globs: Vec<String>,
    #[arg(long)]
    pub lang: Option<String>,
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long, default_value_t = false)]
    pub include_fixtures: bool,
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
    #[arg(long, default_value_t = false)]
    pub compact: bool,
    #[command(flatten)]
    pub filters: SymbolFilterArgs,
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
    #[arg(long, default_value_t = false)]
    pub compact: bool,
    #[command(flatten)]
    pub filters: SymbolFilterArgs,
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
    #[arg(long = "changed-file")]
    pub changed_files: Vec<String>,
    #[arg(long = "changed-line")]
    pub changed_lines: Vec<String>,
    #[arg(long = "changed-symbol")]
    pub changed_symbols: Vec<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long, default_value_t = false)]
    pub unstaged: bool,
    #[arg(long = "max-targeted")]
    pub max_targeted: Option<u32>,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DiffImpactArgs {
    #[arg(long = "changed-file")]
    pub changed_files: Vec<String>,
    #[arg(long = "changed-line")]
    pub changed_lines: Vec<String>,
    #[arg(long = "changed-symbol")]
    pub changed_symbols: Vec<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long, default_value_t = false)]
    pub unstaged: bool,
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
pub struct DepsArgs {
    pub file: String,
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
    #[arg(long, default_value_t = false)]
    pub compact: bool,
    #[command(flatten)]
    pub filters: SymbolFilterArgs,
}

#[derive(Debug, Args)]
pub struct HotspotsArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 10)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct CallPathArgs {
    pub from: String,
    pub to: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 10)]
    pub max_depth: u32,
}

#[derive(Debug, Args)]
pub struct AnatomyArgs {
    pub file: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub clusters: bool,
    #[arg(long, default_value_t = false)]
    pub cohesion: bool,
    #[arg(long = "suggest-split", default_value_t = false)]
    pub suggest_split: bool,
}

#[derive(Debug, Args)]
pub struct CouplingArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 20)]
    pub limit: u32,
}

#[derive(Debug, Args)]
pub struct DeadArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub filters: SymbolFilterArgs,
}

#[derive(Debug, Args)]
pub struct TestGapsArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    pub target: String,
    #[arg(long = "min-risk")]
    pub min_risk: Option<String>,
}

#[derive(Debug, Args)]
pub struct SuggestArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = 10)]
    pub top: u32,
    #[arg(long, default_value_t = false)]
    pub safe_only: bool,
    #[arg(long = "min-score")]
    pub min_score: Option<f64>,
}

#[derive(Debug, Args)]
pub struct BoundaryArgs {
    pub file: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long = "public-only", default_value_t = false)]
    pub public_only: bool,
}

#[derive(Debug, Args)]
pub struct ExtractCheckArgs {
    pub symbol: String,
    #[arg(long)]
    pub lines: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct MoveCheckArgs {
    pub symbol: String,
    #[arg(long)]
    pub to: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct RenameCheckArgs {
    pub symbol: String,
    #[arg(long)]
    pub to: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SplitCheckArgs {
    pub file: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub auto: bool,
    #[arg(long)]
    pub groups: Option<String>,
}

#[derive(Debug, Args)]
pub struct TestScaffoldArgs {
    pub symbol: String,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum SafeStepsAction {
    Extract,
    Move,
    Rename,
    Split,
}

#[derive(Debug, Args)]
pub struct SafeStepsArgs {
    pub symbol: String,
    #[arg(long, value_enum)]
    pub action: SafeStepsAction,
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub lines: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Debug, Args)]
pub struct VerifyRefactorArgs {
    #[arg(long)]
    pub repo: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub before: String,
    #[arg(long)]
    pub after: Option<String>,
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}
