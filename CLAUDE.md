# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this
repository.

## What This Is

repo-scout is a Rust CLI for local repository indexing, code navigation, and structural analysis. It
parses ASTs via tree-sitter (Rust, Go, Python, TypeScript), stores symbols in SQLite
(`<repo>/.repo-scout/index.db`), and provides deterministic terminal + JSON output for agent-driven
and CI workflows.

## Build, Test, Lint

```bash
just check              # fmt-check + clippy + test (use before PRs)
cargo build             # debug build
cargo build --release   # release build
cargo test              # full test suite
cargo test <test_name>  # single test
cargo fmt               # format code
cargo clippy --all-targets --all-features -- -D warnings  # strict lints
```

### Contract Validation (required before PRs)

```bash
just contract-check                  # all contract checks
just contract-tdd origin/main        # TDD cycle validation
just contract-evidence               # evidence packet validation
```

### Dogfooding (required per feature slice)

```bash
just dogfood-pre <symbol>            # before changes: index + find + refs (JSON)
just dogfood-post <symbol>           # after changes: index + find + refs + test
```

## Architecture

**Layered design** with clear separation:

1. **CLI** (`src/cli.rs`, `src/main.rs`) — Clap derive-based parsing, command dispatch, structured
   exit codes (2=usage, 3=stale index, 4=internal, 5=partial)
2. **Indexer** (`src/indexer/`) — File discovery (`files.rs`), text tokenization (`text.rs`),
   language adapters in `src/indexer/languages/{rust,go,python,typescript}.rs` using tree-sitter AST
   parsing
3. **Query** (`src/query/`) — Symbol find/refs, impact analysis, refactor planning (`planning.rs`),
   verification (`verification.rs`), diagnostics (`diagnostics.rs`), orientation (`orientation.rs`)
4. **Store** (`src/store/`) — SQLite schema management with versioned migrations, content-hash
   (blake3) change detection
5. **Output** (`src/output.rs`) — Dual-mode: human-readable + `--json` with versioned schemas
   (`JSON_SCHEMA_VERSION` v1/v2/v3)

## Error Handling

- **Production code (`src/`)**: Never use `unwrap()`, `expect()`, or `panic!()`. Return
  `Result<T, anyhow::Error>` with `thiserror` for structured errors and `anyhow::Context` for
  wrapping.
- **Test code (`tests/`)**: `unwrap()`/`expect()` allowed for fixture setup and assertions. `panic!`
  allowed in test helpers.
- Custom `AppError` with `ErrorKind` variants (`Usage`, `Index`, `Partial`, `Internal`) mapping to
  specific exit codes.

## Testing

- Integration-style tests using `assert_cmd`, `predicates`, `tempfile` via helpers in
  `tests/common/mod.rs`
- Tests organized by milestone: `tests/milestone*.rs`
- All tests create isolated temp repos — no shared mutable state

## Strict TDD Workflow

Every feature slice follows Red → Green → Refactor:

1. **RED**: Write failing test, commit with `RED:` prefix
2. **GREEN**: Minimal implementation to pass, commit with `GREEN:` prefix
3. **REFACTOR**: Improve without behavior change, commit with `REFACTOR:` prefix

Other allowed prefixes: `DOCS:`, `CHORE:`, `BUILD:`, `TEST:`

## Contract System v2

- Core contracts (always active): `contracts/core/` — TDD enforcement, risk tiers, architecture,
  security, performance, review
- Active language contract: `contracts/languages/RUST_CODING_CONTRACT.md` (Python/TS contracts exist
  but are intentionally not installed)
- CI enforced via `.github/workflows/contract-gates.yml`
- When AGENTS.md and contract assets conflict, the stricter rule wins

## Key Conventions

- Rust 2024 edition, default rustfmt (4-space indent)
- `snake_case` functions/modules, `UpperCamelCase` types, `SCREAMING_SNAKE_CASE` constants
- CLI flags: lowercase/kebab-case (`--json`, `--code-only`)
- Functions: 70 lines max (excluding signature/attributes)
- All public items need doc comments
- Declare risk tier (0/1/2/3) before implementation; tier 2/3 requires adversarial review checklist
- Fixed-width integers for serialized data, units in variable names
- Immutable by default, smallest possible scope for variables
