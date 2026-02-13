# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

## What This Is

repo-scout is a local, deterministic Rust CLI that indexes repositories into SQLite and answers
code-navigation queries (find, refs, impact, context, tests-for, verify-plan, diff-impact, explain).
It uses tree-sitter for AST extraction across Rust, TypeScript, Python, and Go.

## Build & Test Commands

```bash
just build          # cargo build
just test           # cargo test
just clippy         # cargo clippy --all-targets --all-features -- -D warnings
just fmt            # cargo fmt
just check          # fmt-check + clippy + test (full pre-push gate)

# Run a single test
cargo test milestone50 -- --nocapture

# TDD workflow
just tdd-red <test_name>    # run expecting failure
just tdd-green <test_name>  # run expecting pass
just tdd-refactor           # cargo test (full suite)
```

## Architecture

```
src/
├── main.rs              # Command dispatch + all command handlers
├── cli.rs               # Clap derive definitions
├── output.rs            # Terminal and JSON rendering
├── store/
│   ├── mod.rs           # DB bootstrap, corruption recovery
│   └── schema.rs        # DDL, SCHEMA_VERSION = 3
├── indexer/
│   ├── mod.rs           # Incremental indexing coordinator, edge resolution
│   ├── files.rs         # Ignore-aware file discovery
│   ├── text.rs          # Token occurrence extraction
│   ├── rust_ast.rs      # Rust AST extraction (definitions/references)
│   └── languages/       # Language adapters (rust, typescript, python, go)
└── query/
    └── mod.rs           # All query implementations (find, refs, impact, etc.)
```

Key design: `query/mod.rs` (121k) contains all query logic in one file. `main.rs` (28k) contains all
command handlers. These are large files by design.

## Storage Model

SQLite DB at `<repo>/.repo-scout/index.db` with tables: `meta`, `indexed_files`, `text_occurrences`,
`ast_definitions`, `ast_references`, `symbols_v2`, `symbol_edges_v2`.

## Key Patterns

- **Determinism is paramount**: All queries must produce identical output given identical input.
  Enforce via explicit SQL ordering, stable tie-breakers, path normalization, fixed JSON shapes.
- **AST-first, text-fallback**: `find`/`refs` prefer AST results, fall back to text token matching.
- **Incremental indexing**: Files are skipped if their blake3 hash hasn't changed. Stale rows are
  pruned for deleted files.
- **Edge resolution uses SymbolKey**: exact qualified_symbol → exact (file_path, symbol) → unique
  global match → skip unresolved (fail-safe).
- **Language adapters** in `src/indexer/languages/` follow a normalized extraction contract for
  definitions, references, and call edges.

## Contract System

The repo uses Contract System v2 with strict TDD and evidence enforcement:

- Contracts: `contracts/core/` and `contracts/languages/RUST_CODING_CONTRACT.md`
- Validators: `scripts/validate_tdd_cycle.sh`, `scripts/validate_evidence_packet.sh`
- Pre-PR checks: `just contract-check`
- Docs consistency: `just docs-consistency`

## Test Structure

Tests live in `tests/` as integration tests named `milestone<N>_<feature>.rs`. Fixtures are in
`tests/fixtures/` organized by phase. Tests use `assert_cmd` for CLI testing and `tempfile` for
isolated repos.

## Phase Gates (CI-equivalent)

```bash
just phase15-convergence-pack .    # cross-language tests-for/verify-plan contracts
just phase16-deterministic-replay . # stable JSON output across runs
just phase16-benchmark-pack .       # timing guardrails
just phase16-release-checklist .    # closure quality gates
just phase18-maintenance-pack .     # backlog/freshness guardrails
just e2e-release-matrix .          # full release matrix
```

## Coding Style & Naming Conventions

- Rust 2024 edition; default rustfmt formatting (4-space indent, trailing commas, etc.).
- Use `snake_case` for modules/functions/files, `UpperCamelCase` for types, and
  `SCREAMING_SNAKE_CASE` for constants.
- CLI subcommands and flags should stay lowercase/kebab-case to match existing commands (`find`,
  `refs`, `--json`).

## Strict TDD and Evidence Rules

- Production code must not be written before a failing test exists for that feature slice.
- Enforce Red -> Green -> Refactor ordering.
- Record evidence for each slice in PR body or evidence packet.
- Required commit prefixes (if enforced): `RED`, `GREEN`, `REFACTOR`, `DOCS`, `CHORE`, `BUILD`,
  `TEST`.

## Risk Tier Rules

- Declare risk tier (`0 | 1 | 2 | 3`) before implementation.
- If uncertain between tiers, choose the higher tier.
- Require controls from `contracts/core/RISK_TIER_POLICY.md`.

## PR and Review Expectations

- Complete `checklists/PR_CONTRACT_CHECKLIST.md` for meaningful changes.
- Complete `checklists/ADVERSARIAL_REVIEW_CHECKLIST.md` for Tier 2/Tier 3.
- Include required evidence headings in PR body.
- Merge only after contract gates pass.

## Validation Commands (Required Before PR)

```bash
bash scripts/validate_tdd_cycle.sh --base origin/main
bash scripts/validate_evidence_packet.sh --pr-body /tmp/pr_body.md
```

## Dogfooding Rules

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

## Test Error-Handling Policy

- In `tests/` code, `unwrap()` and `expect()` are allowed for fixture setup, UTF-8 decoding, JSON
  parsing, and assertion preconditions where failure should immediately fail the test.
- In `tests/common/`, `panic!` is allowed for terminal helper failures (e.g., bounded retry
  timeouts).
- In `src/` production code, do not introduce `unwrap()`/`expect()`/`panic!` unless an explicit
  contract exception applies.

## Contract Installation Policy

- Contract installation scope is intentionally Rust-only.
- `contracts/languages/PYTHON_CODING_CONTRACT.md` and
  `contracts/languages/TYPESCRIPT_CODING_CONTRACT.md` are intentionally not installed.
- Python and TypeScript are supported as indexed/query languages, but Rust is the only active coding
  contract scope.

## Contract Integration Statement

This repository follows Contract System v2. Core contracts in `contracts/core/` are mandatory.
Language contract activation is declared in `contracts/ACTIVE_LANGUAGE_CONTRACTS.md`, and active
contracts in `contracts/languages/` apply based on changed files. In conflicts, the stricter rule
applies and must be documented in evidence.
