# Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` wires the CLI entrypoint; `src/cli.rs` defines clap parsing and command routing.
- `src/indexer/` handles file walking, text indexing, and Rust AST extraction (`files.rs`,
  `text.rs`, `rust_ast.rs`).
- `src/query/` contains query logic for `find` and `refs`; `src/store/` owns SQLite schema and
  persistence.
- Integration tests live in `tests/` (milestone-focused specs plus `tests/common/` helpers). Docs
  live in `docs/`, planning artifacts in `agents/`, and Contract System v2 assets in
  `contracts/`, `templates/`, `checklists/`, and `scripts/`.

## Build, Test, and Development Commands

- `cargo build` compiles the CLI.
- `cargo run -- --help` shows CLI usage; `cargo run -- index --repo /path/to/repo` builds an index.
- `cargo run -- find <symbol> --repo /path/to/repo` and
  `cargo run -- refs <symbol> --repo /path/to/repo` query the index.
- `cargo test` runs the full integration suite.
- `cargo fmt` formats Rust sources using default rustfmt rules.
- `just check` runs `fmt`, `clippy`, and `test`.
- `just contract-check` runs local Contract System v2 validators.

## Coding Style & Naming Conventions

- Rust 2024 edition; default rustfmt formatting (4-space indent, trailing commas, etc.).
- Use `snake_case` for modules/functions/files, `UpperCamelCase` for types, and
  `SCREAMING_SNAKE_CASE` for constants.
- CLI subcommands and flags should stay lowercase/kebab-case to match existing commands (`find`,
  `refs`, `--json`).

## Strict TDD and Evidence Rules

- Production code must not be written before a failing test exists for that feature slice.
- Enforce Red -> Green -> Refactor ordering for every feature slice.
- Commit subjects for meaningful changes must use one allowed prefix:
  `RED`, `GREEN`, `REFACTOR`, `DOCS`, `CHORE`, `BUILD`, `TEST`.
- Evidence is primarily captured in PR body headings in `.github/pull_request_template.md`.
  `.evidence/EVIDENCE_PACKET.md` is optional.
- Required local validators before PR:
  - `bash scripts/validate_tdd_cycle.sh --base origin/main`
  - `bash scripts/validate_evidence_packet.sh --pr-body .github/pull_request_template.md`

## Risk Tier Rules

- Declare risk tier (`0 | 1 | 2 | 3`) before implementation.
- If uncertain between tiers, choose the higher tier.
- Apply controls from `contracts/core/RISK_TIER_POLICY.md`.
- For Tier 2 and Tier 3 work, complete `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md`.

## Dogfooding Rules (Codex Enforcement)

- Treat `repo-scout` as the first navigation/query tool when working in this repository.
- Before implementing a feature slice, run:
  - `cargo run -- index --repo .`
  - `cargo run -- find <target_symbol> --repo . --json`
  - `cargo run -- refs <target_symbol> --repo . --json`
- After implementing a feature slice, run:
  - `cargo run -- index --repo .`
  - `cargo run -- find <target_symbol> --repo .`
  - `cargo run -- refs <target_symbol> --repo .`
  - `cargo test`
- If dogfooding reveals a defect, write a failing integration test first, then implement the minimal
  fix, then refactor with the full suite passing (strict red-green-refactor).
- Every milestone should include at least one dogfood transcript in planning artifacts or PR notes.

## Testing Guidelines

- Tests are integration-style and use `assert_cmd`, `predicates`, and `tempfile` from
  `tests/common/`.
- Name new tests to match the milestone pattern (e.g., `tests/milestone6_new_behavior.rs`) and
  prefer end-to-end flows.
- Run `cargo test` before PRs; add new tests when changing ranking, JSON output, or AST behavior.

## Commit & Pull Request Guidelines

- Commit subjects should use contract prefixes and imperative summary text after the prefix (example:
  `GREEN: Implement milestone 4 ranking and JSON query contract`).
- PRs must include required evidence headings from `.github/pull_request_template.md`, plus dogfood
  evidence and docs/plans updates when behavior changes.
- Contract validators and CI gate must pass before merge.

## Security & Configuration Notes

- Index data is stored under the target repo at `.repo-scout/index.db`; do not commit it.
- The CLI is local-only and does not require network access.

## Contract Integration Statement

- This repository follows Contract System v2.
- Core contracts in `contracts/core/` are mandatory.
- The active language contract is `contracts/languages/RUST_CODING_CONTRACT.md`.
- Templates in `templates/`, checklists in `checklists/`, validators in `scripts/`, and CI gate in
  `.github/workflows/contract-gates.yml` are part of the required workflow.
- If `AGENTS.md` guidance and contract assets conflict, the stricter rule wins.

## ExecPlans

When writing complex features or significant refactors, use an ExecPlan (as described in
`agents/PLANS.md`) from design to implementation, and include required contract references and
validator commands in the plan.
