# Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` wires the CLI entrypoint; `src/cli.rs` defines clap parsing and command routing.
- `src/indexer/` handles file walking, text indexing, and Rust AST extraction (`files.rs`,
  `text.rs`, `rust_ast.rs`).
- `src/query/` contains query logic for `find` and `refs`; `src/store/` owns SQLite schema and
  persistence.
- Integration tests live in `tests/` (milestone-focused specs plus `tests/common/` helpers). Docs
  live in `docs/`, and planning artifacts in `agents/`.

## Build, Test, and Development Commands

- `cargo build` compiles the CLI.
- `cargo run -- --help` shows CLI usage; `cargo run -- index --repo /path/to/repo` builds an index.
- `cargo run -- find <symbol> --repo /path/to/repo` and
  `cargo run -- refs <symbol> --repo /path/to/repo` query the index.
- `cargo test` runs the full integration suite.
- `cargo fmt` formats Rust sources using default rustfmt rules.

## Coding Style & Naming Conventions

- Rust 2024 edition; default rustfmt formatting (4-space indent, trailing commas, etc.).
- Use `snake_case` for modules/functions/files, `UpperCamelCase` for types, and
  `SCREAMING_SNAKE_CASE` for constants.
- CLI subcommands and flags should stay lowercase/kebab-case to match existing commands (`find`,
  `refs`, `--json`).

## Testing Guidelines

- Tests are integration-style and use `assert_cmd`, `predicates`, and `tempfile` from
  `tests/common/`.
- Name new tests to match the milestone pattern (e.g., `tests/milestone6_new_behavior.rs`) and
  prefer end-to-end flows.
- Run `cargo test` before PRs; add new tests when changing ranking, JSON output, or AST behavior.

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

## Commit & Pull Request Guidelines

- Commit subjects in this repo are imperative, sentence-case, and unprefixed (e.g., “Implement
  Milestone 4 ranking and JSON query contract via TDD”).
- PRs should include a short summary, relevant command output (`cargo test`), and doc updates in
  `README.md` or `docs/` when CLI or output changes.

## Security & Configuration Notes

- Index data is stored under the target repo at `.repo-scout/index.db`; do not commit it.
- The CLI is local-only and does not require network access.

## ExecPlans

When writing complex features or significant refactors, use an ExecPlan (as described in
@agents/PLANS.md) from design to implementation.
