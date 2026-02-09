# 01 - Source Of Truth And Method

## Scope

This audit covers the entire repository:

- Runtime code: `src/`
- Tests and fixtures: `tests/`
- Process/config: `AGENTS.md`, `.github/`, `scripts/`, `Justfile`, `Cargo.toml`
- Documentation and planning artifacts: `README.md`, `docs/`, `agents/`
- Installed contract assets: `contracts/`, `templates/`, `checklists/`

## How Compliance Was Evaluated

### Contract and Convention Extraction

Read and mapped requirements from:

- `contracts/core/AI_AGENT_CORE_CONTRACT.md`
- `contracts/core/TDD_ENFORCEMENT_CONTRACT.md`
- `contracts/core/RISK_TIER_POLICY.md`
- `contracts/core/EVIDENCE_REQUIREMENTS.md`
- `contracts/core/REVIEW_CONTRACT.md`
- `contracts/core/ARCHITECTURE_CONTRACT.md`
- `contracts/core/SECURITY_CONTRACT.md`
- `contracts/core/PERFORMANCE_CONTRACT.md`
- `contracts/core/DEPENDENCY_POLICY.md`
- `contracts/core/INTERACTION_CONTRACT_FOR_CODEX.md`
- `contracts/languages/RUST_CODING_CONTRACT.md`
- `AGENTS.md`

### Upstream Drift Comparison

Compared installed Tiger assets in this repo to upstream:

- Upstream: `/Users/robertguss/Projects/programming_tiger_style`
- Compared paths: `contracts/`, `templates/`, `checklists/`, `scripts/`, `.github/pull_request_template.md`, `.github/workflows/contract-gates.yml`

### Evidence Collection Commands

- Baseline quality gates:
  - `just check`
  - `just contract-check`
- Dogfooding commands:
  - `cargo run -- index --repo .`
  - `cargo run -- find main --repo . --json`
  - `cargo run -- refs main --repo . --json`
- Structural/static audit sampling:
  - Rust function-length extraction for >70 line functions
  - Rust line-length extraction for >100 columns
  - Recursive call site search
  - `bool` and `usize` boundary usage scans
  - ExecPlan section coverage scan
  - Commit-prefix compliance scan across full history

## Important Interpretation Rules

- Findings are mapped against Tiger rules currently installed in this repository, even where those rules are strict or ambiguous.
- Findings that appear to expose weaknesses in Tiger itself are still listed as repo findings where relevant, and are also called out explicitly in `agents/tiger-style-audit/06-tiger-style-framework-feedback.md`.
- This report intentionally distinguishes:
  - Repo-scout remediation items
  - Upstream Tiger-style framework remediation items

## Severity Model Used

The report uses the review contract severity levels:

- `P0`: merge-blocking correctness/safety/security issue
- `P1`: high-risk defect/process gap likely to cause regressions
- `P2`: medium-risk maintainability/reliability/process concern
- `P3`: low-risk improvement or clarification item

