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
